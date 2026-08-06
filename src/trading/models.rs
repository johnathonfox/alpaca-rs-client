//! Response models for the trading API.
//!
//! The trading API serializes all money and quantity numbers as JSON strings
//! (e.g. `"filled_avg_price": "105.89"`), so numeric fields use
//! [`Decimal`] (with the `serde-str` feature, `Decimal` (de)serializes as a
//! string by default).

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::enums::{
    ActivityType, AssetClass, AssetExchange, AssetStatus, ContractType, DTBPCheck, ExerciseStyle,
    NonTradeActivityStatus, OrderClass, OrderSide, OrderStatus, OrderType, PDTCheck, PositionSide,
    TimeInForce, TradeActivityType, TradeConfirmationEmail,
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

/// An order.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub order_class: Option<OrderClass>,
    /// Order type (accepts both the legacy `type` key and `order_type`).
    #[serde(rename = "type", alias = "order_type")]
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
    pub easy_to_borrow: Option<bool>,
    /// Whether the asset is fractionable.
    pub fractionable: Option<bool>,
    /// Maintenance margin requirement as a percentage.
    pub maintenance_margin_requirement: Option<f64>,
    /// Asset attributes (e.g. `ptp_with_exception`).
    #[serde(default)]
    pub attributes: Vec<String>,
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
    /// The closed position, when the close succeeded.
    pub body: Option<Position>,
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
    pub account_id: String,
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
    pub account_id: String,
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
}
