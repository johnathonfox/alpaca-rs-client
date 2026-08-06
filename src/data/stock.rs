//! Historical and latest stock market data (`/v2`).

use super::models::{
    BarsResponse, LatestBarsResponse, LatestQuotesResponse, LatestTradesResponse, QuotesResponse,
    SnapshotsResponse, TradesResponse,
};
use super::requests::{BarsRequest, LatestRequest, QuotesRequest, TradesRequest};
use crate::error::Result;
use crate::rest::{Credentials, PAGE_LIMIT_BARS_TRADES_QUOTES, RestClient};

const DATA_BASE: &str = "https://data.alpaca.markets";

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
        self.rest
            .get_paginated("/v2/stocks/snapshots", req, None)
            .await
    }
}
