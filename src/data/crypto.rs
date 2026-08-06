//! Historical and latest crypto market data (`/v1beta3`).

use super::enums::CryptoFeed;
use super::models::{
    BarsResponse, LatestBarsResponse, LatestQuotesResponse, LatestTradesResponse,
    OrderbooksResponse, QuotesResponse, SnapshotsResponse, TradesResponse,
};
use super::requests::{BarsRequest, LatestRequest, QuotesRequest, TradesRequest};
use crate::error::Result;
use crate::rest::{Credentials, PAGE_LIMIT_BARS_TRADES_QUOTES, RestClient};

const DATA_BASE: &str = "https://data.alpaca.markets";

/// Client for the crypto market data API. The feed (e.g. `us`) is fixed at
/// construction.
pub struct CryptoHistoricalDataClient {
    rest: RestClient,
    feed: CryptoFeed,
}

impl CryptoHistoricalDataClient {
    /// Creates a new crypto data client for the given feed.
    pub fn new(creds: Credentials, feed: CryptoFeed) -> Result<Self> {
        Self::with_base_url(creds, feed, DATA_BASE)
    }

    /// Creates a new crypto data client targeting a custom base URL instead
    /// of the default Alpaca endpoint (parity with alpaca-py's
    /// `url_override`).
    pub fn with_base_url(creds: Credentials, feed: CryptoFeed, base_url: &str) -> Result<Self> {
        Ok(Self {
            rest: RestClient::new(base_url, creds)?,
            feed,
        })
    }

    fn path(&self, suffix: &str) -> String {
        format!("/v1beta3/crypto/{}/{suffix}", self.feed.as_str())
    }

    /// `GET /v1beta3/crypto/{feed}/bars` — historical bars.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn bars(&self, req: &BarsRequest) -> Result<BarsResponse> {
        self.rest
            .get_paginated(&self.path("bars"), req, Some(PAGE_LIMIT_BARS_TRADES_QUOTES))
            .await
    }

    /// `GET /v1beta3/crypto/{feed}/trades` — historical trades.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn trades(&self, req: &TradesRequest) -> Result<TradesResponse> {
        self.rest
            .get_paginated(
                &self.path("trades"),
                req,
                Some(PAGE_LIMIT_BARS_TRADES_QUOTES),
            )
            .await
    }

    /// `GET /v1beta3/crypto/{feed}/quotes` — historical quotes.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn quotes(&self, req: &QuotesRequest) -> Result<QuotesResponse> {
        self.rest
            .get_paginated(
                &self.path("quotes"),
                req,
                Some(PAGE_LIMIT_BARS_TRADES_QUOTES),
            )
            .await
    }

    /// `GET /v1beta3/crypto/{feed}/latest/trades` — latest trade per symbol.
    pub async fn latest_trades(&self, req: &LatestRequest) -> Result<LatestTradesResponse> {
        self.rest.get(&self.path("latest/trades"), req).await
    }

    /// `GET /v1beta3/crypto/{feed}/latest/quotes` — latest quote per symbol.
    pub async fn latest_quotes(&self, req: &LatestRequest) -> Result<LatestQuotesResponse> {
        self.rest.get(&self.path("latest/quotes"), req).await
    }

    /// `GET /v1beta3/crypto/{feed}/latest/bars` — latest bar per symbol.
    pub async fn latest_bars(&self, req: &LatestRequest) -> Result<LatestBarsResponse> {
        self.rest.get(&self.path("latest/bars"), req).await
    }

    /// `GET /v1beta3/crypto/{feed}/latest/orderbooks` — latest orderbook per
    /// symbol.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn latest_orderbooks(&self, req: &LatestRequest) -> Result<OrderbooksResponse> {
        self.rest
            .get_paginated(&self.path("latest/orderbooks"), req, None)
            .await
    }

    /// `GET /v1beta3/crypto/{feed}/snapshots` — snapshot per symbol.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn snapshots(&self, req: &LatestRequest) -> Result<SnapshotsResponse> {
        self.rest
            .get_paginated(&self.path("snapshots"), req, None)
            .await
    }
}
