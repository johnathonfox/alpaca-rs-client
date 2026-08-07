//! Forex rates (`/v1beta1/forex`).

use super::models::{ForexRatesResponse, LatestForexRatesResponse};
use super::requests::{ForexRatesRequest, LatestForexRatesRequest};
use crate::error::Result;
use crate::rest::{Credentials, RestClient};

const DATA_BASE: &str = "https://data.alpaca.markets";

/// Client for the forex rates API.
pub struct ForexClient {
    rest: RestClient,
}

impl ForexClient {
    /// Creates a new forex client.
    pub fn new(creds: Credentials) -> Result<Self> {
        Self::with_base_url(creds, DATA_BASE)
    }

    /// Creates a new forex client targeting a custom base URL instead of the
    /// default Alpaca endpoint (parity with alpaca-py's `url_override`).
    pub fn with_base_url(creds: Credentials, base_url: &str) -> Result<Self> {
        Ok(Self {
            rest: RestClient::new(base_url, creds)?,
        })
    }

    /// `GET /v1beta1/forex/rates` — historical rates per currency pair.
    ///
    /// Follows `next_page_token` and returns all pages merged. No default
    /// page size is imposed; set [`ForexRatesRequest::limit`] to pick one.
    pub async fn rates(&self, req: &ForexRatesRequest) -> Result<ForexRatesResponse> {
        self.rest
            .get_paginated("/v1beta1/forex/rates", req, None)
            .await
    }

    /// `GET /v1beta1/forex/latest/rates` — latest rate per currency pair.
    pub async fn latest_rates(
        &self,
        req: &LatestForexRatesRequest,
    ) -> Result<LatestForexRatesResponse> {
        self.rest.get("/v1beta1/forex/latest/rates", req).await
    }
}
