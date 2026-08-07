//! Historical and latest option market data (`/v1beta1`).
//!
//! The options feed (`opra` or `indicative`) is passed as a query parameter
//! on each call.

use serde::Serialize;

use super::enums::{OptionsFeed, TickType};
use super::models::{
    BarsResponse, ConditionsResponse, LatestQuotesResponse, LatestTradesResponse,
    OptionExchangesResponse, SnapshotsResponse, TradesResponse,
};
use super::requests::{BarsRequest, LatestRequest, TradesRequest};
use crate::error::Result;
use crate::rest::{
    Credentials, PAGE_LIMIT_BARS_TRADES_QUOTES, PAGE_LIMIT_OPTIONS_SNAPSHOTS, RestClient,
    encode_segment,
};

const DATA_BASE: &str = "https://data.alpaca.markets";

/// Query parameters for the per-underlying snapshots endpoint.
#[derive(Debug, Clone, Default, Serialize)]
pub struct OptionSnapshotsRequest {
    /// Data feed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed: Option<OptionsFeed>,
    /// Maximum number of contracts per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

/// Adds an options `feed` query parameter to another request struct.
#[derive(Serialize)]
struct WithFeed<'a, T: Serialize> {
    #[serde(flatten)]
    inner: &'a T,
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<OptionsFeed>,
}

/// Client for the option market data API.
pub struct OptionHistoricalDataClient {
    rest: RestClient,
}

impl OptionHistoricalDataClient {
    /// Creates a new option data client.
    pub fn new(creds: Credentials) -> Result<Self> {
        Self::with_base_url(creds, DATA_BASE)
    }

    /// Creates a new option data client targeting a custom base URL instead
    /// of the default Alpaca endpoint (parity with alpaca-py's
    /// `url_override`).
    pub fn with_base_url(creds: Credentials, base_url: &str) -> Result<Self> {
        Ok(Self {
            rest: RestClient::new(base_url, creds)?,
        })
    }

    fn with_feed<T: Serialize>(inner: &T, feed: Option<OptionsFeed>) -> WithFeed<'_, T> {
        WithFeed { inner, feed }
    }

    /// `GET /v1beta1/options/bars` — historical bars.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn bars(&self, req: &BarsRequest, feed: Option<OptionsFeed>) -> Result<BarsResponse> {
        self.rest
            .get_paginated(
                "/v1beta1/options/bars",
                &Self::with_feed(req, feed),
                Some(PAGE_LIMIT_BARS_TRADES_QUOTES),
            )
            .await
    }

    /// `GET /v1beta1/options/trades` — historical trades.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn trades(
        &self,
        req: &TradesRequest,
        feed: Option<OptionsFeed>,
    ) -> Result<TradesResponse> {
        self.rest
            .get_paginated(
                "/v1beta1/options/trades",
                &Self::with_feed(req, feed),
                Some(PAGE_LIMIT_BARS_TRADES_QUOTES),
            )
            .await
    }

    /// `GET /v1beta1/options/quotes/latest` — latest quote per contract.
    pub async fn latest_quotes(
        &self,
        req: &LatestRequest,
        feed: Option<OptionsFeed>,
    ) -> Result<LatestQuotesResponse> {
        self.rest
            .get(
                "/v1beta1/options/quotes/latest",
                &Self::with_feed(req, feed),
            )
            .await
    }

    /// `GET /v1beta1/options/trades/latest` — latest trade per contract.
    pub async fn latest_trades(
        &self,
        req: &LatestRequest,
        feed: Option<OptionsFeed>,
    ) -> Result<LatestTradesResponse> {
        self.rest
            .get(
                "/v1beta1/options/trades/latest",
                &Self::with_feed(req, feed),
            )
            .await
    }

    /// `GET /v1beta1/options/snapshots` — snapshot per contract.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn snapshots(
        &self,
        req: &LatestRequest,
        feed: Option<OptionsFeed>,
    ) -> Result<SnapshotsResponse> {
        self.rest
            .get_paginated(
                "/v1beta1/options/snapshots",
                &Self::with_feed(req, feed),
                Some(PAGE_LIMIT_OPTIONS_SNAPSHOTS),
            )
            .await
    }

    /// `GET /v1beta1/options/snapshots/{underlying_symbol}` — option chain
    /// snapshots for one underlying.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn snapshots_for_underlying(
        &self,
        underlying_symbol: &str,
        req: &OptionSnapshotsRequest,
    ) -> Result<SnapshotsResponse> {
        self.rest
            .get_paginated(
                &format!(
                    "/v1beta1/options/snapshots/{}",
                    encode_segment(underlying_symbol)
                ),
                req,
                Some(PAGE_LIMIT_OPTIONS_SNAPSHOTS),
            )
            .await
    }

    /// `GET /v1beta1/options/meta/exchanges` — option exchange code mapping.
    pub async fn exchanges(&self) -> Result<OptionExchangesResponse> {
        self.rest.get("/v1beta1/options/meta/exchanges", &()).await
    }

    /// `GET /v1beta1/options/meta/conditions/{ticktype}` — option condition
    /// code table for trades or quotes.
    pub async fn conditions(&self, tick_type: TickType) -> Result<ConditionsResponse> {
        self.rest
            .get(
                &format!("/v1beta1/options/meta/conditions/{}", tick_type.as_str()),
                &(),
            )
            .await
    }
}
