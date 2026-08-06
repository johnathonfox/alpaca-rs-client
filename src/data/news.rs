//! News data (`/v1beta1`).

use super::models::NewsResponse;
use super::requests::NewsRequest;
use crate::error::Result;
use crate::rest::{Credentials, PAGE_LIMIT_NEWS, RestClient};

const DATA_BASE: &str = "https://data.alpaca.markets";

/// Client for the news API.
pub struct NewsClient {
    rest: RestClient,
}

impl NewsClient {
    /// Creates a new news client.
    pub fn new(creds: Credentials) -> Result<Self> {
        Self::with_base_url(creds, DATA_BASE)
    }

    /// Creates a new news client targeting a custom base URL instead of the
    /// default Alpaca endpoint (parity with alpaca-py's `url_override`).
    pub fn with_base_url(creds: Credentials, base_url: &str) -> Result<Self> {
        Ok(Self {
            rest: RestClient::new(base_url, creds)?,
        })
    }

    /// `GET /v1beta1/news` — latest news articles.
    ///
    /// Follows `next_page_token` and returns all pages merged.
    pub async fn news(&self, req: &NewsRequest) -> Result<NewsResponse> {
        self.rest
            .get_paginated("/v1beta1/news", req, Some(PAGE_LIMIT_NEWS))
            .await
    }
}
