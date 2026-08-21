//! Response models for the trading API.
//!
//! The trading API serializes all money and quantity numbers as JSON strings
//! (e.g. `"filled_avg_price": "105.89"`), so numeric fields use
//! [`Decimal`] (with the `serde-str` feature, `Decimal` (de)serializes as a
//! string by default).

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::enums::{
    ActivityType, AssetAttribute, AssetClass, AssetExchange, AssetStatus, BorrowStatus,
    ContractType, DTBPCheck, ExerciseStyle, LocateQuoteErrorCode, LocateStatus, Market,
    MarketPhase, NonTradeActivityStatus, OrderClass, OrderSide, OrderStatus, OrderType, PDTCheck,
    PositionSide, TimeInForce, TradeActivityType, TradeConfirmationEmail,
};

/// A trading account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account id.
    pub id: String,
    /// Account number.
    pub account_number: String,
    /// Account status (e.g. `ACTIVE`).
    pub status: String,
    /// Crypto trading status, if crypto is enabled.
    pub crypto_status: Option<String>,
    /// Currency of the account.
    pub currency: Option<String>,
    /// Current available cash buying power.
    pub buying_power: Decimal,
    /// Reg T initial margin buying power.
    pub regt_buying_power: Option<Decimal>,
    /// Day trading buying power.
    pub daytrading_buying_power: Option<Decimal>,
    /// Buying power without margin.
    pub non_marginable_buying_power: Option<Decimal>,
    /// Cash balance.
    pub cash: Decimal,
    /// Fees accrued.
    pub accrued_fees: Option<Decimal>,
    /// Total portfolio value.
    pub portfolio_value: Decimal,
    /// Whether the account is flagged as a pattern day trader. Omitted by the
    /// API since the 2026 PDT-rule sunset.
    pub pattern_day_trader: Option<bool>,
    /// Whether trading is blocked.
    pub trading_blocked: bool,
    /// Whether transfers are blocked.
    pub transfers_blocked: bool,
    /// Whether the account is blocked.
    pub account_blocked: bool,
    /// When the account was created.
    pub created_at: DateTime<Utc>,
    /// Whether trading was suspended by the user.
    pub trade_suspended_by_user: bool,
    /// The account multiplier (e.g. `"4"`).
    pub multiplier: String,
    /// Whether shorting is enabled.
    pub shorting_enabled: bool,
    /// Equity value.
    pub equity: Decimal,
    /// Equity as of the previous trading day.
    pub last_equity: Decimal,
    /// Long market value.
    pub long_market_value: Option<Decimal>,
    /// Short market value.
    pub short_market_value: Option<Decimal>,
    /// Initial margin requirement.
    pub initial_margin: Option<Decimal>,
    /// Maintenance margin requirement.
    pub maintenance_margin: Option<Decimal>,
    /// Maintenance margin requirement as of the previous trading day.
    pub last_maintenance_margin: Option<Decimal>,
    /// Special memorandum account value.
    pub sma: Option<Decimal>,
    /// Number of day trades in the current 5-trading-day window. Omitted by
    /// the API since the 2026 PDT-rule sunset.
    pub daytrade_count: Option<i64>,
}

/// Account configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfigurations {
    /// Day trade buying power check.
    pub dtbp_check: DTBPCheck,
    /// Trade confirmation email setting.
    pub trade_confirm_email: TradeConfirmationEmail,
    /// Whether trading is suspended.
    pub suspend_trade: bool,
    /// Whether shorting is disabled.
    pub no_shorting: bool,
    /// Whether fractional trading is enabled.
    pub fractional_trading: bool,
    /// Maximum margin multiplier (e.g. `"4"`).
    pub max_margin_multiplier: String,
    /// Pattern day trader check.
    pub pdt_check: PDTCheck,
    /// Maximum approved options trading level.
    pub max_options_trading_level: Option<u32>,
}

/// Deserializes an optional value where the API emits `""` instead of `null`
/// (observed live: `order_class` on equity orders).
fn empty_string_as_none<'de, D, T>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    match Option::<String>::deserialize(deserializer)? {
        None => Ok(None),
        Some(raw) if raw.is_empty() => Ok(None),
        Some(raw) => {
            T::deserialize(serde::de::value::StrDeserializer::<D::Error>::new(&raw)).map(Some)
        }
    }
}

/// An order.
#[derive(Debug, Clone, Serialize)]
pub struct Order {
    /// Order id.
    pub id: String,
    /// Client-provided order id.
    pub client_order_id: String,
    /// When the order was created.
    pub created_at: Option<DateTime<Utc>>,
    /// When the order was last updated.
    pub updated_at: Option<DateTime<Utc>>,
    /// When the order was submitted.
    pub submitted_at: Option<DateTime<Utc>>,
    /// When the order was filled.
    pub filled_at: Option<DateTime<Utc>>,
    /// When the order expired.
    pub expired_at: Option<DateTime<Utc>>,
    /// When the order was canceled.
    pub canceled_at: Option<DateTime<Utc>>,
    /// When the order failed.
    pub failed_at: Option<DateTime<Utc>>,
    /// When the order was replaced.
    pub replaced_at: Option<DateTime<Utc>>,
    /// Id of the order that replaced this one.
    pub replaced_by: Option<String>,
    /// Id of the order this one replaced.
    pub replaces: Option<String>,
    /// Asset id.
    pub asset_id: String,
    /// Asset symbol.
    pub symbol: String,
    /// Asset class.
    pub asset_class: AssetClass,
    /// Ordered notional amount.
    pub notional: Option<Decimal>,
    /// Ordered quantity.
    pub qty: Option<Decimal>,
    /// Filled quantity.
    pub filled_qty: Option<Decimal>,
    /// Filled average price.
    pub filled_avg_price: Option<Decimal>,
    /// Order class.
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub order_class: Option<OrderClass>,
    /// Order type (accepts both the legacy `type` key and `order_type`;
    /// serialized as `type`).
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order side.
    pub side: OrderSide,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Limit price.
    pub limit_price: Option<Decimal>,
    /// Stop price.
    pub stop_price: Option<Decimal>,
    /// Lifecycle status of the order.
    pub status: OrderStatus,
    /// Whether the order can execute during extended hours.
    pub extended_hours: bool,
    /// Child orders of a multi-leg order.
    pub legs: Option<Vec<Order>>,
    /// Trail percent of a trailing stop order.
    pub trail_percent: Option<Decimal>,
    /// Trail price of a trailing stop order.
    pub trail_price: Option<Decimal>,
    /// High water mark of a trailing stop order.
    pub hwm: Option<String>,
}

/// Wire shape of [`Order`]. Observed live, order payloads carry **both**
/// `type` and `order_type` (with the same value), so serde's `alias` — which
/// errors on duplicate fields — cannot be used; both keys are captured here
/// and merged in [`Order`]'s `Deserialize` impl.
#[derive(Deserialize)]
struct OrderWire {
    id: String,
    client_order_id: String,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    submitted_at: Option<DateTime<Utc>>,
    filled_at: Option<DateTime<Utc>>,
    expired_at: Option<DateTime<Utc>>,
    canceled_at: Option<DateTime<Utc>>,
    failed_at: Option<DateTime<Utc>>,
    replaced_at: Option<DateTime<Utc>>,
    replaced_by: Option<String>,
    replaces: Option<String>,
    asset_id: String,
    symbol: String,
    asset_class: AssetClass,
    notional: Option<Decimal>,
    qty: Option<Decimal>,
    filled_qty: Option<Decimal>,
    filled_avg_price: Option<Decimal>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    order_class: Option<OrderClass>,
    #[serde(rename = "type")]
    legacy_order_type: Option<OrderType>,
    order_type: Option<OrderType>,
    side: OrderSide,
    time_in_force: TimeInForce,
    limit_price: Option<Decimal>,
    stop_price: Option<Decimal>,
    status: OrderStatus,
    extended_hours: bool,
    legs: Option<Vec<Order>>,
    trail_percent: Option<Decimal>,
    trail_price: Option<Decimal>,
    hwm: Option<String>,
}

impl<'de> Deserialize<'de> for Order {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = OrderWire::deserialize(deserializer)?;
        Ok(Order {
            id: wire.id,
            client_order_id: wire.client_order_id,
            created_at: wire.created_at,
            updated_at: wire.updated_at,
            submitted_at: wire.submitted_at,
            filled_at: wire.filled_at,
            expired_at: wire.expired_at,
            canceled_at: wire.canceled_at,
            failed_at: wire.failed_at,
            replaced_at: wire.replaced_at,
            replaced_by: wire.replaced_by,
            replaces: wire.replaces,
            asset_id: wire.asset_id,
            symbol: wire.symbol,
            asset_class: wire.asset_class,
            notional: wire.notional,
            qty: wire.qty,
            filled_qty: wire.filled_qty,
            filled_avg_price: wire.filled_avg_price,
            order_class: wire.order_class,
            order_type: wire
                .legacy_order_type
                .or(wire.order_type)
                .ok_or_else(|| <D::Error as serde::de::Error>::missing_field("type"))?,
            side: wire.side,
            time_in_force: wire.time_in_force,
            limit_price: wire.limit_price,
            stop_price: wire.stop_price,
            status: wire.status,
            extended_hours: wire.extended_hours,
            legs: wire.legs,
            trail_percent: wire.trail_percent,
            trail_price: wire.trail_price,
            hwm: wire.hwm,
        })
    }
}

/// A position in an asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Asset id.
    pub asset_id: String,
    /// Asset symbol.
    pub symbol: String,
    /// Exchange the asset trades on.
    pub exchange: AssetExchange,
    /// Asset class.
    pub asset_class: AssetClass,
    /// Average entry price.
    pub avg_entry_price: Decimal,
    /// Quantity held (negative for short positions).
    pub qty: Decimal,
    /// Position side.
    pub side: PositionSide,
    /// Total market value.
    pub market_value: Option<Decimal>,
    /// Total cost basis.
    pub cost_basis: Option<Decimal>,
    /// Unrealized profit/loss.
    pub unrealized_pl: Option<Decimal>,
    /// Unrealized profit/loss as a fraction of cost basis.
    pub unrealized_plpc: Option<Decimal>,
    /// Unrealized profit/loss for the current day.
    pub unrealized_intraday_pl: Option<Decimal>,
    /// Unrealized intraday profit/loss as a fraction.
    pub unrealized_intraday_plpc: Option<Decimal>,
    /// Current asset price.
    pub current_price: Option<Decimal>,
    /// Previous day's closing price.
    pub lastday_price: Option<Decimal>,
    /// Today's price change as a fraction.
    pub change_today: Option<Decimal>,
    /// Quantity available for trading.
    pub qty_available: Option<Decimal>,
    /// Whether the asset is marginable.
    pub asset_marginable: Option<bool>,
}

/// A tradable asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// Asset id.
    pub id: String,
    /// Asset class (serialized as `class`).
    #[serde(rename = "class")]
    pub asset_class: AssetClass,
    /// Exchange the asset trades on.
    pub exchange: AssetExchange,
    /// Asset symbol.
    pub symbol: String,
    /// Asset name.
    pub name: Option<String>,
    /// Asset status.
    pub status: AssetStatus,
    /// Whether the asset is tradable on Alpaca.
    pub tradable: bool,
    /// Whether the asset is marginable.
    pub marginable: Option<bool>,
    /// Whether the asset is shortable.
    pub shortable: Option<bool>,
    /// Whether the asset is easy to borrow.
    ///
    /// Deprecated by the API (sunset 2026-09-22) in favour of
    /// [`borrow_status`](Self::borrow_status); still returned for now.
    pub easy_to_borrow: Option<bool>,
    /// How hard the asset is to borrow for short selling.
    pub borrow_status: Option<BorrowStatus>,
    /// Whether the asset is fractionable.
    pub fractionable: Option<bool>,
    /// Maintenance margin requirement as a percentage.
    ///
    /// Deprecated by the API in favour of
    /// [`margin_requirement_long`](Self::margin_requirement_long) and
    /// [`margin_requirement_short`](Self::margin_requirement_short); still
    /// returned for now. Note this one arrives as a JSON number, unlike the
    /// replacements.
    pub maintenance_margin_requirement: Option<f64>,
    /// Margin requirement for a long position, as a percentage.
    pub margin_requirement_long: Option<Decimal>,
    /// Margin requirement for a short position, as a percentage.
    pub margin_requirement_short: Option<Decimal>,
    /// Asset attributes (e.g. [`AssetAttribute::PtpWithException`]).
    #[serde(default)]
    pub attributes: Vec<AssetAttribute>,
    /// Minimum order size (fractionable assets).
    pub min_order_size: Option<Decimal>,
    /// Minimum trade increment.
    pub min_trade_increment: Option<Decimal>,
    /// Minimum price increment.
    pub price_increment: Option<Decimal>,
}

/// Market open/close state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clock {
    /// Current timestamp.
    pub timestamp: DateTime<Utc>,
    /// Whether the market is currently open.
    pub is_open: bool,
    /// Next market open.
    pub next_open: DateTime<Utc>,
    /// Next market close.
    pub next_close: DateTime<Utc>,
}

/// A single day in the market calendar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarDay {
    /// The date.
    pub date: NaiveDate,
    /// Session open time (e.g. `"09:30"`).
    pub open: String,
    /// Session close time (e.g. `"16:00"`).
    pub close: String,
    /// Settlement date, if present.
    pub settlement_date: Option<NaiveDate>,
}

/// A market described by the v3 clock and calendar endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketInfo {
    /// Acronym of the market (e.g. [`Market::NYSE`]).
    pub acronym: Market,
    /// Full name of the market (e.g. `"New York Stock Exchange"`).
    pub name: String,
    /// IANA timezone of the market (e.g. `"America/New_York"`).
    pub timezone: String,
    /// ISO 10383 market identifier code (e.g. `"XNYS"`), when the market has
    /// one.
    pub mic: Option<String>,
    /// BIC/SWIFT business identifier code, when the market has one (banking
    /// calendars).
    pub bic: Option<String>,
}

/// The clock of a single market, as returned by `GET /v3/clock`.
///
/// Timestamps keep the offset the API sent them with — session times are in
/// the market's own timezone — so call `.with_timezone(&Utc)` before comparing
/// across markets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockV3 {
    /// The market this clock belongs to.
    pub market: MarketInfo,
    /// The time on the clock.
    pub timestamp: DateTime<FixedOffset>,
    /// Whether the clock is on a market day.
    pub is_market_day: bool,
    /// Next market open.
    pub next_market_open: DateTime<FixedOffset>,
    /// Next market close.
    pub next_market_close: DateTime<FixedOffset>,
    /// The session the market is currently in.
    pub phase: MarketPhase,
    /// The end of the current phase.
    pub phase_until: DateTime<FixedOffset>,
}

/// Response of `GET /v3/clock`: one [`ClockV3`] per requested market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockV3Response {
    /// The per-market clocks.
    pub clocks: Vec<ClockV3>,
}

/// A single day in the v3 market calendar.
///
/// Only `date` and the core session are always present; a market without a
/// pre-market, after-hours or lunch break omits those fields. As with
/// [`ClockV3`], times carry the market's own UTC offset unless the request
/// asked for [`CalendarTimezone::Utc`](super::enums::CalendarTimezone::Utc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarDayV3 {
    /// The date of the calendar day.
    pub date: NaiveDate,
    /// Start of the core (regular) session.
    pub core_start: DateTime<FixedOffset>,
    /// End of the core (regular) session.
    pub core_end: DateTime<FixedOffset>,
    /// Start of the pre-market session.
    pub pre_start: Option<DateTime<FixedOffset>>,
    /// End of the pre-market session.
    pub pre_end: Option<DateTime<FixedOffset>>,
    /// Start of the after-hours session.
    pub post_start: Option<DateTime<FixedOffset>>,
    /// End of the after-hours session.
    pub post_end: Option<DateTime<FixedOffset>>,
    /// Start of the lunch break.
    pub lunch_start: Option<DateTime<FixedOffset>>,
    /// End of the lunch break.
    pub lunch_end: Option<DateTime<FixedOffset>>,
    /// Settlement date of trades made on this day.
    pub settlement_date: Option<NaiveDate>,
}

/// Response of `GET /v3/calendar/{market}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarV3Response {
    /// The market the calendar belongs to.
    pub market: MarketInfo,
    /// The calendar days in the requested range.
    pub calendar: Vec<CalendarDayV3>,
}

/// Portfolio equity history over a period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHistory {
    /// Timestamps of each data point (seconds since epoch).
    #[serde(default)]
    pub timestamp: Vec<i64>,
    /// Equity at each data point (may contain nulls).
    #[serde(default)]
    pub equity: Vec<Option<f64>>,
    /// Profit/loss at each data point.
    #[serde(default)]
    pub profit_loss: Vec<Option<f64>>,
    /// Profit/loss as a fraction at each data point.
    #[serde(default)]
    pub profit_loss_pct: Vec<Option<f64>>,
    /// Base value the profit/loss is computed against.
    pub base_value: Option<f64>,
    /// The date the base value was captured, if returned.
    pub base_value_asof: Option<String>,
    /// Resolution of the data (e.g. `"1D"`).
    pub timeframe: Option<String>,
}

/// A watchlist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchlist {
    /// Watchlist id.
    pub id: String,
    /// Account id the watchlist belongs to.
    pub account_id: String,
    /// Watchlist name.
    pub name: String,
    /// When the watchlist was created.
    pub created_at: Option<DateTime<Utc>>,
    /// When the watchlist was last updated.
    pub updated_at: Option<DateTime<Utc>>,
    /// Assets in the watchlist (only present when fetching a single
    /// watchlist).
    pub assets: Option<Vec<Asset>>,
}

/// A corporate action announcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorporateActionAnnouncement {
    /// Announcement id.
    pub id: String,
    /// Corporate action id.
    pub corporate_action_id: String,
    /// Corporate action type (e.g. `dividend`, `merger`, `split`).
    pub ca_type: String,
    /// Corporate action sub type.
    pub ca_sub_type: Option<String>,
    /// Symbol of the company initiating the action.
    pub initiating_symbol: Option<String>,
    /// Symbol of the target company.
    pub target_symbol: Option<String>,
    /// Record date.
    pub record_date: Option<NaiveDate>,
    /// Ex date.
    pub ex_date: Option<NaiveDate>,
    /// Payable date.
    pub payable_date: Option<NaiveDate>,
    /// Cash amount of the action.
    pub cash: Option<Decimal>,
    /// Old rate of the action.
    pub old_rate: Option<Decimal>,
    /// New rate of the action.
    pub new_rate: Option<Decimal>,
}

/// An option contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionContract {
    /// Contract id.
    pub id: String,
    /// Contract symbol (OCC format).
    pub symbol: String,
    /// Contract name.
    pub name: Option<String>,
    /// Contract status.
    pub status: AssetStatus,
    /// Whether the contract is tradable on Alpaca.
    pub tradable: bool,
    /// Expiration date.
    pub expiration_date: NaiveDate,
    /// Root symbol.
    pub root_symbol: String,
    /// Underlying symbol.
    pub underlying_symbol: String,
    /// Underlying asset id.
    pub underlying_asset_id: String,
    /// Contract type (serialized as `type`).
    #[serde(rename = "type")]
    pub contract_type: ContractType,
    /// Exercise style.
    pub style: ExerciseStyle,
    /// Strike price.
    pub strike_price: Decimal,
    /// Contract multiplier (e.g. `"100"`).
    pub size: Option<String>,
    /// Open interest.
    pub open_interest: Option<String>,
    /// Date of the open interest figure.
    pub open_interest_date: Option<NaiveDate>,
    /// Last close price of the contract.
    pub close_price: Option<String>,
    /// Date of the close price.
    pub close_price_date: Option<NaiveDate>,
}

/// Paginated response of the list option contracts endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionContractsResponse {
    /// The contracts in this page.
    pub option_contracts: Vec<OptionContract>,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

/// One entry of the cancel-all-orders response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderResult {
    /// Order id.
    pub id: Option<String>,
    /// HTTP status of the cancel attempt.
    pub status: Option<u16>,
    /// The canceled order, when the cancel succeeded.
    pub body: Option<Order>,
    /// API error code, when the cancel failed.
    pub code: Option<i64>,
    /// API error message, when the cancel failed.
    pub message: Option<String>,
}

/// One entry of the close-all-positions response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseAllPositionsResult {
    /// Id of the liquidation order.
    pub order_id: Option<String>,
    /// HTTP status of the close attempt.
    pub status: Option<u16>,
    /// Symbol of the position.
    pub symbol: Option<String>,
    /// Response body of the close attempt. Observed live: on success this is
    /// the liquidation **order** object (not a position); on failure it is an
    /// error payload — hence an untyped value.
    pub body: Option<serde_json::Value>,
    /// API error code, when the close failed.
    pub code: Option<i64>,
    /// API error message, when the close failed.
    pub message: Option<String>,
}

/// A trade activity — a fill or partial fill of an order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeActivity {
    /// Unique id of the activity, formatted like
    /// `20220203000000000::045b3b8d-c566-4bef-b741-2bf598dd6ae7` (a date
    /// string followed by a UUID).
    pub id: String,
    /// Id of the account the activity relates to.
    ///
    /// The live `/v2/account/activities` endpoint omits this field entirely —
    /// the account is implied by the credentials — so it is optional.
    #[serde(default)]
    pub account_id: Option<String>,
    /// The kind of activity (e.g. `FILL`).
    pub activity_type: ActivityType,
    /// When the trade was processed.
    pub transaction_time: DateTime<Utc>,
    /// Whether this was a fill or a partial fill (serialized as `type`).
    #[serde(rename = "type")]
    pub trade_type: TradeActivityType,
    /// The per-share execution price.
    pub price: Decimal,
    /// The number of shares involved in the execution.
    pub qty: Decimal,
    /// The side of the trade.
    pub side: OrderSide,
    /// The symbol that was traded.
    pub symbol: String,
    /// Shares left to be filled (`0` unless the order is partially filled).
    pub leaves_qty: Decimal,
    /// Id of the order that was filled.
    pub order_id: String,
    /// Cumulative quantity of shares filled on the order.
    pub cum_qty: Decimal,
    /// Status of the order that executed the trade.
    pub order_status: OrderStatus,
}

/// A non-trade activity — an account activity unrelated to orders or trades
/// (dividends, transfers, fees, ...).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonTradeActivity {
    /// Unique id of the activity, formatted like
    /// `20220203000000000::045b3b8d-c566-4bef-b741-2bf598dd6ae7` (a date
    /// string followed by a UUID).
    pub id: String,
    /// Id of the account the activity relates to.
    ///
    /// The live `/v2/account/activities` endpoint omits this field entirely —
    /// the account is implied by the credentials — so it is optional.
    #[serde(default)]
    pub account_id: Option<String>,
    /// The kind of activity (e.g. `DIV`).
    pub activity_type: ActivityType,
    /// The date the activity occurred or the transaction settled.
    pub date: NaiveDate,
    /// The net amount of money (positive or negative) of the activity.
    pub net_amount: Decimal,
    /// Extra description of the activity (may be empty).
    pub description: String,
    /// Status of the activity (not present for all activity types).
    pub status: Option<NonTradeActivityStatus>,
    /// Symbol of the security involved (not present for all activity types).
    pub symbol: Option<String>,
    /// For dividend activities, the shares that contributed to the payment.
    pub qty: Option<Decimal>,
    /// Price involved in the activity (not present for all activity types).
    pub price: Option<Decimal>,
    /// For dividend activities, the average amount paid per share.
    pub per_share_amount: Option<Decimal>,
}

/// An account activity, as returned by `GET /v2/account/activities`.
///
/// The endpoint returns a flat JSON array mixing both shapes, and the two
/// families share no tag field beyond `activity_type`, so the enum is
/// untagged: the shapes are disjoint (only trade activities carry
/// `transaction_time`/`order_id`/...), which lets serde pick the right
/// variant structurally.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AccountActivity {
    /// A fill or partial fill of an order.
    Trade(TradeActivity),
    /// A non-trade activity (dividend, transfer, fee, ...).
    NonTrade(NonTradeActivity),
}

/// Locate availability and pricing for a single symbol.
///
/// Quotes are advisory: they neither reserve inventory nor guarantee the fee
/// of a later [`Locate`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocateQuote {
    /// Stock symbol.
    pub symbol: String,
    /// Number of shares available to locate.
    pub available_qty: i64,
    /// Locate fee per share in USD; absent when no quantity is available.
    pub price: Option<Decimal>,
    /// When the quote was issued.
    pub quoted_at: DateTime<Utc>,
}

/// A symbol from a locate-quotes request that could not be quoted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocateQuoteError {
    /// The requested symbol that could not be quoted.
    pub symbol: String,
    /// Machine-readable reason the symbol could not be quoted.
    pub code: LocateQuoteErrorCode,
    /// Human-readable error message.
    pub message: String,
}

/// Response of `GET /v1/locates/quotes`.
///
/// Symbols are split between the two lists: every requested symbol appears
/// either in `quotes` or in `errors`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocateQuotesResponse {
    /// Quotes for the symbols that could be quoted.
    pub quotes: Vec<LocateQuote>,
    /// Symbols that could not be quoted (empty when all succeeded).
    #[serde(default)]
    pub errors: Vec<LocateQuoteError>,
}

/// A short-sale locate request and its current lifecycle status.
///
/// A rejected locate carries only `rejection_reason`; the `located_*`,
/// `total_fee` and `expires_at` fields are absent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Locate {
    /// Locate id.
    pub id: String,
    /// Stock symbol.
    pub symbol: String,
    /// Number of shares requested.
    pub requested_qty: i64,
    /// Whether the request required the full quantity.
    pub all_or_none: bool,
    /// Lifecycle status of the locate.
    pub status: LocateStatus,
    /// When the locate was created.
    pub created_at: DateTime<Utc>,
    /// Number of shares actually located; absent when rejected.
    pub located_qty: Option<i64>,
    /// Locate fee per share in USD; absent when rejected.
    pub located_price: Option<Decimal>,
    /// Total (non-refundable) locate fee in USD; absent when rejected.
    pub total_fee: Option<Decimal>,
    /// Maximum acceptable fee per share carried over from the request.
    pub limit_price: Option<Decimal>,
    /// When an active locate expires; absent when rejected.
    pub expires_at: Option<DateTime<Utc>>,
    /// Machine-readable rejection reason (e.g. `"inventory_unavailable"`),
    /// present only when rejected.
    pub rejection_reason: Option<String>,
}

/// Paginated response of `GET /v1/locates`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocatesResponse {
    /// The locates in this page, newest first.
    pub locates: Vec<Locate>,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_order() {
        let json = r#"{
            "id": "61e69015-8549-4bfd-b9c3-01e75843f47d",
            "client_order_id": "eb9e2aaa-f71a-4f51-b5b4-52a6c565dad4",
            "created_at": "2024-07-24T07:56:53.123456789Z",
            "updated_at": "2024-07-24T07:56:53.123456789Z",
            "submitted_at": "2024-07-24T07:56:53.123456789Z",
            "filled_at": null,
            "expired_at": null,
            "canceled_at": null,
            "failed_at": null,
            "replaced_at": null,
            "replaced_by": null,
            "replaces": null,
            "asset_id": "b0b6dd9d-8b9b-48a9-ba46-b9d54906e415",
            "symbol": "AAPL",
            "asset_class": "us_equity",
            "notional": null,
            "qty": "1",
            "filled_qty": "0",
            "filled_avg_price": null,
            "order_class": "simple",
            "type": "market",
            "side": "buy",
            "time_in_force": "day",
            "limit_price": null,
            "stop_price": null,
            "status": "accepted",
            "extended_hours": false,
            "legs": null,
            "trail_percent": null,
            "trail_price": null,
            "hwm": null
        }"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.symbol, "AAPL");
        assert_eq!(order.qty, Some(Decimal::new(1, 0)));
        assert_eq!(order.filled_qty, Some(Decimal::ZERO));
        assert_eq!(order.filled_avg_price, None);
        assert_eq!(order.status, OrderStatus::Accepted);
        assert_eq!(order.order_type, OrderType::Market);
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.order_class, Some(OrderClass::Simple));
    }

    #[test]
    fn deserialize_order_with_empty_order_class() {
        // Observed on the live wire: equity orders carry `order_class: ""`
        // and both `type` and `order_type` keys with the same value.
        let json = r#"{
            "id": "61e69015-8549-4bfd-b9c3-01e75843f47d",
            "client_order_id": "eb9e2aaa-f71a-4f51-b5b4-52a6c565dad4",
            "created_at": "2024-07-24T07:56:53.123456789Z",
            "updated_at": "2024-07-24T07:56:53.123456789Z",
            "submitted_at": "2024-07-24T07:56:53.123456789Z",
            "filled_at": null,
            "expired_at": null,
            "canceled_at": null,
            "failed_at": null,
            "replaced_at": null,
            "replaced_by": null,
            "replaces": null,
            "asset_id": "b0b6dd9d-8b9b-48a9-ba46-b9d54906e415",
            "symbol": "AAPL",
            "asset_class": "us_equity",
            "notional": null,
            "qty": "1",
            "filled_qty": "0",
            "filled_avg_price": null,
            "order_class": "",
            "type": "market",
            "order_type": "market",
            "side": "buy",
            "time_in_force": "day",
            "limit_price": null,
            "stop_price": null,
            "status": "accepted",
            "extended_hours": false,
            "legs": null,
            "trail_percent": null,
            "trail_price": null,
            "hwm": null
        }"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.order_class, None);
        assert_eq!(order.order_type, OrderType::Market);
        assert_eq!(order.symbol, "AAPL");
    }

    #[test]
    fn deserialize_order_with_only_order_type_key() {
        // The `order_type` key alone (no legacy `type`) must also parse.
        let json = r#"{
            "id": "61e69015-8549-4bfd-b9c3-01e75843f47d",
            "client_order_id": "eb9e2aaa-f71a-4f51-b5b4-52a6c565dad4",
            "created_at": "2024-07-24T07:56:53.123456789Z",
            "updated_at": null,
            "submitted_at": null,
            "filled_at": null,
            "expired_at": null,
            "canceled_at": null,
            "failed_at": null,
            "replaced_at": null,
            "replaced_by": null,
            "replaces": null,
            "asset_id": "b0b6dd9d-8b9b-48a9-ba46-b9d54906e415",
            "symbol": "AAPL",
            "asset_class": "us_equity",
            "notional": null,
            "qty": "1",
            "filled_qty": "0",
            "filled_avg_price": null,
            "order_class": null,
            "order_type": "limit",
            "side": "sell",
            "time_in_force": "gtc",
            "limit_price": "150.0",
            "stop_price": null,
            "status": "new",
            "extended_hours": false,
            "legs": null,
            "trail_percent": null,
            "trail_price": null,
            "hwm": null
        }"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.order_type, OrderType::Limit);
    }

    #[test]
    fn deserialize_position() {
        let json = r#"{
            "asset_id": "904837e3-3b76-47ec-b432-046443621571",
            "symbol": "AAPL",
            "exchange": "NASDAQ",
            "asset_class": "us_equity",
            "asset_marginable": true,
            "qty": "5",
            "avg_entry_price": "100.0",
            "side": "long",
            "market_value": "600.0",
            "cost_basis": "500.0",
            "unrealized_pl": "100.0",
            "unrealized_plpc": "0.20",
            "unrealized_intraday_pl": "10.0",
            "unrealized_intraday_plpc": "0.0169",
            "current_price": "120.0",
            "lastday_price": "119.0",
            "change_today": "0.0084",
            "qty_available": "5"
        }"#;
        let position: Position = serde_json::from_str(json).unwrap();
        assert_eq!(position.qty, Decimal::new(5, 0));
        assert_eq!(position.side, PositionSide::Long);
        assert_eq!(position.exchange, AssetExchange::NASDAQ);
        assert_eq!(position.unrealized_plpc, Some(Decimal::new(20, 2)));
    }

    #[test]
    fn deserialize_asset_with_refreshed_fields() {
        let json = r#"{
            "id": "b0b6dd9d-8b9b-48a9-ba46-b9d54906e415",
            "class": "us_equity",
            "exchange": "NASDAQ",
            "symbol": "AAPL",
            "name": "Apple Inc. Common Stock",
            "status": "active",
            "tradable": true,
            "marginable": true,
            "shortable": true,
            "easy_to_borrow": true,
            "borrow_status": "easy_to_borrow",
            "fractionable": true,
            "maintenance_margin_requirement": 30,
            "margin_requirement_long": "30",
            "margin_requirement_short": "30",
            "attributes": ["has_options", "options_late_close", "warp_drive_enabled"]
        }"#;
        let asset: Asset = serde_json::from_str(json).unwrap();
        assert_eq!(asset.symbol, "AAPL");
        assert_eq!(asset.borrow_status, Some(BorrowStatus::EasyToBorrow));
        // The replacement margin fields arrive as JSON strings, the
        // deprecated one as a JSON number.
        assert_eq!(asset.margin_requirement_long, Some(Decimal::new(30, 0)));
        assert_eq!(asset.margin_requirement_short, Some(Decimal::new(30, 0)));
        assert_eq!(asset.maintenance_margin_requirement, Some(30.0));
        assert_eq!(asset.easy_to_borrow, Some(true));
        // An unknown attribute is preserved rather than rejected.
        assert_eq!(
            asset.attributes,
            vec![
                AssetAttribute::HasOptions,
                AssetAttribute::OptionsLateClose,
                AssetAttribute::Other("warp_drive_enabled".into()),
            ]
        );

        // Round-trip.
        let reparsed: Asset =
            serde_json::from_str(&serde_json::to_string(&asset).unwrap()).unwrap();
        assert_eq!(reparsed.attributes, asset.attributes);
        assert_eq!(reparsed.borrow_status, asset.borrow_status);
        assert_eq!(
            reparsed.margin_requirement_long,
            asset.margin_requirement_long
        );
    }

    #[test]
    fn deserialize_asset_without_optional_fields() {
        // Crypto assets omit the equity-only fields entirely.
        let json = r#"{
            "id": "276e2673-764b-4ab6-a611-caf665ca6340",
            "class": "crypto",
            "exchange": "CRYPTO",
            "symbol": "BTC/USD",
            "name": "Bitcoin / US Dollar",
            "status": "active",
            "tradable": true,
            "min_order_size": "0.0001",
            "min_trade_increment": "0.0001",
            "price_increment": "1"
        }"#;
        let asset: Asset = serde_json::from_str(json).unwrap();
        assert_eq!(asset.borrow_status, None);
        assert_eq!(asset.margin_requirement_long, None);
        assert!(asset.attributes.is_empty());
        assert_eq!(asset.min_order_size, Some(Decimal::new(1, 4)));
    }

    const FILL_ACTIVITY_JSON: &str = r#"{
        "id": "20220307115529661::4f8cb7a9-cc9b-46ea-bb48-a80ec1d5b4f8",
        "account_id": "5c56a945-89a0-4c5e-9d1f-2b3c4d5e6f70",
        "activity_type": "FILL",
        "transaction_time": "2022-03-07T16:55:29.661Z",
        "type": "fill",
        "price": "127.81",
        "qty": "1",
        "side": "buy",
        "symbol": "AAPL",
        "leaves_qty": "0",
        "order_id": "61e69015-8549-4bfd-b9c3-01e75843f47d",
        "cum_qty": "1",
        "order_status": "filled"
    }"#;

    const DIV_ACTIVITY_JSON: &str = r#"{
        "id": "20220307000000000::045b3b8d-c566-4bef-b741-2bf598dd6ae7",
        "account_id": "5c56a945-89a0-4c5e-9d1f-2b3c4d5e6f70",
        "activity_type": "DIV",
        "date": "2022-03-07",
        "net_amount": "1.12",
        "description": "",
        "status": "executed",
        "symbol": "AAPL",
        "qty": "2",
        "price": "150.5",
        "per_share_amount": "0.56"
    }"#;

    /// Both activity families exactly as the live `/v2/account/activities`
    /// endpoint returns them: no `account_id` on either, and the non-trade
    /// item carrying `activity_sub_type`/`created_at`/`currency`.
    ///
    /// Regression test — the models required `account_id`, so every real
    /// response failed to deserialize while the mocked tests (whose fixtures
    /// invented the field) passed.
    #[test]
    fn deserialize_live_activities_without_account_id() {
        const LIVE_JSON: &str = r#"[
            {"activity_type":"FILL","cum_qty":"1","id":"20220203000000000::045b3b8d-c566-4bef-b741-2bf598dd6ae7",
             "leaves_qty":"0","order_id":"6f2b9b1a-1111-2222-3333-444455556666","order_status":"filled",
             "price":"127.81","qty":"1","side":"sell_short","swap_rate":"1","symbol":"AAPL",
             "transaction_time":"2022-03-07T16:55:29.661Z","type":"fill"},
            {"activity_sub_type":"CDIV","activity_type":"DIV","created_at":"2026-03-20T15:42:11.118274Z",
             "currency":"USD","date":"2026-03-20","description":"Cash DIV @ 0.07",
             "id":"20260320000000000::aaaa1111-bbbb-2222-cccc-333344445555",
             "net_amount":"0.07","status":"executed"}
        ]"#;

        let activities: Vec<AccountActivity> = serde_json::from_str(LIVE_JSON).unwrap();
        assert_eq!(activities.len(), 2);

        let AccountActivity::Trade(trade) = &activities[0] else {
            panic!("expected a trade activity, got {:?}", activities[0]);
        };
        assert_eq!(trade.account_id, None);
        assert_eq!(trade.symbol, "AAPL");
        assert_eq!(trade.activity_type, ActivityType::Fill);
        // Short-sale executions report a side the orders API never accepts.
        assert_eq!(trade.side, OrderSide::SellShort);

        let AccountActivity::NonTrade(non_trade) = &activities[1] else {
            panic!("expected a non-trade activity, got {:?}", activities[1]);
        };
        assert_eq!(non_trade.account_id, None);
        assert_eq!(non_trade.activity_type, ActivityType::Div);
        assert_eq!(non_trade.net_amount, Decimal::new(7, 2));
    }

    #[test]
    fn deserialize_trade_activity() {
        let activity: TradeActivity = serde_json::from_str(FILL_ACTIVITY_JSON).unwrap();
        assert_eq!(activity.activity_type, ActivityType::Fill);
        assert_eq!(activity.trade_type, TradeActivityType::Fill);
        assert_eq!(activity.price, Decimal::new(12_781, 2));
        assert_eq!(activity.qty, Decimal::new(1, 0));
        assert_eq!(activity.side, OrderSide::Buy);
        assert_eq!(activity.symbol, "AAPL");
        assert_eq!(activity.leaves_qty, Decimal::ZERO);
        assert_eq!(activity.cum_qty, Decimal::new(1, 0));
        assert_eq!(activity.order_status, OrderStatus::Filled);
        assert_eq!(
            activity.transaction_time.to_rfc3339(),
            "2022-03-07T16:55:29.661+00:00"
        );

        // Round-trip.
        let json = serde_json::to_string(&activity).unwrap();
        let reparsed: TradeActivity = serde_json::from_str(&json).unwrap();
        assert_eq!(reparsed.id, activity.id);
        assert_eq!(reparsed.price, activity.price);
    }

    #[test]
    fn deserialize_non_trade_activity() {
        let activity: NonTradeActivity = serde_json::from_str(DIV_ACTIVITY_JSON).unwrap();
        assert_eq!(activity.activity_type, ActivityType::Div);
        assert_eq!(activity.date, NaiveDate::from_ymd_opt(2022, 3, 7).unwrap());
        assert_eq!(activity.net_amount, Decimal::new(112, 2));
        assert_eq!(activity.status, Some(NonTradeActivityStatus::Executed));
        assert_eq!(activity.symbol.as_deref(), Some("AAPL"));
        assert_eq!(activity.qty, Some(Decimal::new(2, 0)));
        assert_eq!(activity.per_share_amount, Some(Decimal::new(56, 2)));

        // Round-trip.
        let json = serde_json::to_string(&activity).unwrap();
        let reparsed: NonTradeActivity = serde_json::from_str(&json).unwrap();
        assert_eq!(reparsed.id, activity.id);
        assert_eq!(reparsed.net_amount, activity.net_amount);
    }

    /// Two markets on the same instant: NYSE mid-session, and the overnight
    /// venue BOATS whose core session runs through the night.
    const CLOCK_V3_JSON: &str = r#"{
        "clocks": [
            {
                "market": {
                    "acronym": "NYSE",
                    "name": "New York Stock Exchange",
                    "timezone": "America/New_York",
                    "mic": "XNYS"
                },
                "timestamp": "2025-06-24T14:15:22-04:00",
                "is_market_day": true,
                "next_market_open": "2025-06-25T09:30:00-04:00",
                "next_market_close": "2025-06-24T16:00:00-04:00",
                "phase": "core",
                "phase_until": "2025-06-24T16:00:00-04:00"
            },
            {
                "market": {
                    "acronym": "BOATS",
                    "name": "Blue Ocean Alternative Trading System",
                    "timezone": "America/New_York"
                },
                "timestamp": "2025-06-24T14:15:22-04:00",
                "is_market_day": true,
                "next_market_open": "2025-06-24T20:00:00-04:00",
                "next_market_close": "2025-06-25T04:00:00-04:00",
                "phase": "closed",
                "phase_until": "2025-06-24T20:00:00-04:00"
            }
        ]
    }"#;

    #[test]
    fn deserialize_clock_v3_response() {
        let clock: ClockV3Response = serde_json::from_str(CLOCK_V3_JSON).unwrap();
        assert_eq!(clock.clocks.len(), 2);

        let nyse = &clock.clocks[0];
        assert_eq!(nyse.market.acronym, Market::NYSE);
        assert_eq!(nyse.market.name, "New York Stock Exchange");
        assert_eq!(nyse.market.timezone, "America/New_York");
        assert_eq!(nyse.market.mic.as_deref(), Some("XNYS"));
        assert_eq!(nyse.market.bic, None);
        assert!(nyse.is_market_day);
        assert_eq!(nyse.phase, MarketPhase::Core);
        // The market's own offset is preserved rather than normalized to UTC.
        assert_eq!(nyse.phase_until.to_rfc3339(), "2025-06-24T16:00:00-04:00");
        assert_eq!(
            nyse.timestamp.with_timezone(&Utc).to_rfc3339(),
            "2025-06-24T18:15:22+00:00"
        );

        // The overnight venue is closed during the regular session and opens
        // again in the evening.
        let boats = &clock.clocks[1];
        assert_eq!(boats.market.acronym, Market::BOATS);
        assert_eq!(boats.market.mic, None);
        assert_eq!(boats.phase, MarketPhase::Closed);
        assert_eq!(
            boats.next_market_open.to_rfc3339(),
            "2025-06-24T20:00:00-04:00"
        );
        assert_eq!(
            boats.next_market_close.to_rfc3339(),
            "2025-06-25T04:00:00-04:00"
        );

        // Round-trip.
        let json = serde_json::to_string(&clock).unwrap();
        let reparsed: ClockV3Response = serde_json::from_str(&json).unwrap();
        assert_eq!(reparsed.clocks.len(), 2);
        assert_eq!(reparsed.clocks[1].market.acronym, Market::BOATS);
        assert_eq!(reparsed.clocks[1].phase, MarketPhase::Closed);
        assert_eq!(reparsed.clocks[0].phase_until, clock.clocks[0].phase_until);
    }

    #[test]
    fn deserialize_clock_v3_with_unknown_market_and_phase() {
        let json = r#"{
            "clocks": [{
                "market": {
                    "acronym": "XPHL",
                    "name": "Philadelphia Stock Exchange",
                    "timezone": "America/New_York"
                },
                "timestamp": "2025-06-24T02:15:22-04:00",
                "is_market_day": true,
                "next_market_open": "2025-06-24T09:30:00-04:00",
                "next_market_close": "2025-06-24T16:00:00-04:00",
                "phase": "overnight",
                "phase_until": "2025-06-24T04:00:00-04:00"
            }]
        }"#;
        let clock: ClockV3Response = serde_json::from_str(json).unwrap();
        let entry = &clock.clocks[0];
        assert_eq!(entry.market.acronym, Market::Other("XPHL".into()));
        assert_eq!(entry.phase, MarketPhase::Other("overnight".into()));
    }

    #[test]
    fn deserialize_calendar_v3_response() {
        let json = r#"{
            "market": {
                "acronym": "NYSE",
                "name": "New York Stock Exchange",
                "timezone": "America/New_York",
                "mic": "XNYS"
            },
            "calendar": [{
                "date": "2025-01-02",
                "pre_start": "2025-01-02T04:00:00-05:00",
                "pre_end": "2025-01-02T09:30:00-05:00",
                "core_start": "2025-01-02T09:30:00-05:00",
                "core_end": "2025-01-02T16:00:00-05:00",
                "post_start": "2025-01-02T16:00:00-05:00",
                "post_end": "2025-01-02T20:00:00-05:00",
                "settlement_date": "2025-01-03"
            }]
        }"#;
        let calendar: CalendarV3Response = serde_json::from_str(json).unwrap();
        assert_eq!(calendar.market.acronym, Market::NYSE);
        assert_eq!(calendar.calendar.len(), 1);

        let day = &calendar.calendar[0];
        assert_eq!(day.date, NaiveDate::from_ymd_opt(2025, 1, 2).unwrap());
        assert_eq!(day.core_start.to_rfc3339(), "2025-01-02T09:30:00-05:00");
        assert_eq!(day.core_end.to_rfc3339(), "2025-01-02T16:00:00-05:00");
        assert_eq!(
            day.pre_start.map(|t| t.to_rfc3339()).as_deref(),
            Some("2025-01-02T04:00:00-05:00")
        );
        assert_eq!(
            day.post_end.map(|t| t.to_rfc3339()).as_deref(),
            Some("2025-01-02T20:00:00-05:00")
        );
        // A US venue has no lunch break.
        assert_eq!(day.lunch_start, None);
        assert_eq!(day.lunch_end, None);
        assert_eq!(
            day.settlement_date,
            Some(NaiveDate::from_ymd_opt(2025, 1, 3).unwrap())
        );

        // Round-trip.
        let json = serde_json::to_string(&calendar).unwrap();
        let reparsed: CalendarV3Response = serde_json::from_str(&json).unwrap();
        assert_eq!(reparsed.calendar[0].date, day.date);
        assert_eq!(reparsed.calendar[0].core_start, day.core_start);
        assert_eq!(reparsed.calendar[0].lunch_end, None);
    }

    #[test]
    fn deserialize_calendar_v3_day_with_only_core_session() {
        // An overnight venue reports its whole session as the core one, with
        // no pre/post sessions and no settlement date.
        let json = r#"{
            "market": {
                "acronym": "BOATS",
                "name": "Blue Ocean Alternative Trading System",
                "timezone": "America/New_York"
            },
            "calendar": [{
                "date": "2025-01-02",
                "core_start": "2025-01-01T20:00:00-05:00",
                "core_end": "2025-01-02T04:00:00-05:00"
            }]
        }"#;
        let calendar: CalendarV3Response = serde_json::from_str(json).unwrap();
        let day = &calendar.calendar[0];
        assert_eq!(calendar.market.acronym, Market::BOATS);
        assert_eq!(day.core_start.to_rfc3339(), "2025-01-01T20:00:00-05:00");
        assert_eq!(day.pre_start, None);
        assert_eq!(day.post_end, None);
        assert_eq!(day.settlement_date, None);
    }

    #[test]
    fn deserialize_mixed_activity_array() {
        let json = format!("[{FILL_ACTIVITY_JSON},{DIV_ACTIVITY_JSON}]");
        let activities: Vec<AccountActivity> = serde_json::from_str(&json).unwrap();
        assert_eq!(activities.len(), 2);
        match &activities[0] {
            AccountActivity::Trade(t) => assert_eq!(t.symbol, "AAPL"),
            other => panic!("expected trade activity, got {other:?}"),
        }
        match &activities[1] {
            AccountActivity::NonTrade(n) => assert_eq!(n.activity_type, ActivityType::Div),
            other => panic!("expected non-trade activity, got {other:?}"),
        }
    }

    const ACTIVE_LOCATE_JSON: &str = r#"{
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "symbol": "TSLA",
        "requested_qty": 100,
        "all_or_none": false,
        "status": "active",
        "created_at": "2026-01-02T15:04:05Z",
        "located_qty": 100,
        "located_price": "0.05",
        "total_fee": "5.00",
        "limit_price": "0.05",
        "expires_at": "2026-01-03T01:00:00Z"
    }"#;

    #[test]
    fn deserialize_active_locate() {
        let locate: Locate = serde_json::from_str(ACTIVE_LOCATE_JSON).unwrap();
        assert_eq!(locate.symbol, "TSLA");
        assert_eq!(locate.requested_qty, 100);
        assert_eq!(locate.located_qty, Some(100));
        assert_eq!(locate.status, LocateStatus::Active);
        assert!(!locate.all_or_none);
        // Locate money fields are string-encoded like every trading number.
        assert_eq!(locate.located_price, Some(Decimal::new(5, 2)));
        assert_eq!(locate.total_fee, Some(Decimal::new(500, 2)));
        assert_eq!(locate.limit_price, Some(Decimal::new(5, 2)));
        assert_eq!(
            locate.expires_at.map(|t| t.to_rfc3339()),
            Some("2026-01-03T01:00:00+00:00".to_string())
        );
        assert_eq!(locate.rejection_reason, None);

        let json = serde_json::to_string(&locate).unwrap();
        let reparsed: Locate = serde_json::from_str(&json).unwrap();
        assert_eq!(reparsed.id, locate.id);
        assert_eq!(reparsed.total_fee, locate.total_fee);
    }

    #[test]
    fn deserialize_rejected_locate_without_fill_fields() {
        let json = r#"{
            "id": "0f2b0b26-4b0e-4d3e-9c1e-6f2ff6a9f0c1",
            "symbol": "GME",
            "requested_qty": 500,
            "all_or_none": true,
            "status": "rejected",
            "created_at": "2026-01-02T15:04:05Z",
            "rejection_reason": "inventory_unavailable"
        }"#;
        let locate: Locate = serde_json::from_str(json).unwrap();
        assert_eq!(locate.status, LocateStatus::Rejected);
        assert!(locate.all_or_none);
        assert_eq!(locate.located_qty, None);
        assert_eq!(locate.located_price, None);
        assert_eq!(locate.total_fee, None);
        assert_eq!(locate.expires_at, None);
        assert_eq!(
            locate.rejection_reason.as_deref(),
            Some("inventory_unavailable")
        );
    }

    #[test]
    fn deserialize_locates_page() {
        let json = format!(
            r#"{{"locates": [{ACTIVE_LOCATE_JSON}], "next_page_token": "eyJpZCI6IjU1MGU4NDAwIn0K"}}"#
        );
        let page: LocatesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(page.locates.len(), 1);
        assert_eq!(
            page.next_page_token.as_deref(),
            Some("eyJpZCI6IjU1MGU4NDAwIn0K")
        );

        // A last page carries a null token.
        let json = format!(r#"{{"locates": [{ACTIVE_LOCATE_JSON}], "next_page_token": null}}"#);
        let page: LocatesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(page.next_page_token, None);
    }

    #[test]
    fn deserialize_locate_quotes_with_errors() {
        let json = r#"{
            "quotes": [
                {
                    "symbol": "TSLA",
                    "available_qty": 1000,
                    "price": "0.0123",
                    "quoted_at": "2026-01-02T15:04:05Z"
                },
                {
                    "symbol": "GME",
                    "available_qty": 0,
                    "quoted_at": "2026-01-02T15:04:05Z"
                }
            ],
            "errors": [
                {"symbol": "AAPL", "code": "easy_to_borrow", "message": "symbol is easy to borrow"},
                {"symbol": "XYZ", "code": "market_closed", "message": "locates are closed"}
            ]
        }"#;
        let quotes: LocateQuotesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(quotes.quotes.len(), 2);
        assert_eq!(quotes.quotes[0].available_qty, 1000);
        assert_eq!(quotes.quotes[0].price, Some(Decimal::new(123, 4)));
        // No availability means no price is quoted.
        assert_eq!(quotes.quotes[1].available_qty, 0);
        assert_eq!(quotes.quotes[1].price, None);

        assert_eq!(quotes.errors[0].code, LocateQuoteErrorCode::EasyToBorrow);
        assert_eq!(quotes.errors[0].symbol, "AAPL");
        // A code added after this crate was released still parses.
        assert_eq!(
            quotes.errors[1].code,
            LocateQuoteErrorCode::Other("market_closed".into())
        );
    }

    #[test]
    fn deserialize_locate_quotes_without_errors_field() {
        let json = r#"{"quotes": []}"#;
        let quotes: LocateQuotesResponse = serde_json::from_str(json).unwrap();
        assert!(quotes.quotes.is_empty());
        assert!(quotes.errors.is_empty());
    }
}
