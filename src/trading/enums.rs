//! Enums used by the trading API.

use serde::{Deserialize, Serialize};

/// The side of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderSide {
    /// Buy.
    Buy,
    /// Sell.
    Sell,
}

/// The type of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    /// Market order.
    Market,
    /// Limit order.
    Limit,
    /// Stop order.
    Stop,
    /// Stop-limit order.
    StopLimit,
    /// Trailing stop order.
    TrailingStop,
}

/// Time in force of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeInForce {
    /// Day order.
    Day,
    /// Good til canceled.
    Gtc,
    /// Market on open.
    Opg,
    /// Market on close.
    Cls,
    /// Immediate or cancel.
    Ioc,
    /// Fill or kill.
    Fok,
}

/// The lifecycle status of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// The order has been received by Alpaca, but hasn't been routed yet.
    New,
    /// The order has been partially filled.
    PartiallyFilled,
    /// The order has been filled.
    Filled,
    /// The order is done executing for the day.
    DoneForDay,
    /// The order has been canceled.
    Canceled,
    /// The order has expired.
    Expired,
    /// The order was replaced by another order.
    Replaced,
    /// The order is waiting to be canceled.
    PendingCancel,
    /// The order is waiting to be replaced.
    PendingReplace,
    /// The order is waiting to be reviewed.
    PendingReview,
    /// The order has been accepted.
    Accepted,
    /// The order is waiting to become active.
    PendingNew,
    /// The order has been accepted for bidding.
    AcceptedForBidding,
    /// The order has been stopped.
    Stopped,
    /// The order has been rejected.
    Rejected,
    /// The order has been suspended.
    Suspended,
    /// The order has been calculated.
    Calculated,
    /// The order is held.
    Held,
}

/// The class of an order (single leg or multi-leg strategies).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderClass {
    /// A simple, single-leg order.
    Simple,
    /// A bracket order.
    Bracket,
    /// One-cancels-other.
    Oco,
    /// One-triggers-other.
    Oto,
    /// Multi-leg (options).
    Mleg,
}

/// The side of a position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionSide {
    /// Short position.
    Short,
    /// Long position.
    Long,
}

/// The intent of an option position order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionIntent {
    /// Buy to open.
    BuyToOpen,
    /// Buy to close.
    BuyToClose,
    /// Sell to open.
    SellToOpen,
    /// Sell to close.
    SellToClose,
}

/// The class of an asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetClass {
    /// US equity.
    UsEquity,
    /// US option.
    UsOption,
    /// Crypto.
    Crypto,
}

/// The status of an asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    /// Active.
    Active,
    /// Inactive.
    Inactive,
}

/// An exchange an asset trades on. Names are serialized as-is
/// (e.g. `"NASDAQ"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetExchange {
    /// NYSE American (AMEX).
    AMEX,
    /// NYSE Arca.
    ARCA,
    /// Cboe BZX (BATS).
    BATS,
    /// New York Stock Exchange.
    NYSE,
    /// NASDAQ.
    NASDAQ,
    /// NYSE Arca (legacy code).
    NYSEARCA,
    /// NYSE American (legacy code).
    NYSEAMERICAN,
    /// Over the counter.
    OTC,
    /// Crypto exchange placeholder used by Alpaca.
    CRYPTO,
}

/// Filter for the list-orders endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryOrderStatus {
    /// Only open orders.
    Open,
    /// Only closed orders.
    Closed,
    /// All orders.
    All,
}

/// The type of an option contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractType {
    /// Call option.
    Call,
    /// Put option.
    Put,
}

/// The exercise style of an option contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExerciseStyle {
    /// American style.
    American,
    /// European style.
    European,
}

/// Day trade buying power check configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DTBPCheck {
    /// Check on both entry and exit.
    Both,
    /// Check on entry only.
    Entry,
    /// Check on exit only.
    Exit,
}

/// Pattern day trader check configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PDTCheck {
    /// Check on both entry and exit.
    Both,
    /// Check on entry only.
    Entry,
    /// Check on exit only.
    Exit,
}

/// Trade confirmation email configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeConfirmationEmail {
    /// Send emails for all trades.
    All,
    /// Send no emails.
    None,
}

/// The kind of an account activity (serialized screaming-case, e.g. `"FILL"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityType {
    /// Order fills (both partial and full fills).
    Fill,
    /// Cash transactions (both CSD and CSW).
    Trans,
    /// Miscellaneous or rarely used activity types.
    Misc,
    /// ACATS IN/OUT (cash).
    Acatc,
    /// ACATS IN/OUT (securities).
    Acats,
    /// Crypto fee.
    Cfee,
    /// Capital gain distribution.
    Cgd,
    /// Cash deposit(+).
    Csd,
    /// Cash withdrawal(-).
    Csw,
    /// Dividends.
    Div,
    /// Dividend (capital gain long term).
    Divcgl,
    /// Dividend (capital gain short term).
    Divcgs,
    /// Dividend fee.
    Divfee,
    /// Dividend adjusted (foreign tax withheld).
    Divft,
    /// Dividend adjusted (NRA withheld).
    Divnra,
    /// Dividend return of capital.
    Divroc,
    /// Dividend adjusted (Tefra withheld).
    Divtw,
    /// Dividend (tax exempt).
    Divtxex,
    /// Fee denominated in USD.
    Fee,
    /// Interest (credit/margin).
    Int,
    /// Interest adjusted (NRA withheld).
    Intnra,
    /// Interest adjusted (Tefra withheld).
    Inttw,
    /// Journal entry.
    Jnl,
    /// Journal entry (cash).
    Jnlc,
    /// Journal entry (stock).
    Jnls,
    /// Merger/acquisition.
    Ma,
    /// Name change.
    Nc,
    /// Option assignment.
    Opasn,
    /// Option corporate action.
    Opca,
    /// Option cash deliverable for non-standard contracts.
    Opcsh,
    /// Option exercise.
    Opexc,
    /// Option expiration.
    Opexp,
    /// Option trade.
    Optrd,
    /// Pass thru charge.
    Ptc,
    /// Pass thru rebate.
    Ptr,
    /// Reorg CA.
    Reorg,
    /// Stock spinoff.
    Spin,
    /// Stock split.
    Split,
    /// Free of payment transfers.
    Fopt,
}

impl ActivityType {
    /// The type as it appears in URLs and query parameters (e.g. `"FILL"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fill => "FILL",
            Self::Trans => "TRANS",
            Self::Misc => "MISC",
            Self::Acatc => "ACATC",
            Self::Acats => "ACATS",
            Self::Cfee => "CFEE",
            Self::Cgd => "CGD",
            Self::Csd => "CSD",
            Self::Csw => "CSW",
            Self::Div => "DIV",
            Self::Divcgl => "DIVCGL",
            Self::Divcgs => "DIVCGS",
            Self::Divfee => "DIVFEE",
            Self::Divft => "DIVFT",
            Self::Divnra => "DIVNRA",
            Self::Divroc => "DIVROC",
            Self::Divtw => "DIVTW",
            Self::Divtxex => "DIVTXEX",
            Self::Fee => "FEE",
            Self::Int => "INT",
            Self::Intnra => "INTNRA",
            Self::Inttw => "INTTW",
            Self::Jnl => "JNL",
            Self::Jnlc => "JNLC",
            Self::Jnls => "JNLS",
            Self::Ma => "MA",
            Self::Nc => "NC",
            Self::Opasn => "OPASN",
            Self::Opca => "OPCA",
            Self::Opcsh => "OPCSH",
            Self::Opexc => "OPEXC",
            Self::Opexp => "OPEXP",
            Self::Optrd => "OPTRD",
            Self::Ptc => "PTC",
            Self::Ptr => "PTR",
            Self::Reorg => "REORG",
            Self::Spin => "SPIN",
            Self::Split => "SPLIT",
            Self::Fopt => "FOPT",
        }
    }
}

/// The type of a trade activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeActivityType {
    /// A partial fill.
    PartialFill,
    /// A full fill.
    Fill,
}

/// The status of a non-trade activity (not present for all activity types).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NonTradeActivityStatus {
    /// The activity has been executed.
    Executed,
    /// The activity has been corrected.
    Correct,
    /// The activity has been canceled.
    Canceled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_type_screaming_case_serde() {
        assert_eq!(
            serde_json::to_string(&ActivityType::Fill).unwrap(),
            "\"FILL\""
        );
        assert_eq!(
            serde_json::to_string(&ActivityType::Divnra).unwrap(),
            "\"DIVNRA\""
        );
        assert_eq!(
            serde_json::to_string(&ActivityType::Opexc).unwrap(),
            "\"OPEXC\""
        );
        assert_eq!(
            serde_json::from_str::<ActivityType>("\"FOPT\"").unwrap(),
            ActivityType::Fopt
        );
        // `as_str` must agree with the serde representation.
        let json = serde_json::to_string(&ActivityType::Opcsh).unwrap();
        assert_eq!(json.trim_matches('"'), ActivityType::Opcsh.as_str());
    }
}
