//! Corporate actions data (`/v1/corporate-actions`).

use super::models::CorporateActionsResponse;
use super::requests::CorporateActionsRequest;
use crate::error::Result;
use crate::rest::{Credentials, PAGE_LIMIT_CORPORATE_ACTIONS, RestClient};

const DATA_BASE: &str = "https://data.alpaca.markets";

/// Client for the corporate actions API.
pub struct CorporateActionsClient {
    rest: RestClient,
}

impl CorporateActionsClient {
    /// Creates a new corporate actions client.
    pub fn new(creds: Credentials) -> Result<Self> {
        Self::with_base_url(creds, DATA_BASE)
    }

    /// Creates a new corporate actions client targeting a custom base URL
    /// instead of the default Alpaca endpoint (parity with alpaca-py's
    /// `url_override`).
    pub fn with_base_url(creds: Credentials, base_url: &str) -> Result<Self> {
        Ok(Self {
            rest: RestClient::new(base_url, creds)?,
        })
    }

    /// `GET /v1/corporate-actions` — corporate actions data.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn corporate_actions(
        &self,
        req: &CorporateActionsRequest,
    ) -> Result<CorporateActionsResponse> {
        self.rest
            .get_paginated(
                "/v1/corporate-actions",
                req,
                Some(PAGE_LIMIT_CORPORATE_ACTIONS),
            )
            .await
    }
}
