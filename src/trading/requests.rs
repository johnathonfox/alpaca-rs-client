//! Request bodies and query parameters for the trading API.
//!
//! Query parameter structs derive [`Serialize`]; `None` fields are skipped.
//! Multi-value fields such as `symbols` are comma-separated strings.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Serialize, Serializer};

use super::enums::{
    ActivityType, AssetClass, AssetExchange, AssetStatus, ContractType, DTBPCheck, ExerciseStyle,
    OrderClass, OrderSide, OrderType, PDTCheck, PositionIntent, QueryOrderStatus, TimeInForce,
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
}
