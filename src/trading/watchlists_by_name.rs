//! Watchlist endpoints addressed by name (`/v2/watchlists:by_name`).
//!
//! Every operation the by-id endpoints offer on an existing watchlist is also
//! reachable by the watchlist's name, passed as the `name` query parameter:
//! fetch, replace, append an asset, delete. Creating a watchlist still goes
//! through [`TradingClient::create_watchlist`], and removing a single asset
//! still needs the watchlist id
//! ([`TradingClient::remove_asset_from_watchlist`]) — the API documents no
//! `:by_name` variant for it.
//!
//! Names are unique per account, so these are a convenience over looking the
//! id up first; an unknown name answers with HTTP 404.

use serde::Serialize;

use crate::error::Result;

use super::{AddAssetToWatchlistRequest, TradingClient, UpdateWatchlistRequest, Watchlist};

/// The query parameter naming the watchlist an `:by_name` request applies to.
#[derive(Debug, Clone, Serialize)]
struct WatchlistNameQuery<'a> {
    name: &'a str,
}

impl<'a> WatchlistNameQuery<'a> {
    fn new(name: &'a str) -> Self {
        Self { name }
    }
}

impl TradingClient {
    /// `GET /v2/watchlists:by_name` — returns a watchlist including its
    /// assets, addressed by name instead of id.
    pub async fn get_watchlist_by_name(&self, name: &str) -> Result<Watchlist> {
        self.rest
            .get("/v2/watchlists:by_name", &WatchlistNameQuery::new(name))
            .await
    }

    /// `PUT /v2/watchlists:by_name` — replaces a watchlist addressed by name.
    ///
    /// The request must carry at least one of
    /// [`name`](UpdateWatchlistRequest::name) (which renames the watchlist)
    /// or [`symbols`](UpdateWatchlistRequest::symbols) (which replaces its
    /// asset list wholesale).
    pub async fn update_watchlist_by_name(
        &self,
        name: &str,
        req: &UpdateWatchlistRequest,
    ) -> Result<Watchlist> {
        self.rest
            .put_with_query(
                "/v2/watchlists:by_name",
                &WatchlistNameQuery::new(name),
                req,
            )
            .await
    }

    /// `POST /v2/watchlists:by_name` — appends an asset to a watchlist
    /// addressed by name.
    pub async fn add_asset_to_watchlist_by_name(
        &self,
        name: &str,
        req: &AddAssetToWatchlistRequest,
    ) -> Result<Watchlist> {
        self.rest
            .post_with_query(
                "/v2/watchlists:by_name",
                &WatchlistNameQuery::new(name),
                req,
            )
            .await
    }

    /// `DELETE /v2/watchlists:by_name` — permanently deletes a watchlist
    /// addressed by name. The endpoint returns an empty body on success.
    pub async fn delete_watchlist_by_name(&self, name: &str) -> Result<()> {
        self.rest
            .delete("/v2/watchlists:by_name", &WatchlistNameQuery::new(name))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_query_serializes_as_name_parameter() {
        let query = WatchlistNameQuery::new("My Watchlist");
        assert_eq!(
            serde_json::to_value(&query).unwrap(),
            serde_json::json!({"name": "My Watchlist"})
        );
    }
}
