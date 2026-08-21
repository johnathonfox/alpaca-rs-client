//! Crypto perpetual futures latest market data
//! (`/v1beta1/crypto-perps/{loc}/latest/*`). Beta.
//!
//! The `loc` path segment has only one published value (`global`), so it is
//! hardcoded in the paths. The symbol symbology is inconsistent across the
//! Alpaca docs (`BTC-PERP` vs `BTCUSDT.P`); symbols are passed through
//! verbatim.

use super::models::{
    CryptoPerpLatestPricingResponse, LatestBarsResponse, LatestQuotesResponse,
    LatestTradesResponse, OrderbooksResponse,
};
use super::requests::CryptoPerpLatestRequest;
use crate::error::Result;
use crate::rest::{Credentials, RestClient};

const DATA_BASE: &str = "https://data.alpaca.markets";

/// Client for the crypto perpetual futures market data API (beta).
pub struct CryptoPerpDataClient {
    rest: RestClient,
}

impl CryptoPerpDataClient {
    /// Creates a new crypto-perp data client.
    pub fn new(creds: Credentials) -> Result<Self> {
        Self::with_base_url(creds, DATA_BASE)
    }

    /// Creates a new crypto-perp data client targeting a custom base URL
    /// instead of the default Alpaca endpoint (parity with alpaca-py's
    /// `url_override`).
    pub fn with_base_url(creds: Credentials, base_url: &str) -> Result<Self> {
        Ok(Self {
            rest: RestClient::new(base_url, creds)?,
        })
    }

    fn path(suffix: &str) -> String {
        // `global` is the only published value of the `loc` path segment.
        format!("/v1beta1/crypto-perps/global/latest/{suffix}")
    }

    /// `GET /v1beta1/crypto-perps/global/latest/bars` — latest bar per
    /// symbol.
    pub async fn latest_bars(&self, req: &CryptoPerpLatestRequest) -> Result<LatestBarsResponse> {
        self.rest.get(&Self::path("bars"), req).await
    }

    /// `GET /v1beta1/crypto-perps/global/latest/trades` — latest trade per
    /// symbol.
    pub async fn latest_trades(
        &self,
        req: &CryptoPerpLatestRequest,
    ) -> Result<LatestTradesResponse> {
        self.rest.get(&Self::path("trades"), req).await
    }

    /// `GET /v1beta1/crypto-perps/global/latest/quotes` — latest quote per
    /// symbol.
    pub async fn latest_quotes(
        &self,
        req: &CryptoPerpLatestRequest,
    ) -> Result<LatestQuotesResponse> {
        self.rest.get(&Self::path("quotes"), req).await
    }

    /// `GET /v1beta1/crypto-perps/global/latest/orderbooks` — latest
    /// orderbook per symbol.
    pub async fn latest_orderbooks(
        &self,
        req: &CryptoPerpLatestRequest,
    ) -> Result<OrderbooksResponse> {
        self.rest.get(&Self::path("orderbooks"), req).await
    }

    /// `GET /v1beta1/crypto-perps/global/latest/pricing` — latest pricing
    /// (index/mark price, funding rate, open interest) per symbol.
    pub async fn latest_pricing(
        &self,
        req: &CryptoPerpLatestRequest,
    ) -> Result<CryptoPerpLatestPricingResponse> {
        self.rest.get(&Self::path("pricing"), req).await
    }
}
