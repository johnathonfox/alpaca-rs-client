//! Request bodies and query parameters for the trading API.
//!
//! Query parameter structs derive [`Serialize`]; `None` fields are skipped.
//! Multi-value fields such as `symbols` are comma-separated strings.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Serialize, Serializer};

use super::enums::{
    ActivityType, AssetAttribute, AssetClass, AssetExchange, AssetStatus, CalendarTimezone,
    ContractType, CryptoChain, DTBPCheck, ExerciseStyle, LocateStatus, Market, OrderClass,
    OrderSide, OrderType, PDTCheck, PositionIntent, QueryOrderStatus, TimeInForce,
    TokenizationIssuer, TokenizationNetwork, TokenizationRequestStatus, TokenizationRequestType,
    TradeConfirmationEmail,
};
use crate::data::enums::Sort;
use crate::error::{Error, Result};

/// Take-profit leg of a bracket order.
#[derive(Debug, Clone, Serialize)]
pub struct TakeProfit {
    /// Limit price of the take-profit leg.
    pub limit_price: Decimal,
}

/// Stop-loss leg of a bracket order.
#[derive(Debug, Clone, Serialize)]
pub struct StopLoss {
    /// Stop price of the stop-loss leg.
    pub stop_price: Decimal,
    /// Limit price of the stop-loss leg (stop-limit).
    pub limit_price: Option<Decimal>,
}

/// A single leg of a multi-leg (options) order.
#[derive(Debug, Clone, Serialize)]
pub struct OrderLeg {
    /// Option contract symbol of the leg.
    pub symbol: String,
    /// Side of the leg.
    pub side: OrderSide,
    /// Quantity multiplier relative to the order's `qty` (serialized as a
    /// JSON string, like all trading-API numbers).
    pub ratio_qty: Decimal,
    /// Whether the leg opens or closes a position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_intent: Option<PositionIntent>,
}

/// A single order request covering all order types (market, limit, stop,
/// stop-limit, trailing stop, bracket legs).
///
/// Use [`OrderRequest::market`] or [`OrderRequest::limit`] for the common
/// cases and set additional fields directly.
#[derive(Debug, Clone, Serialize)]
pub struct OrderRequest {
    /// Symbol to trade.
    pub symbol: String,
    /// Quantity to trade (mutually exclusive with `notional`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qty: Option<Decimal>,
    /// Notional amount to trade (mutually exclusive with `qty`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notional: Option<Decimal>,
    /// Order side.
    pub side: OrderSide,
    /// Order type (serialized as `type`).
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Limit price (limit and stop-limit orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
    /// Stop price (stop and stop-limit orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Decimal>,
    /// Trail percent (trailing stop orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_percent: Option<Decimal>,
    /// Trail price (trailing stop orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_price: Option<Decimal>,
    /// Whether the order is eligible for extended hours.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_hours: Option<bool>,
    /// Client-provided order id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    /// Order class (bracket/OCO/OTO/multi-leg).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_class: Option<OrderClass>,
    /// Take-profit leg.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit: Option<TakeProfit>,
    /// Stop-loss leg.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<StopLoss>,
    /// Whether the order opens or closes a position (options orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_intent: Option<PositionIntent>,
    /// Legs of a multi-leg order (at most 4; required when `order_class` is
    /// `Mleg`, must be absent otherwise).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legs: Option<Vec<OrderLeg>>,
}

impl OrderRequest {
    /// Validates structural rules enforced by the trading API.
    pub(crate) fn validate(&self) -> Result<()> {
        let is_mleg = self.order_class == Some(OrderClass::Mleg);
        match (&self.legs, is_mleg) {
            (None, true) => {
                return Err(Error::InvalidRequest(
                    "multi-leg orders require `legs`".into(),
                ));
            }
            (Some(_), false) => {
                return Err(Error::InvalidRequest(
                    "`legs` requires `order_class` `mleg`".into(),
                ));
            }
            (Some(legs), true) => {
                if legs.is_empty() || legs.len() > 4 {
                    return Err(Error::InvalidRequest(
                        "multi-leg orders need between 1 and 4 legs".into(),
                    ));
                }
                if self.take_profit.is_some() || self.stop_loss.is_some() {
                    return Err(Error::InvalidRequest(
                        "multi-leg orders do not support `take_profit`/`stop_loss`".into(),
                    ));
                }
            }
            (None, false) => {}
        }
        Ok(())
    }
    /// Creates a market order request for a quantity.
    pub fn market(
        symbol: impl Into<String>,
        side: OrderSide,
        qty: Decimal,
        time_in_force: TimeInForce,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            qty: Some(qty),
            notional: None,
            side,
            order_type: OrderType::Market,
            time_in_force,
            limit_price: None,
            stop_price: None,
            trail_percent: None,
            trail_price: None,
            extended_hours: None,
            client_order_id: None,
            order_class: None,
            take_profit: None,
            stop_loss: None,
            position_intent: None,
            legs: None,
        }
    }

    /// Creates a limit order request for a quantity.
    pub fn limit(
        symbol: impl Into<String>,
        side: OrderSide,
        qty: Decimal,
        time_in_force: TimeInForce,
        limit_price: Decimal,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            qty: Some(qty),
            notional: None,
            side,
            order_type: OrderType::Limit,
            time_in_force,
            limit_price: Some(limit_price),
            stop_price: None,
            trail_percent: None,
            trail_price: None,
            extended_hours: None,
            client_order_id: None,
            order_class: None,
            take_profit: None,
            stop_loss: None,
            position_intent: None,
            legs: None,
        }
    }
}

/// Parameters for replacing (amending) an open order.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReplaceOrderRequest {
    /// New quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qty: Option<Decimal>,
    /// New time in force.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    /// New limit price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
    /// New stop price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Decimal>,
    /// New client order id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
}

/// Query parameters for listing orders.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetOrdersRequest {
    /// Order status filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<QueryOrderStatus>,
    /// Maximum number of orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Only orders submitted after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<DateTime<Utc>>,
    /// Only orders submitted until this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<DateTime<Utc>>,
    /// Sort direction of the results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<Sort>,
    /// Comma-separated list of symbols to filter by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<String>,
    /// Filter by order side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<OrderSide>,
    /// Whether to roll up multi-leg orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nested: Option<bool>,
    /// Filter by asset class.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_class: Option<AssetClass>,
    /// Only orders submitted before the order with this id (exclusive;
    /// mutually exclusive with `after_order_id`, do not combine with
    /// `after`/`until`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_order_id: Option<String>,
    /// Only orders submitted after the order with this id (exclusive;
    /// mutually exclusive with `before_order_id`, do not combine with
    /// `after`/`until`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_order_id: Option<String>,
}

/// Query parameters for getting an order by client order id.
#[derive(Debug, Clone, Serialize)]
pub struct GetOrderByClientIdRequest {
    /// The client order id.
    pub client_order_id: String,
}

/// Query parameters for listing crypto wallets (`GET /v2/wallets`). Also used
/// by the perpetuals equivalent (`GET /v2/perpetuals/wallets`), which takes
/// the asset filter only.
///
/// Quirk: with `asset` set, the endpoint answers with a single wallet object
/// instead of an array; [`TradingClient::get_wallets`](super::TradingClient::get_wallets)
/// absorbs both shapes.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetWalletsRequest {
    /// Asset filter (e.g. `"USDC"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    /// Chain filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<CryptoChain>,
}

/// Body for creating a crypto wallet transfer (`POST /v2/wallets/transfers`).
#[derive(Debug, Clone, Serialize)]
pub struct CreateWalletTransferRequest {
    /// Amount to transfer, in units of `asset`.
    pub amount: Decimal,
    /// Destination address.
    pub address: String,
    /// Asset to transfer (e.g. `"USDC"`).
    pub asset: String,
    /// Chain to transfer on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<CryptoChain>,
}

/// Body for whitelisting a crypto address (`POST /v2/wallets/whitelists`).
#[derive(Debug, Clone, Serialize)]
pub struct CreateWhitelistedAddressRequest {
    /// The address to whitelist.
    pub address: String,
    /// The asset the address is whitelisted for.
    pub asset: String,
    /// The chain of the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<CryptoChain>,
}

/// Query parameters for estimating a transfer fee
/// (`GET /v2/wallets/fees/estimate`; also used by the perpetuals equivalent).
#[derive(Debug, Clone, Serialize)]
pub struct TransferFeeEstimateRequest {
    /// Asset to transfer (e.g. `"USDC"`).
    pub asset: String,
    /// The sending address.
    pub from_address: String,
    /// The receiving address.
    pub to_address: String,
    /// Amount to transfer, in units of `asset`.
    pub amount: Decimal,
}

/// Body for creating a crypto perpetuals wallet transfer
/// (`POST /v2/perpetuals/wallets/transfers`).
#[derive(Debug, Clone, Serialize)]
pub struct CreatePerpTransferRequest {
    /// Amount to transfer, in units of `asset`.
    pub amount: Decimal,
    /// Destination address.
    pub address: String,
    /// Asset to transfer (e.g. `"USDC"`).
    pub asset: String,
}

/// Body for whitelisting a crypto perpetuals address
/// (`POST /v2/perpetuals/wallets/whitelists`).
#[derive(Debug, Clone, Serialize)]
pub struct CreatePerpWhitelistedAddressRequest {
    /// The address to whitelist.
    pub address: String,
    /// The asset the address is whitelisted for.
    pub asset: String,
}

/// Body for minting a tokenized asset (`POST /v2/tokenization/mint`).
#[derive(Debug, Clone, Serialize)]
pub struct TokenizationMintRequest {
    /// Symbol of the underlying asset (e.g. `"AAPL"`).
    pub underlying_symbol: String,
    /// Quantity to mint (at most 9 decimal places).
    pub qty: Decimal,
    /// Issuer of the tokenized asset.
    pub issuer: TokenizationIssuer,
    /// Network to mint on.
    pub network: TokenizationNetwork,
    /// Wallet address the minted tokens are delivered to.
    pub wallet_address: String,
    /// Client-provided idempotency reference (documented in the guide,
    /// missing from the schema).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_request_id: Option<String>,
}

/// Query parameters for listing tokenization requests
/// (`GET /v2/tokenization/requests`). All filters are optional.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetTokenizationRequestsRequest {
    /// Only mint or only redeem requests (serialized as `type`).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub request_type: Option<TokenizationRequestType>,
    /// Only requests in this status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TokenizationRequestStatus>,
    /// Only requests for this underlying symbol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlying_symbol: Option<String>,
    /// Only requests from this issuer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<TokenizationIssuer>,
    /// Only requests on this network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<TokenizationNetwork>,
    /// Only requests created after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<DateTime<Utc>>,
    /// Only requests created before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<DateTime<Utc>>,
}

/// Query parameters for closing a single position.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ClosePositionRequest {
    /// Quantity to close.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qty: Option<Decimal>,
    /// Percentage of the position to close.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<Decimal>,
}

/// Query parameters for closing all positions.
#[derive(Debug, Clone, Serialize)]
pub struct CloseAllPositionsRequest {
    /// Whether to also cancel all open orders.
    pub cancel_orders: bool,
}

/// Query parameters for the portfolio history endpoint.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PortfolioHistoryRequest {
    /// Duration of the data (e.g. `1D`, `1W`, `1M`, `1A`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    /// Resolution of the data (e.g. `1Min`, `1H`, `1D`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeframe: Option<String>,
    /// Last day of the data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_end: Option<NaiveDate>,
    /// Whether to include extended hours.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_hours: Option<bool>,
}

/// Serializes a list of asset attributes as a comma-separated query parameter
/// value (e.g. `"has_options,ipo"`).
fn serialize_asset_attributes<S: Serializer>(
    attributes: &Option<Vec<AssetAttribute>>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    match attributes {
        Some(attributes) => {
            let joined = attributes
                .iter()
                .map(AssetAttribute::as_str)
                .collect::<Vec<_>>()
                .join(",");
            serializer.serialize_str(&joined)
        }
        None => serializer.serialize_none(),
    }
}

/// Query parameters for listing assets.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetAssetsRequest {
    /// Asset status filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AssetStatus>,
    /// Asset class filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_class: Option<AssetClass>,
    /// Exchange filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange: Option<AssetExchange>,
    /// Attribute filter (serialized comma-separated). Alpaca matches these
    /// disjunctively: an asset carrying *any* of the listed attributes is
    /// returned, not only those carrying all of them.
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_asset_attributes"
    )]
    pub attributes: Option<Vec<AssetAttribute>>,
}

/// Query parameters for the market calendar.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CalendarRequest {
    /// First day of the range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<NaiveDate>,
    /// Last day of the range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<NaiveDate>,
}

/// Serializes a list of markets as a comma-separated query parameter value
/// (e.g. `"NYSE,OPRA"`).
fn serialize_markets<S: Serializer>(
    markets: &[Market],
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    let joined = markets
        .iter()
        .map(Market::as_str)
        .collect::<Vec<_>>()
        .join(",");
    serializer.serialize_str(&joined)
}

/// Query parameters for the multi-market clock (`GET /v3/clock`), built by
/// [`TradingClient::get_clock_v3`](super::TradingClient::get_clock_v3).
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ClockV3Query {
    /// Markets to report on (serialized comma-separated). An empty list is
    /// omitted, leaving the market selection to the API.
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_markets"
    )]
    pub(crate) markets: Vec<Market>,
    /// Evaluate the clock at this time instead of the current one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) time: Option<DateTime<Utc>>,
}

impl ClockV3Query {
    /// Builds a clock query for the given markets and optional time override.
    pub(crate) fn new(markets: &[Market], time: Option<DateTime<Utc>>) -> Self {
        Self {
            markets: markets.to_vec(),
            time,
        }
    }
}

/// Query parameters for the multi-market calendar
/// (`GET /v3/calendar/{market}`).
///
/// Separate from [`CalendarRequest`], which serves the legacy `/v2/calendar`
/// endpoint: v3 takes a `timezone` rather than a `date_type`, and defaults to
/// a one-week range starting today rather than the full calendar.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CalendarV3Request {
    /// First day of the range (inclusive; default: today).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<NaiveDate>,
    /// Last day of the range (inclusive; default: one week after `start`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<NaiveDate>,
    /// Timezone of the returned session times (default: the market's own).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<CalendarTimezone>,
}

/// Body for creating a watchlist.
#[derive(Debug, Clone, Serialize)]
pub struct CreateWatchlistRequest {
    /// Watchlist name.
    pub name: String,
    /// Symbols to add.
    pub symbols: Vec<String>,
}

/// Body for updating a watchlist.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateWatchlistRequest {
    /// New name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New set of symbols.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
}

/// Body for adding an asset to a watchlist.
#[derive(Debug, Clone, Serialize)]
pub struct AddAssetToWatchlistRequest {
    /// Symbol to add.
    pub symbol: String,
}

/// Partial update body for account configurations.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AccountConfigurationsRequest {
    /// Day trade buying power check.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dtbp_check: Option<DTBPCheck>,
    /// Trade confirmation email setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_confirm_email: Option<TradeConfirmationEmail>,
    /// Whether to suspend trading.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspend_trade: Option<bool>,
    /// Whether to disable shorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_shorting: Option<bool>,
    /// Whether to enable fractional trading.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fractional_trading: Option<bool>,
    /// Maximum margin multiplier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_margin_multiplier: Option<String>,
    /// Pattern day trader check.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdt_check: Option<PDTCheck>,
    /// Maximum options trading level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_options_trading_level: Option<u32>,
}

/// Query parameters for listing corporate action announcements.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CorporateActionsRequest {
    /// Comma-separated list of corporate action types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_types: Option<String>,
    /// Announcements since this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<NaiveDate>,
    /// Announcements until this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<NaiveDate>,
    /// Filter by symbol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Maximum number of announcements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Query parameters for listing option contracts.
#[derive(Debug, Clone, Default, Serialize)]
pub struct OptionContractsRequest {
    /// Comma-separated list of underlying symbols.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlying_symbols: Option<String>,
    /// Contract status filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AssetStatus>,
    /// Exact expiration date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<NaiveDate>,
    /// Minimum expiration date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date_gte: Option<NaiveDate>,
    /// Maximum expiration date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date_lte: Option<NaiveDate>,
    /// Root symbol filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_symbol: Option<String>,
    /// Contract type filter (serialized as `type`).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub contract_type: Option<ContractType>,
    /// Exercise style filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ExerciseStyle>,
    /// Minimum strike price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike_price_gte: Option<Decimal>,
    /// Maximum strike price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike_price_lte: Option<Decimal>,
    /// Maximum number of contracts per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

/// Serializes a list of activity types as a comma-separated query parameter
/// value (e.g. `"FILL,DIV"`).
fn serialize_activity_types<S: Serializer>(
    types: &Option<Vec<ActivityType>>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    match types {
        Some(types) => {
            let joined = types
                .iter()
                .map(ActivityType::as_str)
                .collect::<Vec<_>>()
                .join(",");
            serializer.serialize_str(&joined)
        }
        None => serializer.serialize_none(),
    }
}

/// Query parameters for the account activities endpoints. All fields are
/// optional; pagination is manual via `page_size` and `page_token`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AccountActivitiesRequest {
    /// The activity types to include (serialized comma-separated).
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_activity_types"
    )]
    pub activity_types: Option<Vec<ActivityType>>,
    /// The date for which to see activities (cannot be combined with
    /// `after`/`until`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<NaiveDate>,
    /// Only activities after this timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<DateTime<Utc>>,
    /// Only activities before this timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<DateTime<Utc>>,
    /// Sort direction of the results (default: descending).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<Sort>,
    /// Maximum number of entries per page (at most 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Pagination token from a previous response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

/// Locate quantities must be requested in round lots of this many shares.
const LOCATE_ROUND_LOT: i64 = 100;

/// Maximum number of unique symbols accepted by the locate-quotes endpoint.
const LOCATE_QUOTES_MAX_SYMBOLS: usize = 100;

/// Query parameters for locate quotes (`GET /v1/locates/quotes`), built by
/// [`TradingClient::get_locate_quotes`](super::TradingClient::get_locate_quotes).
#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocateQuotesQuery {
    /// The symbols to quote, comma-separated.
    pub(crate) symbols: String,
}

impl LocateQuotesQuery {
    /// Builds a locate-quotes query, rejecting empty symbol lists and lists of
    /// more than [`LOCATE_QUOTES_MAX_SYMBOLS`] unique symbols (both of which
    /// the API answers with HTTP 400).
    pub(crate) fn new(symbols: &[String]) -> Result<Self> {
        if symbols.is_empty() {
            return Err(Error::InvalidRequest(
                "locate quotes require at least one symbol".into(),
            ));
        }
        // Deduplicate in first-seen order. The cap counts unique symbols, so
        // joining the caller's raw slice would let 150 copies of one symbol
        // through as a 150-entry query the API rejects.
        let mut seen = std::collections::BTreeSet::new();
        let unique: Vec<&str> = symbols
            .iter()
            .map(String::as_str)
            .filter(|symbol| seen.insert(*symbol))
            .collect();
        if unique.len() > LOCATE_QUOTES_MAX_SYMBOLS {
            return Err(Error::InvalidRequest(format!(
                "locate quotes accept at most {LOCATE_QUOTES_MAX_SYMBOLS} unique symbols, got {}",
                unique.len()
            )));
        }
        Ok(Self {
            symbols: unique.join(","),
        })
    }
}

/// Body for creating a short-sale locate (`POST /v1/locates`).
#[derive(Debug, Clone, Serialize)]
pub struct CreateLocateRequest {
    /// Symbol to locate shares of.
    pub symbol: String,
    /// Number of shares to locate; must be a positive round lot of 100.
    pub qty: i64,
    /// Maximum acceptable locate fee per share in USD. Without it, any quoted
    /// fee is accepted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
    /// Reject the locate unless the full quantity is available (default:
    /// `false`, which allows a partial locate).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_or_none: Option<bool>,
}

impl CreateLocateRequest {
    /// Creates a locate request for `qty` shares of `symbol`, accepting any
    /// quoted fee and a partial fill.
    pub fn new(symbol: impl Into<String>, qty: i64) -> Self {
        Self {
            symbol: symbol.into(),
            qty,
            limit_price: None,
            all_or_none: None,
        }
    }

    /// Validates structural rules enforced by the locates API.
    pub(crate) fn validate(&self) -> Result<()> {
        if self.symbol.is_empty() {
            return Err(Error::InvalidRequest("locate requires a symbol".into()));
        }
        if self.qty <= 0 || self.qty % LOCATE_ROUND_LOT != 0 {
            return Err(Error::InvalidRequest(format!(
                "locate qty must be a positive round lot of {LOCATE_ROUND_LOT}, got {}",
                self.qty
            )));
        }
        Ok(())
    }
}

/// Query parameters for listing locates (`GET /v1/locates`). All fields are
/// optional; pagination is manual via `limit` and `page_token`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetLocatesRequest {
    /// Only locates in this status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<LocateStatus>,
    /// Only locates for this symbol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Only locates whose trading date is on or after this date. The locate
    /// trading date is in `America/New_York` and rolls over at 8pm ET.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<NaiveDate>,
    /// Only locates whose trading date is before this date (exclusive), on the
    /// same clock as `start`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<NaiveDate>,
    /// Maximum number of locates per page (1–10000; default 1000).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination token from a previous response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_market_order_request() {
        let req =
            OrderRequest::market("AAPL", OrderSide::Buy, Decimal::new(1, 0), TimeInForce::Day);
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["symbol"], "AAPL");
        assert_eq!(json["qty"], "1");
        assert_eq!(json["side"], "buy");
        assert_eq!(json["type"], "market");
        assert_eq!(json["time_in_force"], "day");
        assert!(json.get("limit_price").is_none());
        assert!(json.get("notional").is_none());
    }

    #[test]
    fn serialize_limit_order_request() {
        let req = OrderRequest::limit(
            "AAPL",
            OrderSide::Sell,
            Decimal::new(5, 0),
            TimeInForce::Gtc,
            Decimal::new(10589, 2),
        );
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["type"], "limit");
        assert_eq!(json["limit_price"], "105.89");
        assert_eq!(json["time_in_force"], "gtc");
    }

    fn spread_request(legs: Vec<OrderLeg>) -> OrderRequest {
        let mut req =
            OrderRequest::market("SPY", OrderSide::Buy, Decimal::new(1, 0), TimeInForce::Day);
        req.order_class = Some(OrderClass::Mleg);
        req.legs = Some(legs);
        req
    }

    #[test]
    fn serialize_mleg_order_request() {
        let req = spread_request(vec![
            OrderLeg {
                symbol: "SPY260619C00500000".into(),
                side: OrderSide::Buy,
                ratio_qty: Decimal::new(1, 0),
                position_intent: Some(PositionIntent::BuyToOpen),
            },
            OrderLeg {
                symbol: "SPY260619C00510000".into(),
                side: OrderSide::Sell,
                ratio_qty: Decimal::new(1, 0),
                position_intent: Some(PositionIntent::SellToOpen),
            },
        ]);
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["order_class"], "mleg");
        assert_eq!(json["legs"][0]["symbol"], "SPY260619C00500000");
        assert_eq!(json["legs"][0]["ratio_qty"], "1");
        assert_eq!(json["legs"][0]["position_intent"], "buy_to_open");
        assert_eq!(json["legs"][1]["side"], "sell");
        assert!(req.validate().is_ok());
    }

    #[test]
    fn mleg_validation() {
        // mleg without legs
        let mut req =
            OrderRequest::market("SPY", OrderSide::Buy, Decimal::new(1, 0), TimeInForce::Day);
        req.order_class = Some(OrderClass::Mleg);
        assert!(req.validate().is_err());

        // legs without mleg
        let mut req =
            OrderRequest::market("SPY", OrderSide::Buy, Decimal::new(1, 0), TimeInForce::Day);
        req.legs = Some(vec![]);
        assert!(req.validate().is_err());

        // too many legs
        let leg = OrderLeg {
            symbol: "SPY260619C00500000".into(),
            side: OrderSide::Buy,
            ratio_qty: Decimal::new(1, 0),
            position_intent: None,
        };
        let req = spread_request(vec![
            leg.clone(),
            leg.clone(),
            leg.clone(),
            leg.clone(),
            leg.clone(),
        ]);
        assert!(req.validate().is_err());

        // bracket legs on an mleg order
        let mut req = spread_request(vec![leg]);
        req.take_profit = Some(TakeProfit {
            limit_price: Decimal::new(1, 0),
        });
        assert!(req.validate().is_err());
    }

    #[test]
    fn serialize_get_orders_request_filters() {
        let req = GetOrdersRequest {
            asset_class: Some(AssetClass::UsOption),
            before_order_id: Some("b0b0b0b0".into()),
            ..Default::default()
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["asset_class"], "us_option");
        assert_eq!(json["before_order_id"], "b0b0b0b0");
        assert!(json.get("after_order_id").is_none());
        assert!(json.get("status").is_none());
    }

    #[test]
    fn serialize_get_assets_request_attributes() {
        let req = GetAssetsRequest {
            status: Some(AssetStatus::Active),
            attributes: Some(vec![
                AssetAttribute::HasOptions,
                AssetAttribute::Other("warp_drive_enabled".into()),
            ]),
            ..Default::default()
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["status"], "active");
        assert_eq!(json["attributes"], "has_options,warp_drive_enabled");
        assert!(json.get("exchange").is_none());

        // An empty request serializes to an empty object.
        let empty = serde_json::to_value(GetAssetsRequest::default()).unwrap();
        assert_eq!(empty, serde_json::json!({}));
    }

    #[test]
    fn serialize_clock_v3_query() {
        let query = ClockV3Query::new(&[Market::NYSE, Market::OPRA, Market::BOATS], None);
        let json = serde_json::to_value(&query).unwrap();
        assert_eq!(json["markets"], "NYSE,OPRA,BOATS");
        assert!(json.get("time").is_none());

        // A market added after this crate was released still serializes to
        // its wire value.
        let query = ClockV3Query::new(&[Market::Other("XPHL".into())], None);
        assert_eq!(serde_json::to_value(&query).unwrap()["markets"], "XPHL");

        // The time override is sent as an RFC 3339 timestamp.
        let time = DateTime::parse_from_rfc3339("2025-06-24T18:15:22Z")
            .unwrap()
            .with_timezone(&Utc);
        let query = ClockV3Query::new(&[Market::XNYS], Some(time));
        let json = serde_json::to_value(&query).unwrap();
        assert_eq!(json["markets"], "XNYS");
        assert_eq!(json["time"], "2025-06-24T18:15:22Z");

        // No markets means no parameter: the API picks its own default set.
        let empty = serde_json::to_value(ClockV3Query::new(&[], None)).unwrap();
        assert_eq!(empty, serde_json::json!({}));
    }

    #[test]
    fn serialize_calendar_v3_request() {
        let req = CalendarV3Request {
            start: Some(NaiveDate::from_ymd_opt(2025, 1, 2).unwrap()),
            end: Some(NaiveDate::from_ymd_opt(2025, 1, 9).unwrap()),
            timezone: Some(CalendarTimezone::Utc),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["start"], "2025-01-02");
        assert_eq!(json["end"], "2025-01-09");
        assert_eq!(json["timezone"], "UTC");

        // An empty request serializes to an empty object.
        let empty = serde_json::to_value(CalendarV3Request::default()).unwrap();
        assert_eq!(empty, serde_json::json!({}));
    }

    #[test]
    fn serialize_account_activities_request() {
        let req = AccountActivitiesRequest {
            activity_types: Some(vec![ActivityType::Fill, ActivityType::Div]),
            date: Some(NaiveDate::from_ymd_opt(2022, 3, 7).unwrap()),
            direction: Some(Sort::Desc),
            page_size: Some(50),
            page_token: Some("tok123".into()),
            ..Default::default()
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["activity_types"], "FILL,DIV");
        assert_eq!(json["date"], "2022-03-07");
        assert_eq!(json["direction"], "desc");
        assert_eq!(json["page_size"], 50);
        assert_eq!(json["page_token"], "tok123");
        assert!(json.get("after").is_none());
        assert!(json.get("until").is_none());

        // An empty request serializes to an empty object.
        let empty = serde_json::to_value(AccountActivitiesRequest::default()).unwrap();
        assert_eq!(empty, serde_json::json!({}));
    }

    #[test]
    fn serialize_create_locate_request() {
        let req = CreateLocateRequest::new("TSLA", 100);
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["symbol"], "TSLA");
        assert_eq!(json["qty"], 100);
        assert!(json.get("limit_price").is_none());
        assert!(json.get("all_or_none").is_none());
        assert!(req.validate().is_ok());

        let req = CreateLocateRequest {
            limit_price: Some(Decimal::new(5, 2)),
            all_or_none: Some(true),
            ..CreateLocateRequest::new("TSLA", 300)
        };
        let json = serde_json::to_value(&req).unwrap();
        // The fee is a trading-API number, so it goes out as a JSON string;
        // the share count is a plain integer.
        assert_eq!(json["limit_price"], "0.05");
        assert_eq!(json["qty"], 300);
        assert_eq!(json["all_or_none"], true);
    }

    #[test]
    fn create_locate_validation_requires_positive_round_lots() {
        for qty in [1, 99, 101, 150, 0, -100] {
            let err = CreateLocateRequest::new("TSLA", qty)
                .validate()
                .expect_err("qty must be a positive round lot of 100");
            assert!(
                matches!(err, Error::InvalidRequest(_)),
                "qty {qty}: {err:?}"
            );
        }
        for qty in [100, 200, 1_000] {
            assert!(CreateLocateRequest::new("TSLA", qty).validate().is_ok());
        }
        assert!(matches!(
            CreateLocateRequest::new("", 100).validate(),
            Err(Error::InvalidRequest(_))
        ));
    }

    #[test]
    fn locate_quotes_query_joins_and_limits_symbols() {
        let query = LocateQuotesQuery::new(&["TSLA".to_string(), "GME".to_string()]).unwrap();
        assert_eq!(serde_json::to_value(&query).unwrap()["symbols"], "TSLA,GME");

        // Duplicates collapse in first-seen order rather than being forwarded:
        // the cap counts unique symbols, so the query must too.
        let dupes =
            LocateQuotesQuery::new(&["TSLA".to_string(), "GME".to_string(), "TSLA".to_string()])
                .unwrap();
        assert_eq!(serde_json::to_value(&dupes).unwrap()["symbols"], "TSLA,GME");

        // 150 copies of one symbol is 1 unique symbol, so it is accepted — and
        // must not go out as a 150-entry list.
        let many = vec!["TSLA".to_string(); 150];
        let query = LocateQuotesQuery::new(&many).unwrap();
        assert_eq!(serde_json::to_value(&query).unwrap()["symbols"], "TSLA");

        // Empty and oversized symbol lists are rejected client-side.
        assert!(matches!(
            LocateQuotesQuery::new(&[]),
            Err(Error::InvalidRequest(_))
        ));
        let too_many: Vec<String> = (0..101).map(|i| format!("SYM{i}")).collect();
        assert!(matches!(
            LocateQuotesQuery::new(&too_many),
            Err(Error::InvalidRequest(_))
        ));
        // The cap counts unique symbols, so duplicates stay under it.
        let duplicates: Vec<String> = std::iter::repeat_n("TSLA".to_string(), 150).collect();
        assert!(LocateQuotesQuery::new(&duplicates).is_ok());
        // Exactly the cap is still accepted.
        let at_cap: Vec<String> = (0..100).map(|i| format!("SYM{i}")).collect();
        assert!(LocateQuotesQuery::new(&at_cap).is_ok());
    }

    #[test]
    fn serialize_get_locates_request() {
        let req = GetLocatesRequest {
            status: Some(LocateStatus::Active),
            symbol: Some("TSLA".into()),
            start: Some(NaiveDate::from_ymd_opt(2026, 1, 2).unwrap()),
            end: Some(NaiveDate::from_ymd_opt(2026, 1, 9).unwrap()),
            limit: Some(50),
            page_token: Some("tok123".into()),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["status"], "active");
        assert_eq!(json["symbol"], "TSLA");
        assert_eq!(json["start"], "2026-01-02");
        assert_eq!(json["end"], "2026-01-09");
        assert_eq!(json["limit"], 50);
        assert_eq!(json["page_token"], "tok123");

        // A status added after this crate was released still serializes to its
        // wire value.
        let req = GetLocatesRequest {
            status: Some(LocateStatus::Other("pending".into())),
            ..Default::default()
        };
        assert_eq!(serde_json::to_value(&req).unwrap()["status"], "pending");

        // An empty request serializes to an empty object.
        let empty = serde_json::to_value(GetLocatesRequest::default()).unwrap();
        assert_eq!(empty, serde_json::json!({}));
    }

    #[test]
    fn serialize_wallet_requests() {
        let req = GetWalletsRequest {
            asset: Some("USDC".into()),
            chain: Some(CryptoChain::Eth),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["asset"], "USDC");
        assert_eq!(json["chain"], "ETH");
        let empty = serde_json::to_value(GetWalletsRequest::default()).unwrap();
        assert_eq!(empty, serde_json::json!({}));

        let req = CreateWalletTransferRequest {
            amount: Decimal::new(10, 0),
            address: "0x42a76C83014e886e639768D84EAF3573b1876844".into(),
            asset: "USDC".into(),
            chain: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        // Trading-API numbers go out as JSON strings.
        assert_eq!(json["amount"], "10");
        assert_eq!(json["asset"], "USDC");
        assert!(json.get("chain").is_none());

        let req = CreateWhitelistedAddressRequest {
            address: "0xf38Ecf5764fD2dEcB0dd9C1E7513a0b6eC0dD08a".into(),
            asset: "USDC".into(),
            chain: Some(CryptoChain::Eth),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["chain"], "ETH");

        let req = TransferFeeEstimateRequest {
            asset: "USDC".into(),
            from_address: "0x3C3380cdFb94dFEEaA41cAD9F58254AE380d752D".into(),
            to_address: "0x42a76C83014e886e639768D84EAF3573b1876844".into(),
            amount: Decimal::new(10, 0),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["amount"], "10");
        assert_eq!(json["asset"], "USDC");
    }

    #[test]
    fn serialize_perp_requests() {
        let req = CreatePerpTransferRequest {
            amount: Decimal::new(25, 1),
            address: "0x42a76C83014e886e639768D84EAF3573b1876844".into(),
            asset: "USDC".into(),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["amount"], "2.5");
        assert_eq!(json["asset"], "USDC");

        let req = CreatePerpWhitelistedAddressRequest {
            address: "0xf38Ecf5764fD2dEcB0dd9C1E7513a0b6eC0dD08a".into(),
            asset: "USDC".into(),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json["address"],
            "0xf38Ecf5764fD2dEcB0dd9C1E7513a0b6eC0dD08a"
        );
        assert_eq!(json["asset"], "USDC");
    }

    #[test]
    fn serialize_tokenization_requests() {
        let req = TokenizationMintRequest {
            underlying_symbol: "AAPL".into(),
            qty: Decimal::new(3, 0),
            issuer: TokenizationIssuer::XStocks,
            network: TokenizationNetwork::Solana,
            wallet_address: "5dXY1aH2tQpV3wXmJg6Z7c8B4nKvF9bA1pQrSt2uVwYxXz".into(),
            client_request_id: Some("my-mint-ref-001".into()),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["underlying_symbol"], "AAPL");
        assert_eq!(json["qty"], "3");
        assert_eq!(json["issuer"], "xstocks");
        assert_eq!(json["network"], "solana");
        assert_eq!(json["client_request_id"], "my-mint-ref-001");

        // The type filter is serialized under the `type` key.
        let req = GetTokenizationRequestsRequest {
            request_type: Some(TokenizationRequestType::Mint),
            status: Some(TokenizationRequestStatus::Completed),
            ..Default::default()
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["type"], "mint");
        assert_eq!(json["status"], "completed");
        assert!(json.get("issuer").is_none());
        let empty = serde_json::to_value(GetTokenizationRequestsRequest::default()).unwrap();
        assert_eq!(empty, serde_json::json!({}));
    }
}
