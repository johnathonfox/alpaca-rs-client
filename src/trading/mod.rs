//! Client and models for the Alpaca trading API (`/v2`).
//!
//! The paper trading base URL is `https://paper-api.alpaca.markets`, the live
//! one is `https://api.alpaca.markets`.

pub mod enums;
pub mod models;
pub mod requests;

mod activities;
mod locates;
mod watchlists_by_name;

pub use enums::*;
pub use models::*;
pub use requests::*;

use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::rest::{Credentials, RestClient, encode_segment};

const PAPER_BASE: &str = "https://paper-api.alpaca.markets";
const LIVE_BASE: &str = "https://api.alpaca.markets";

/// Client for the Alpaca trading API.
pub struct TradingClient {
    pub(crate) rest: RestClient,
}

impl TradingClient {
    /// Creates a new trading client. `paper: true` targets the paper trading
    /// environment.
    pub fn new(creds: Credentials, paper: bool) -> Result<Self> {
        let base = if paper { PAPER_BASE } else { LIVE_BASE };
        Self::with_base_url(creds, base)
    }

    /// Creates a new trading client targeting a custom base URL instead of
    /// the default Alpaca endpoint (parity with alpaca-py's `url_override`).
    pub fn with_base_url(creds: Credentials, base_url: &str) -> Result<Self> {
        Ok(Self {
            rest: RestClient::new(base_url, creds)?,
        })
    }

    /// `GET /v2/account` — returns the account.
    pub async fn get_account(&self) -> Result<Account> {
        self.rest.get("/v2/account", &()).await
    }

    /// `GET /v2/account/configurations` — returns the account configuration.
    pub async fn get_account_configurations(&self) -> Result<AccountConfigurations> {
        self.rest.get("/v2/account/configurations", &()).await
    }

    /// `PATCH /v2/account/configurations` — updates the account
    /// configuration.
    pub async fn update_account_configurations(
        &self,
        req: &AccountConfigurationsRequest,
    ) -> Result<AccountConfigurations> {
        self.rest.patch("/v2/account/configurations", req).await
    }

    /// `POST /v2/orders` — submits an order.
    ///
    /// Multi-leg orders (`order_class` `mleg`) are validated client-side
    /// before submission.
    pub async fn submit_order(&self, req: &OrderRequest) -> Result<Order> {
        req.validate()?;
        self.rest.post("/v2/orders", req).await
    }

    /// `GET /v2/orders` — lists orders.
    pub async fn get_orders(&self, req: &GetOrdersRequest) -> Result<Vec<Order>> {
        self.rest.get("/v2/orders", req).await
    }

    /// `GET /v2/orders/{order_id}` — returns a single order.
    pub async fn get_order(&self, order_id: &str) -> Result<Order> {
        self.rest
            .get(&format!("/v2/orders/{}", encode_segment(order_id)), &())
            .await
    }

    /// `GET /v2/orders:by_client_order_id` — returns an order by its client
    /// order id.
    pub async fn get_order_by_client_order_id(&self, client_order_id: &str) -> Result<Order> {
        self.rest
            .get(
                "/v2/orders:by_client_order_id",
                &GetOrderByClientIdRequest {
                    client_order_id: client_order_id.to_string(),
                },
            )
            .await
    }

    /// `PATCH /v2/orders/{order_id}` — replaces an open order.
    pub async fn replace_order(&self, order_id: &str, req: &ReplaceOrderRequest) -> Result<Order> {
        self.rest
            .patch(&format!("/v2/orders/{}", encode_segment(order_id)), req)
            .await
    }

    /// `DELETE /v2/orders` — cancels all open orders.
    pub async fn cancel_all_orders(&self) -> Result<Vec<CancelOrderResult>> {
        self.rest.delete("/v2/orders", &()).await
    }

    /// `DELETE /v2/orders/{order_id}` — cancels a single order.
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        self.rest
            .delete(&format!("/v2/orders/{}", encode_segment(order_id)), &())
            .await
    }

    /// `GET /v2/positions` — lists all open positions.
    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        self.rest.get("/v2/positions", &()).await
    }

    /// `GET /v2/positions/{symbol_or_asset_id}` — returns a single position.
    pub async fn get_position(&self, symbol_or_asset_id: &str) -> Result<Position> {
        self.rest
            .get(
                &format!("/v2/positions/{}", encode_segment(symbol_or_asset_id)),
                &(),
            )
            .await
    }

    /// `DELETE /v2/positions` — closes all open positions.
    pub async fn close_all_positions(
        &self,
        cancel_orders: bool,
    ) -> Result<Vec<CloseAllPositionsResult>> {
        self.rest
            .delete("/v2/positions", &CloseAllPositionsRequest { cancel_orders })
            .await
    }

    /// `DELETE /v2/positions/{symbol_or_asset_id}` — closes a position
    /// (optionally partially, via `qty` or `percentage` in the request).
    pub async fn close_position(
        &self,
        symbol_or_asset_id: &str,
        req: &ClosePositionRequest,
    ) -> Result<Position> {
        self.rest
            .delete(
                &format!("/v2/positions/{}", encode_segment(symbol_or_asset_id)),
                req,
            )
            .await
    }

    /// `POST /v2/positions/{symbol_or_contract_id}/exercise` — exercises an
    /// option contract held in the account.
    pub async fn exercise_option(&self, symbol_or_contract_id: &str) -> Result<()> {
        self.rest
            .post(
                &format!(
                    "/v2/positions/{}/exercise",
                    encode_segment(symbol_or_contract_id)
                ),
                &serde_json::json!({}),
            )
            .await
    }

    /// `POST /v2/positions/{symbol_or_contract_id}/do-not-exercise` — submits
    /// a do-not-exercise instruction for a long option position.
    ///
    /// By default Alpaca auto-exercises contracts that expire in the money by
    /// at least $0.01; this suppresses that for the given contract. The
    /// endpoint returns an empty body on success.
    pub async fn do_not_exercise_option(&self, symbol_or_contract_id: &str) -> Result<()> {
        self.rest
            .post(
                &format!(
                    "/v2/positions/{}/do-not-exercise",
                    encode_segment(symbol_or_contract_id)
                ),
                &serde_json::json!({}),
            )
            .await
    }

    /// `GET /v2/account/portfolio/history` — returns portfolio equity
    /// history.
    pub async fn get_portfolio_history(
        &self,
        req: &PortfolioHistoryRequest,
    ) -> Result<PortfolioHistory> {
        self.rest.get("/v2/account/portfolio/history", req).await
    }

    /// `GET /v2/assets` — lists assets.
    pub async fn get_assets(&self, req: &GetAssetsRequest) -> Result<Vec<Asset>> {
        self.rest.get("/v2/assets", req).await
    }

    /// `GET /v2/assets/{symbol_or_asset_id}` — returns a single asset.
    pub async fn get_asset(&self, symbol_or_asset_id: &str) -> Result<Asset> {
        self.rest
            .get(
                &format!("/v2/assets/{}", encode_segment(symbol_or_asset_id)),
                &(),
            )
            .await
    }

    /// `GET /v2/clock` — returns the market clock.
    pub async fn get_clock(&self) -> Result<Clock> {
        self.rest.get("/v2/clock", &()).await
    }

    /// `GET /v2/calendar` — returns the market calendar.
    pub async fn get_calendar(&self, req: &CalendarRequest) -> Result<Vec<CalendarDay>> {
        self.rest.get("/v2/calendar", req).await
    }

    /// `GET /v3/clock` — returns the current clock of each of the given
    /// markets, including the session (phase) each one is in.
    ///
    /// The multi-market successor to [`get_clock`](Self::get_clock), which
    /// keeps serving the single-market `/v2` clock. An empty `markets` slice
    /// omits the parameter and leaves the selection to the API.
    ///
    /// Use [`get_clock_v3_at`](Self::get_clock_v3_at) to ask what the clocks
    /// look like at some other point in time.
    pub async fn get_clock_v3(&self, markets: &[Market]) -> Result<ClockV3Response> {
        self.rest
            .get("/v3/clock", &ClockV3Query::new(markets, None))
            .await
    }

    /// `GET /v3/clock` evaluated at `time` instead of now — otherwise
    /// identical to [`get_clock_v3`](Self::get_clock_v3).
    pub async fn get_clock_v3_at(
        &self,
        markets: &[Market],
        time: DateTime<Utc>,
    ) -> Result<ClockV3Response> {
        self.rest
            .get("/v3/clock", &ClockV3Query::new(markets, Some(time)))
            .await
    }

    /// `GET /v3/calendar/{market}` — returns the trading calendar of a single
    /// market, with each day's pre/core/post (and, where applicable, lunch)
    /// sessions.
    ///
    /// The multi-market successor to [`get_calendar`](Self::get_calendar),
    /// which keeps serving the US-only `/v2` calendar. The range defaults to
    /// one week from today; widen it via
    /// [`CalendarV3Request::start`]/[`end`](CalendarV3Request::end).
    pub async fn get_calendar_v3(
        &self,
        market: &Market,
        req: &CalendarV3Request,
    ) -> Result<CalendarV3Response> {
        self.rest
            .get(
                &format!("/v3/calendar/{}", encode_segment(market.as_str())),
                req,
            )
            .await
    }

    /// `GET /v2/watchlists` — lists all watchlists.
    pub async fn get_watchlists(&self) -> Result<Vec<Watchlist>> {
        self.rest.get("/v2/watchlists", &()).await
    }

    /// `POST /v2/watchlists` — creates a watchlist.
    pub async fn create_watchlist(&self, req: &CreateWatchlistRequest) -> Result<Watchlist> {
        self.rest.post("/v2/watchlists", req).await
    }

    /// `GET /v2/watchlists/{watchlist_id}` — returns a watchlist including
    /// its assets.
    pub async fn get_watchlist(&self, watchlist_id: &str) -> Result<Watchlist> {
        self.rest
            .get(
                &format!("/v2/watchlists/{}", encode_segment(watchlist_id)),
                &(),
            )
            .await
    }

    /// `PUT /v2/watchlists/{watchlist_id}` — replaces a watchlist.
    pub async fn update_watchlist(
        &self,
        watchlist_id: &str,
        req: &UpdateWatchlistRequest,
    ) -> Result<Watchlist> {
        self.rest
            .put(
                &format!("/v2/watchlists/{}", encode_segment(watchlist_id)),
                req,
            )
            .await
    }

    /// `DELETE /v2/watchlists/{watchlist_id}` — deletes a watchlist.
    pub async fn delete_watchlist(&self, watchlist_id: &str) -> Result<()> {
        self.rest
            .delete(
                &format!("/v2/watchlists/{}", encode_segment(watchlist_id)),
                &(),
            )
            .await
    }

    /// `POST /v2/watchlists/{watchlist_id}` — adds an asset to a watchlist.
    pub async fn add_asset_to_watchlist(
        &self,
        watchlist_id: &str,
        req: &AddAssetToWatchlistRequest,
    ) -> Result<Watchlist> {
        self.rest
            .post(
                &format!("/v2/watchlists/{}", encode_segment(watchlist_id)),
                req,
            )
            .await
    }

    /// `DELETE /v2/watchlists/{watchlist_id}/{symbol}` — removes an asset
    /// from a watchlist.
    pub async fn remove_asset_from_watchlist(
        &self,
        watchlist_id: &str,
        symbol: &str,
    ) -> Result<Watchlist> {
        self.rest
            .delete(
                &format!(
                    "/v2/watchlists/{}/{}",
                    encode_segment(watchlist_id),
                    encode_segment(symbol)
                ),
                &(),
            )
            .await
    }

    /// `GET /v2/corporate_actions/announcements` — lists corporate action
    /// announcements.
    pub async fn get_corporate_action_announcements(
        &self,
        req: &CorporateActionsRequest,
    ) -> Result<Vec<CorporateActionAnnouncement>> {
        self.rest
            .get("/v2/corporate_actions/announcements", req)
            .await
    }

    /// `GET /v2/corporate_actions/announcements/{id}` — returns a single
    /// corporate action announcement.
    pub async fn get_corporate_action_announcement(
        &self,
        id: &str,
    ) -> Result<CorporateActionAnnouncement> {
        self.rest
            .get(
                &format!("/v2/corporate_actions/announcements/{}", encode_segment(id)),
                &(),
            )
            .await
    }

    /// `GET /v2/options/contracts` — lists option contracts.
    pub async fn get_option_contracts(
        &self,
        req: &OptionContractsRequest,
    ) -> Result<OptionContractsResponse> {
        self.rest.get("/v2/options/contracts", req).await
    }

    /// `GET /v2/options/contracts/{symbol_or_id}` — returns a single option
    /// contract.
    pub async fn get_option_contract(&self, symbol_or_id: &str) -> Result<OptionContract> {
        self.rest
            .get(
                &format!("/v2/options/contracts/{}", encode_segment(symbol_or_id)),
                &(),
            )
            .await
    }
}
