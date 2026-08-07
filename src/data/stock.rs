//! Historical and latest stock market data (`/v2`).

use serde::Serialize;

use super::enums::{Tape, TickType};
use super::models::{
    AuctionsResponse, BarsResponse, ConditionsResponse, ExchangesResponse, LatestBarsResponse,
    LatestQuotesResponse, LatestTradesResponse, QuotesResponse, SnapshotsResponse,
    SymbolAuctionsResponse, SymbolBarResponse, SymbolBarsResponse, SymbolQuoteResponse,
    SymbolQuotesResponse, SymbolSnapshot, SymbolTradeResponse, SymbolTradesResponse,
    TradesResponse,
};
use super::requests::{AuctionsRequest, BarsRequest, LatestRequest, QuotesRequest, TradesRequest};
use crate::error::Result;
use crate::rest::{Credentials, PAGE_LIMIT_BARS_TRADES_QUOTES, RestClient, encode_segment};

const DATA_BASE: &str = "https://data.alpaca.markets";

/// Reserializes a multi-symbol request without its `symbols` parameter: the
/// single-symbol endpoints take the symbol in the path and reject nothing,
/// but sending a redundant `symbols` query parameter is misleading.
fn without_symbols<T: Serialize + ?Sized>(req: &T) -> Result<serde_json::Value> {
    let mut query = serde_json::to_value(req)?;
    if let Some(object) = query.as_object_mut() {
        object.remove("symbols");
    }
    Ok(query)
}

/// The query parameter carrying the tape a condition table applies to.
#[derive(Debug, Clone, Copy, Serialize)]
struct TapeQuery {
    tape: Tape,
}

/// Client for the stock market data API.
pub struct StockHistoricalDataClient {
    rest: RestClient,
}

impl StockHistoricalDataClient {
    /// Creates a new stock data client.
    pub fn new(creds: Credentials) -> Result<Self> {
        Self::with_base_url(creds, DATA_BASE)
    }

    /// Creates a new stock data client targeting a custom base URL instead
    /// of the default Alpaca endpoint (parity with alpaca-py's
    /// `url_override`).
    pub fn with_base_url(creds: Credentials, base_url: &str) -> Result<Self> {
        Ok(Self {
            rest: RestClient::new(base_url, creds)?,
        })
    }

    /// `GET /v2/stocks/bars` — historical bars for multiple symbols.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn bars(&self, req: &BarsRequest) -> Result<BarsResponse> {
        self.rest
            .get_paginated("/v2/stocks/bars", req, Some(PAGE_LIMIT_BARS_TRADES_QUOTES))
            .await
    }

    /// `GET /v2/stocks/trades` — historical trades for multiple symbols.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn trades(&self, req: &TradesRequest) -> Result<TradesResponse> {
        self.rest
            .get_paginated(
                "/v2/stocks/trades",
                req,
                Some(PAGE_LIMIT_BARS_TRADES_QUOTES),
            )
            .await
    }

    /// `GET /v2/stocks/quotes` — historical quotes for multiple symbols.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn quotes(&self, req: &QuotesRequest) -> Result<QuotesResponse> {
        self.rest
            .get_paginated(
                "/v2/stocks/quotes",
                req,
                Some(PAGE_LIMIT_BARS_TRADES_QUOTES),
            )
            .await
    }

    /// `GET /v2/stocks/trades/latest` — latest trade per symbol.
    pub async fn latest_trades(&self, req: &LatestRequest) -> Result<LatestTradesResponse> {
        self.rest.get("/v2/stocks/trades/latest", req).await
    }

    /// `GET /v2/stocks/quotes/latest` — latest quote per symbol.
    pub async fn latest_quotes(&self, req: &LatestRequest) -> Result<LatestQuotesResponse> {
        self.rest.get("/v2/stocks/quotes/latest", req).await
    }

    /// `GET /v2/stocks/bars/latest` — latest bar per symbol.
    pub async fn latest_bars(&self, req: &LatestRequest) -> Result<LatestBarsResponse> {
        self.rest.get("/v2/stocks/bars/latest", req).await
    }

    /// `GET /v2/stocks/snapshots` — snapshot per symbol.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn snapshots(&self, req: &LatestRequest) -> Result<SnapshotsResponse> {
        // Unlike the crypto and options snapshot endpoints, this one returns
        // the symbol-keyed map at the *top level* rather than nested under a
        // `snapshots` key, and it does not paginate. Decode the bare map and
        // wrap it so the return type stays consistent across the clients.
        let snapshots = self.rest.get("/v2/stocks/snapshots", req).await?;
        Ok(SnapshotsResponse {
            snapshots,
            next_page_token: None,
        })
    }

    /// `GET /v2/stocks/auctions` — historical opening and closing auction
    /// prices for multiple symbols.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    ///
    /// Auction data is sourced from the SIP feed only, so it requires an
    /// Algo Trader Plus subscription; `feed` values other than
    /// [`DataFeed::Sip`](super::enums::DataFeed::Sip) are rejected by the
    /// API.
    pub async fn auctions(&self, req: &AuctionsRequest) -> Result<AuctionsResponse> {
        self.rest
            .get_paginated(
                "/v2/stocks/auctions",
                req,
                Some(PAGE_LIMIT_BARS_TRADES_QUOTES),
            )
            .await
    }

    /// `GET /v2/stocks/{symbol}/auctions` — historical auction prices for one
    /// symbol. The `symbols` field of `req` is ignored.
    ///
    /// Follows `next_page_token` and returns all pages merged. Same SIP-only
    /// subscription requirement as [`auctions`](Self::auctions).
    pub async fn auctions_for_symbol(
        &self,
        symbol: &str,
        req: &AuctionsRequest,
    ) -> Result<SymbolAuctionsResponse> {
        self.rest
            .get_paginated(
                &format!("/v2/stocks/{}/auctions", encode_segment(symbol)),
                &without_symbols(req)?,
                Some(PAGE_LIMIT_BARS_TRADES_QUOTES),
            )
            .await
    }

    /// `GET /v2/stocks/{symbol}/bars` — historical bars for one symbol. The
    /// `symbols` field of `req` is ignored.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn bars_for_symbol(
        &self,
        symbol: &str,
        req: &BarsRequest,
    ) -> Result<SymbolBarsResponse> {
        self.rest
            .get_paginated(
                &format!("/v2/stocks/{}/bars", encode_segment(symbol)),
                &without_symbols(req)?,
                Some(PAGE_LIMIT_BARS_TRADES_QUOTES),
            )
            .await
    }

    /// `GET /v2/stocks/{symbol}/trades` — historical trades for one symbol.
    /// The `symbols` field of `req` is ignored.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn trades_for_symbol(
        &self,
        symbol: &str,
        req: &TradesRequest,
    ) -> Result<SymbolTradesResponse> {
        self.rest
            .get_paginated(
                &format!("/v2/stocks/{}/trades", encode_segment(symbol)),
                &without_symbols(req)?,
                Some(PAGE_LIMIT_BARS_TRADES_QUOTES),
            )
            .await
    }

    /// `GET /v2/stocks/{symbol}/quotes` — historical quotes for one symbol.
    /// The `symbols` field of `req` is ignored.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn quotes_for_symbol(
        &self,
        symbol: &str,
        req: &QuotesRequest,
    ) -> Result<SymbolQuotesResponse> {
        self.rest
            .get_paginated(
                &format!("/v2/stocks/{}/quotes", encode_segment(symbol)),
                &without_symbols(req)?,
                Some(PAGE_LIMIT_BARS_TRADES_QUOTES),
            )
            .await
    }

    /// `GET /v2/stocks/{symbol}/trades/latest` — latest trade for one symbol.
    /// The `symbols` field of `req` is ignored.
    pub async fn latest_trade_for_symbol(
        &self,
        symbol: &str,
        req: &LatestRequest,
    ) -> Result<SymbolTradeResponse> {
        self.rest
            .get(
                &format!("/v2/stocks/{}/trades/latest", encode_segment(symbol)),
                &without_symbols(req)?,
            )
            .await
    }

    /// `GET /v2/stocks/{symbol}/quotes/latest` — latest quote for one symbol.
    /// The `symbols` field of `req` is ignored.
    pub async fn latest_quote_for_symbol(
        &self,
        symbol: &str,
        req: &LatestRequest,
    ) -> Result<SymbolQuoteResponse> {
        self.rest
            .get(
                &format!("/v2/stocks/{}/quotes/latest", encode_segment(symbol)),
                &without_symbols(req)?,
            )
            .await
    }

    /// `GET /v2/stocks/{symbol}/bars/latest` — latest bar for one symbol. The
    /// `symbols` field of `req` is ignored.
    pub async fn latest_bar_for_symbol(
        &self,
        symbol: &str,
        req: &LatestRequest,
    ) -> Result<SymbolBarResponse> {
        self.rest
            .get(
                &format!("/v2/stocks/{}/bars/latest", encode_segment(symbol)),
                &without_symbols(req)?,
            )
            .await
    }

    /// `GET /v2/stocks/{symbol}/snapshot` — snapshot for one symbol. The
    /// `symbols` field of `req` is ignored.
    pub async fn snapshot(&self, symbol: &str, req: &LatestRequest) -> Result<SymbolSnapshot> {
        self.rest
            .get(
                &format!("/v2/stocks/{}/snapshot", encode_segment(symbol)),
                &without_symbols(req)?,
            )
            .await
    }

    /// `GET /v2/stocks/meta/conditions/trade` — trade condition code table
    /// for the given tape.
    pub async fn trade_conditions(&self, tape: Tape) -> Result<ConditionsResponse> {
        self.conditions(TickType::Trade, tape).await
    }

    /// `GET /v2/stocks/meta/conditions/quote` — quote condition code table
    /// for the given tape.
    pub async fn quote_conditions(&self, tape: Tape) -> Result<ConditionsResponse> {
        self.conditions(TickType::Quote, tape).await
    }

    async fn conditions(&self, tick_type: TickType, tape: Tape) -> Result<ConditionsResponse> {
        self.rest
            .get(
                &format!("/v2/stocks/meta/conditions/{}", tick_type.as_str()),
                &TapeQuery { tape },
            )
            .await
    }

    /// `GET /v2/stocks/meta/exchanges` — exchange code to exchange name
    /// table.
    pub async fn exchanges(&self) -> Result<ExchangesResponse> {
        self.rest.get("/v2/stocks/meta/exchanges", &()).await
    }
}
