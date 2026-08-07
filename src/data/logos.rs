//! Company and crypto logo images (`/v1beta1/logos`).
//!
//! The logos endpoint is an add-on product: accounts without it get HTTP 403.

use serde::Serialize;

use super::models::Logo;
use crate::error::Result;
use crate::rest::{Credentials, RestClient, encode_segment};

const DATA_BASE: &str = "https://data.alpaca.markets";

/// The query parameter controlling whether a generated placeholder stands in
/// for a missing logo. Omitted when the API default (`true`) is wanted.
#[derive(Debug, Clone, Copy, Serialize)]
struct PlaceholderQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<bool>,
}

/// Client for the logo API.
pub struct LogoClient {
    rest: RestClient,
}

impl LogoClient {
    /// Creates a new logo client.
    pub fn new(creds: Credentials) -> Result<Self> {
        Self::with_base_url(creds, DATA_BASE)
    }

    /// Creates a new logo client targeting a custom base URL instead of the
    /// default Alpaca endpoint (parity with alpaca-py's `url_override`).
    pub fn with_base_url(creds: Credentials, base_url: &str) -> Result<Self> {
        Ok(Self {
            rest: RestClient::new(base_url, creds)?,
        })
    }

    /// `GET /v1beta1/logos/{symbol}` — the logo of a stock or crypto symbol
    /// (e.g. `AAPL`, `BTCUSD`) as a 300x300 PNG.
    ///
    /// The response is an image rather than JSON, so it comes back as
    /// [`Logo`] — raw bytes plus the reported content type. Symbols without a
    /// logo get a generated placeholder image showing the symbol's first
    /// letter; use [`logo_without_placeholder`](Self::logo_without_placeholder)
    /// to get [`Error::Api`](crate::Error::Api) with status 404 instead.
    pub async fn logo(&self, symbol: &str) -> Result<Logo> {
        self.fetch_logo(symbol, None).await
    }

    /// `GET /v1beta1/logos/{symbol}?placeholder=false` — as
    /// [`logo`](Self::logo), but symbols without a logo fail with
    /// [`Error::Api`](crate::Error::Api) (status 404) rather than returning a
    /// placeholder image.
    pub async fn logo_without_placeholder(&self, symbol: &str) -> Result<Logo> {
        self.fetch_logo(symbol, Some(false)).await
    }

    async fn fetch_logo(&self, symbol: &str, placeholder: Option<bool>) -> Result<Logo> {
        let (bytes, content_type) = self
            .rest
            .get_bytes(
                &format!("/v1beta1/logos/{}", encode_segment(symbol)),
                &PlaceholderQuery { placeholder },
            )
            .await?;
        Ok(Logo {
            bytes,
            content_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_query_is_omitted_unless_set() {
        let default = PlaceholderQuery { placeholder: None };
        assert_eq!(
            serde_json::to_value(default).unwrap(),
            serde_json::json!({})
        );
        let explicit = PlaceholderQuery {
            placeholder: Some(false),
        };
        assert_eq!(
            serde_json::to_value(explicit).unwrap(),
            serde_json::json!({"placeholder": false})
        );
    }
}
