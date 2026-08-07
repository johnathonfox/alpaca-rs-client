//! Short-sale locate endpoints (`/v1/locates`).
//!
//! Shorting a hard-to-borrow security requires reserving inventory first: read
//! [`Asset::borrow_status`](super::models::Asset::borrow_status), quote the
//! symbol with [`TradingClient::get_locate_quotes`], then reserve shares with
//! [`TradingClient::create_locate`].
//!
//! These endpoints live on the trading host but under `/v1` rather than `/v2`,
//! and they require a live account with hard-to-borrow trading enabled —
//! paper-trading support is not documented, so treat a 403 on paper as
//! expected. Locate fees are non-refundable and locates cannot be reused.

use crate::error::Result;
use crate::rest::encode_segment;

use super::{
    CreateLocateRequest, GetLocatesRequest, Locate, LocateQuotesQuery, LocateQuotesResponse,
    LocatesResponse, TradingClient,
};

impl TradingClient {
    /// `GET /v1/locates/quotes` — returns locate availability and pricing for
    /// the given symbols (at most 100 unique ones, validated client-side).
    ///
    /// Quotes are advisory: they neither reserve shares nor fix the fee of a
    /// later locate. Symbols that cannot be quoted come back in
    /// [`LocateQuotesResponse::errors`] rather than failing the request.
    pub async fn get_locate_quotes(&self, symbols: &[String]) -> Result<LocateQuotesResponse> {
        self.rest
            .get("/v1/locates/quotes", &LocateQuotesQuery::new(symbols)?)
            .await
    }

    /// `POST /v1/locates` — reserves shares for a short sale.
    ///
    /// The quantity must be a positive round lot of 100 (validated
    /// client-side). Unless
    /// [`all_or_none`](CreateLocateRequest::all_or_none) is set, the locate may
    /// come back partially filled, with
    /// [`located_qty`](Locate::located_qty) below the requested quantity.
    pub async fn create_locate(&self, req: &CreateLocateRequest) -> Result<Locate> {
        req.validate()?;
        self.rest.post("/v1/locates", req).await
    }

    /// `GET /v1/locates` — lists the account's locates, newest first.
    ///
    /// Pagination is manual: pass the previous response's
    /// [`next_page_token`](LocatesResponse::next_page_token) back in
    /// [`GetLocatesRequest::page_token`] to fetch the next page. No automatic
    /// page merging is done.
    pub async fn get_locates(&self, req: &GetLocatesRequest) -> Result<LocatesResponse> {
        self.rest.get("/v1/locates", req).await
    }

    /// `GET /v1/locates/{locate_id}` — returns a single locate.
    pub async fn get_locate(&self, locate_id: &str) -> Result<Locate> {
        self.rest
            .get(&format!("/v1/locates/{}", encode_segment(locate_id)), &())
            .await
    }
}
