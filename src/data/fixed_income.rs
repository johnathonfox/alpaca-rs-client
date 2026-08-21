//! Fixed income latest market data (`/v1beta1/fixed_income`).
//!
//! This is the public Market Data API surface (standard API-key auth) and is
//! in beta. Asset discovery (the list of tradable bonds) lives on the Broker
//! API instead — see the broker module.

use super::models::{FixedIncomeLatestPricesResponse, FixedIncomeLatestQuotesResponse};
use super::requests::{FixedIncomeLatestQuotesRequest, FixedIncomeLatestRequest};
use crate::error::Result;
use crate::rest::{Credentials, RestClient};

const DATA_BASE: &str = "https://data.alpaca.markets";

/// Client for the fixed income market data API (beta).
pub struct FixedIncomeDataClient {
    rest: RestClient,
}

impl FixedIncomeDataClient {
    /// Creates a new fixed income data client.
    pub fn new(creds: Credentials) -> Result<Self> {
        Self::with_base_url(creds, DATA_BASE)
    }

    /// Creates a new fixed income data client targeting a custom base URL
    /// instead of the default Alpaca endpoint (parity with alpaca-py's
    /// `url_override`).
    pub fn with_base_url(creds: Credentials, base_url: &str) -> Result<Self> {
        Ok(Self {
            rest: RestClient::new(base_url, creds)?,
        })
    }

    /// `GET /v1beta1/fixed_income/latest/prices` — latest price per ISIN.
    ///
    /// At most 1000 ISINs per request (validated client-side).
    pub async fn latest_prices(
        &self,
        req: &FixedIncomeLatestRequest,
    ) -> Result<FixedIncomeLatestPricesResponse> {
        self.rest
            .get("/v1beta1/fixed_income/latest/prices", req)
            .await
    }

    /// `GET /v1beta1/fixed_income/latest/quotes` — latest quote per ISIN.
    ///
    /// At most 100 ISINs per request (validated client-side).
    pub async fn latest_quotes(
        &self,
        req: &FixedIncomeLatestQuotesRequest,
    ) -> Result<FixedIncomeLatestQuotesResponse> {
        self.rest
            .get("/v1beta1/fixed_income/latest/quotes", req)
            .await
    }
}
