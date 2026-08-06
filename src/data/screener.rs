//! Screener data (`/v1beta1/screener`): most actives and market movers.

use super::models::{MostActives, Movers};
use super::requests::{MarketMoversRequest, MostActivesRequest};
use crate::error::Result;
use crate::rest::{Credentials, RestClient};

const DATA_BASE: &str = "https://data.alpaca.markets";

/// Client for the screener API.
pub struct ScreenerClient {
    rest: RestClient,
}

impl ScreenerClient {
    /// Creates a new screener client.
    pub fn new(creds: Credentials) -> Result<Self> {
        Self::with_base_url(creds, DATA_BASE)
    }

    /// Creates a new screener client targeting a custom base URL instead of
    /// the default Alpaca endpoint (parity with alpaca-py's `url_override`).
    pub fn with_base_url(creds: Credentials, base_url: &str) -> Result<Self> {
        Ok(Self {
            rest: RestClient::new(base_url, creds)?,
        })
    }

    /// `GET /v1beta1/screener/stocks/most-actives` — most active stocks.
    pub async fn most_actives(&self, req: &MostActivesRequest) -> Result<MostActives> {
        self.rest
            .get("/v1beta1/screener/stocks/most-actives", req)
            .await
    }

    /// `GET /v1beta1/screener/{stocks|crypto}/movers` — top market movers.
    ///
    /// The market type is taken from the request and selects the URL path.
    pub async fn movers(&self, req: &MarketMoversRequest) -> Result<Movers> {
        let path = format!("/v1beta1/screener/{}/movers", req.market_type.as_str());
        self.rest.get(&path, req).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::enums::MarketType;

    #[test]
    fn movers_path_follows_market_type() {
        for (market_type, segment) in [
            (MarketType::Stocks, "stocks"),
            (MarketType::Crypto, "crypto"),
        ] {
            let req = MarketMoversRequest::new(market_type, 10);
            let path = format!("/v1beta1/screener/{}/movers", req.market_type.as_str());
            assert_eq!(path, format!("/v1beta1/screener/{segment}/movers"));
        }
    }
}
