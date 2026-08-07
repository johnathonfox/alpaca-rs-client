//! Enums used by the trading API.

use serde::{Deserialize, Serialize};

/// The side of an order or execution.
///
/// `SellShort` does not appear on orders you submit, but the activities and
/// events endpoints report it on short-sale executions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderSide {
    /// Buy.
    Buy,
    /// Sell.
    Sell,
    /// Sell short — reported on executions, not submitted.
    SellShort,
    /// A side added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
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

/// How hard an asset is to borrow for short selling. Supersedes the
/// deprecated `easy_to_borrow` flag on [`Asset`](super::models::Asset).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BorrowStatus {
    /// The asset is easy to borrow.
    EasyToBorrow,
    /// The asset is hard to borrow.
    HardToBorrow,
    /// A status added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

/// A behavioural attribute of an asset, as returned in `Asset.attributes` and
/// accepted by the `attributes` filter of the list-assets endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetAttribute {
    /// Publicly traded partnership with no section 1446(f) exception.
    PtpNoException,
    /// Publicly traded partnership with a section 1446(f) exception.
    PtpWithException,
    /// The asset is in its initial public offering period.
    Ipo,
    /// The asset has tradable option contracts.
    HasOptions,
    /// Options on the asset close later than the regular session.
    OptionsLateClose,
    /// Fractional trading is enabled during extended hours.
    FractionalEhEnabled,
    /// The asset is tradable during the overnight session.
    OvernightTradable,
    /// The asset is halted for the overnight session.
    OvernightHalted,
    /// An attribute added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

impl AssetAttribute {
    /// The attribute as it appears in query parameters (e.g.
    /// `"has_options"`).
    pub fn as_str(&self) -> &str {
        match self {
            Self::PtpNoException => "ptp_no_exception",
            Self::PtpWithException => "ptp_with_exception",
            Self::Ipo => "ipo",
            Self::HasOptions => "has_options",
            Self::OptionsLateClose => "options_late_close",
            Self::FractionalEhEnabled => "fractional_eh_enabled",
            Self::OvernightTradable => "overnight_tradable",
            Self::OvernightHalted => "overnight_halted",
            Self::Other(value) => value,
        }
    }
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
///
/// Alpaca adds activity types over time, so an unrecognized value parses into
/// [`Other`](Self::Other) rather than failing the whole response, and
/// round-trips back out unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// An activity type added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

impl ActivityType {
    /// The type as it appears in URLs and query parameters (e.g. `"FILL"`).
    pub fn as_str(&self) -> &str {
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
            Self::Other(raw) => raw,
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

/// A market addressed by the v3 clock and calendar endpoints, identified by
/// its MIC, BIC or acronym. Values are serialized as-is (e.g. `"NYSE"`).
///
/// Some markets appear under more than one code (e.g. [`Market::NYSE`] and
/// [`Market::XNYS`]); the API treats them as distinct identifiers and echoes
/// back whichever one was asked for.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Market {
    /// Bank of Montreal — US banking calendar (US operations).
    BMO,
    /// The Bank of New York Mellon — US banking calendar.
    BNYM,
    /// Blue Ocean Alternative Trading System — US overnight trading.
    BOATS,
    /// Cboe CEUX Europe — European equities (Cboe Netherlands).
    CEUX,
    /// Cboe CHIX Europe — European equities (Cboe UK).
    CHIX,
    /// Hong Kong Stock Exchange — Hong Kong equities.
    HKEX,
    /// Investors Exchange — US equities.
    IEX,
    /// Investors Exchange — US equities.
    IEXG,
    /// Euronext Dublin — Irish equities.
    ISE,
    /// London Stock Exchange — UK equities.
    LSE,
    /// Euronext Milan — Italian equities.
    MTA,
    /// Euronext Milan — Italian equities.
    MTAA,
    /// NASDAQ — US equities.
    NASDAQ,
    /// New York Stock Exchange — US equities.
    NYSE,
    /// BlueOcean ATS — US overnight trading.
    OCEA,
    /// Options Price Reporting Authority — US options.
    OPRA,
    /// Over-the-counter — US OTC equities.
    OTC,
    /// OTC Markets — US OTC equities.
    OTCM,
    /// Securities Industry and Financial Markets Association — US bonds.
    SIFMA,
    /// Saudi Stock Exchange — Saudi equities.
    TADAWUL,
    /// Euronext Amsterdam — Dutch equities.
    XAMS,
    /// Euronext Brussels — Belgian equities.
    XBRU,
    /// Euronext Dublin — Irish equities.
    XDUB,
    /// Frankfurt Stock Exchange — German equities.
    XETR,
    /// Frankfurt Stock Exchange — German equities.
    XETRA,
    /// Hong Kong Stock Exchange — Hong Kong equities.
    XHKG,
    /// Euronext Lisbon — Portuguese equities.
    XLIS,
    /// London Stock Exchange — UK equities.
    XLON,
    /// NASDAQ — US equities.
    XNAS,
    /// New York Stock Exchange — US equities.
    XNYS,
    /// Euronext Paris — French equities.
    XPAR,
    /// Saudi Stock Exchange — Saudi equities.
    XSAU,
    /// A market added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

impl Market {
    /// The market as it appears in URLs and query parameters (e.g.
    /// `"NYSE"`).
    pub fn as_str(&self) -> &str {
        match self {
            Self::BMO => "BMO",
            Self::BNYM => "BNYM",
            Self::BOATS => "BOATS",
            Self::CEUX => "CEUX",
            Self::CHIX => "CHIX",
            Self::HKEX => "HKEX",
            Self::IEX => "IEX",
            Self::IEXG => "IEXG",
            Self::ISE => "ISE",
            Self::LSE => "LSE",
            Self::MTA => "MTA",
            Self::MTAA => "MTAA",
            Self::NASDAQ => "NASDAQ",
            Self::NYSE => "NYSE",
            Self::OCEA => "OCEA",
            Self::OPRA => "OPRA",
            Self::OTC => "OTC",
            Self::OTCM => "OTCM",
            Self::SIFMA => "SIFMA",
            Self::TADAWUL => "TADAWUL",
            Self::XAMS => "XAMS",
            Self::XBRU => "XBRU",
            Self::XDUB => "XDUB",
            Self::XETR => "XETR",
            Self::XETRA => "XETRA",
            Self::XHKG => "XHKG",
            Self::XLIS => "XLIS",
            Self::XLON => "XLON",
            Self::XNAS => "XNAS",
            Self::XNYS => "XNYS",
            Self::XPAR => "XPAR",
            Self::XSAU => "XSAU",
            Self::Other(value) => value,
        }
    }
}

/// The session a market is currently in, as reported by `GET /v3/clock`.
///
/// Markets without a given session simply never report it — an overnight
/// venue such as [`Market::BOATS`] runs its whole session as
/// [`MarketPhase::Core`] and is [`MarketPhase::Closed`] during the regular
/// day.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketPhase {
    /// The market is closed.
    Closed,
    /// Pre-market session.
    Pre,
    /// Core (regular) session.
    Core,
    /// Midday break between core sessions (Asian markets).
    Lunch,
    /// Post-market (after-hours) session.
    Post,
    /// A phase added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

impl MarketPhase {
    /// The phase as it appears on the wire (e.g. `"core"`).
    pub fn as_str(&self) -> &str {
        match self {
            Self::Closed => "closed",
            Self::Pre => "pre",
            Self::Core => "core",
            Self::Lunch => "lunch",
            Self::Post => "post",
            Self::Other(value) => value,
        }
    }
}

/// Timezone the v3 calendar renders session times in. The API accepts only
/// UTC; without it, times come back in the market's own timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CalendarTimezone {
    /// Coordinated Universal Time.
    #[serde(rename = "UTC")]
    Utc,
}

/// The lifecycle status of a short-sale locate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocateStatus {
    /// The locate is reserved and can be shorted against until it expires.
    Active,
    /// The locate has expired and no longer reserves inventory.
    Expired,
    /// The locate was rejected; see
    /// [`Locate::rejection_reason`](super::models::Locate::rejection_reason).
    Rejected,
    /// A status added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

impl LocateStatus {
    /// The status as it appears on the wire (e.g. `"active"`).
    pub fn as_str(&self) -> &str {
        match self {
            Self::Active => "active",
            Self::Expired => "expired",
            Self::Rejected => "rejected",
            Self::Other(value) => value,
        }
    }
}

/// Why a symbol could not be quoted for a locate, as returned in the `errors`
/// list of `GET /v1/locates/quotes`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocateQuoteErrorCode {
    /// The symbol is unknown to Alpaca.
    SymbolNotFound,
    /// The security is easy to borrow and needs no locate.
    EasyToBorrow,
    /// The security is on the threshold list and cannot be located.
    ThresholdSecurity,
    /// A pending corporate action blocks locates on the security.
    CorporateAction,
    /// No locate quote is currently available for the security.
    QuoteUnavailable,
    /// A code added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
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
    fn activity_type_keeps_unknown_wire_values() {
        // Alpaca adds activity types over time; an unrecognized one must not
        // fail the whole activities response.
        let parsed: ActivityType = serde_json::from_str("\"VOF\"").unwrap();
        assert_eq!(parsed, ActivityType::Other("VOF".to_string()));
        assert_eq!(parsed.as_str(), "VOF");
        // ...and it round-trips back out unchanged.
        assert_eq!(serde_json::to_string(&parsed).unwrap(), "\"VOF\"");
    }

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

    #[test]
    fn asset_attribute_serde_and_forward_compat() {
        assert_eq!(
            serde_json::to_string(&AssetAttribute::PtpWithException).unwrap(),
            "\"ptp_with_exception\""
        );
        assert_eq!(
            serde_json::from_str::<AssetAttribute>("\"fractional_eh_enabled\"").unwrap(),
            AssetAttribute::FractionalEhEnabled
        );

        // An attribute this crate does not know falls into `Other` and
        // round-trips back to the same wire value.
        let unknown: AssetAttribute = serde_json::from_str("\"warrants_enabled\"").unwrap();
        assert_eq!(unknown, AssetAttribute::Other("warrants_enabled".into()));
        assert_eq!(
            serde_json::to_string(&unknown).unwrap(),
            "\"warrants_enabled\""
        );

        // `as_str` must agree with the serde representation for both known
        // and unknown attributes.
        for attribute in [
            AssetAttribute::Ipo,
            AssetAttribute::OvernightHalted,
            unknown,
        ] {
            let json = serde_json::to_string(&attribute).unwrap();
            assert_eq!(json.trim_matches('"'), attribute.as_str());
        }
    }

    #[test]
    fn market_serde_and_forward_compat() {
        assert_eq!(serde_json::to_string(&Market::NYSE).unwrap(), "\"NYSE\"");
        assert_eq!(
            serde_json::from_str::<Market>("\"BOATS\"").unwrap(),
            Market::BOATS
        );
        assert_eq!(
            serde_json::from_str::<Market>("\"TADAWUL\"").unwrap(),
            Market::TADAWUL
        );

        // A market this crate does not know falls into `Other` and round-trips
        // back to the same wire value.
        let unknown: Market = serde_json::from_str("\"XPHL\"").unwrap();
        assert_eq!(unknown, Market::Other("XPHL".into()));
        assert_eq!(serde_json::to_string(&unknown).unwrap(), "\"XPHL\"");

        // `as_str` must agree with the serde representation.
        for market in [Market::OPRA, Market::OCEA, Market::XNYS, unknown] {
            let json = serde_json::to_string(&market).unwrap();
            assert_eq!(json.trim_matches('"'), market.as_str());
        }
    }

    #[test]
    fn market_phase_serde_and_forward_compat() {
        assert_eq!(
            serde_json::to_string(&MarketPhase::Core).unwrap(),
            "\"core\""
        );
        assert_eq!(
            serde_json::from_str::<MarketPhase>("\"closed\"").unwrap(),
            MarketPhase::Closed
        );
        assert_eq!(
            serde_json::from_str::<MarketPhase>("\"lunch\"").unwrap(),
            MarketPhase::Lunch
        );

        let unknown: MarketPhase = serde_json::from_str("\"overnight\"").unwrap();
        assert_eq!(unknown, MarketPhase::Other("overnight".into()));
        assert_eq!(serde_json::to_string(&unknown).unwrap(), "\"overnight\"");

        for phase in [MarketPhase::Pre, MarketPhase::Post, unknown] {
            let json = serde_json::to_string(&phase).unwrap();
            assert_eq!(json.trim_matches('"'), phase.as_str());
        }
    }

    #[test]
    fn calendar_timezone_serializes_uppercase() {
        assert_eq!(
            serde_json::to_string(&CalendarTimezone::Utc).unwrap(),
            "\"UTC\""
        );
        assert_eq!(
            serde_json::from_str::<CalendarTimezone>("\"UTC\"").unwrap(),
            CalendarTimezone::Utc
        );
    }

    #[test]
    fn locate_status_serde_and_forward_compat() {
        assert_eq!(
            serde_json::to_string(&LocateStatus::Active).unwrap(),
            "\"active\""
        );
        assert_eq!(
            serde_json::from_str::<LocateStatus>("\"expired\"").unwrap(),
            LocateStatus::Expired
        );
        assert_eq!(
            serde_json::from_str::<LocateStatus>("\"rejected\"").unwrap(),
            LocateStatus::Rejected
        );

        let unknown: LocateStatus = serde_json::from_str("\"pending\"").unwrap();
        assert_eq!(unknown, LocateStatus::Other("pending".into()));
        assert_eq!(serde_json::to_string(&unknown).unwrap(), "\"pending\"");

        // `as_str` must agree with the serde representation.
        for status in [LocateStatus::Active, LocateStatus::Expired, unknown] {
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json.trim_matches('"'), status.as_str());
        }
    }

    #[test]
    fn locate_quote_error_code_serde_and_forward_compat() {
        assert_eq!(
            serde_json::from_str::<LocateQuoteErrorCode>("\"symbol_not_found\"").unwrap(),
            LocateQuoteErrorCode::SymbolNotFound
        );
        assert_eq!(
            serde_json::to_string(&LocateQuoteErrorCode::ThresholdSecurity).unwrap(),
            "\"threshold_security\""
        );
        assert_eq!(
            serde_json::from_str::<LocateQuoteErrorCode>("\"market_closed\"").unwrap(),
            LocateQuoteErrorCode::Other("market_closed".into())
        );
    }

    #[test]
    fn borrow_status_serde_and_forward_compat() {
        assert_eq!(
            serde_json::from_str::<BorrowStatus>("\"hard_to_borrow\"").unwrap(),
            BorrowStatus::HardToBorrow
        );
        assert_eq!(
            serde_json::to_string(&BorrowStatus::EasyToBorrow).unwrap(),
            "\"easy_to_borrow\""
        );
        assert_eq!(
            serde_json::from_str::<BorrowStatus>("\"not_borrowable\"").unwrap(),
            BorrowStatus::Other("not_borrowable".into())
        );
    }
}
