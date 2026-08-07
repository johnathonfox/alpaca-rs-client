//! Enums and the [`TimeFrame`] type used by the market data API.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{Error, Result};

/// The market data feed to source data from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataFeed {
    /// Investors Exchange (free plan).
    Iex,
    /// Securities Information Processor (all US exchanges).
    Sip,
    /// SIP data delayed by 15 minutes.
    DelayedSip,
    /// Over the counter.
    Otc,
    /// Cboe One ("BOATS").
    Boats,
    /// Overnight session feed.
    Overnight,
}

impl DataFeed {
    /// The feed as it appears in stream URL paths.
    pub fn as_str(&self) -> &'static str {
        match self {
            DataFeed::Iex => "iex",
            DataFeed::Sip => "sip",
            DataFeed::DelayedSip => "delayed_sip",
            DataFeed::Otc => "otc",
            DataFeed::Boats => "boats",
            DataFeed::Overnight => "overnight",
        }
    }
}

/// The corporate action adjustment applied to historical bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Adjustment {
    /// No adjustment.
    Raw,
    /// Split adjustment.
    Split,
    /// Dividend adjustment.
    Dividend,
    /// All adjustments.
    All,
}

/// Sort direction for paginated endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    /// Ascending.
    Asc,
    /// Descending.
    Desc,
}

/// The crypto data feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CryptoFeed {
    /// United States feed.
    Us,
}

impl CryptoFeed {
    /// The feed as it appears in URL paths.
    pub fn as_str(&self) -> &'static str {
        match self {
            CryptoFeed::Us => "us",
        }
    }
}

/// The options data feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionsFeed {
    /// OPRA feed (all exchanges, requires agreement).
    Opra,
    /// Indicative feed (free plan).
    Indicative,
}

impl OptionsFeed {
    /// The feed as it appears in URL paths and query parameters.
    pub fn as_str(&self) -> &'static str {
        match self {
            OptionsFeed::Opra => "opra",
            OptionsFeed::Indicative => "indicative",
        }
    }
}

/// The metric used to rank the most active stocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MostActivesBy {
    /// Trading volume.
    Volume,
    /// Number of trades.
    Trades,
}

/// The market screened by the movers endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketType {
    /// US stocks.
    Stocks,
    /// Crypto.
    Crypto,
}

impl MarketType {
    /// The market type as it appears in URL paths.
    pub fn as_str(&self) -> &'static str {
        match self {
            MarketType::Stocks => "stocks",
            MarketType::Crypto => "crypto",
        }
    }
}

/// The consolidated tape a stock condition-code table applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Tape {
    /// Tape A — NYSE-listed securities.
    A,
    /// Tape B — NYSE Arca, NYSE American and other regionally listed
    /// securities.
    B,
    /// Tape C — Nasdaq-listed securities.
    C,
}

impl Tape {
    /// The tape as it appears in query parameters.
    pub fn as_str(&self) -> &'static str {
        match self {
            Tape::A => "A",
            Tape::B => "B",
            Tape::C => "C",
        }
    }
}

/// The kind of tick a condition-code table describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TickType {
    /// Trade conditions.
    Trade,
    /// Quote conditions.
    Quote,
}

impl TickType {
    /// The tick type as it appears in URL paths.
    pub fn as_str(&self) -> &'static str {
        match self {
            TickType::Trade => "trade",
            TickType::Quote => "quote",
        }
    }
}

/// The type of a corporate action.
///
/// ref. <https://docs.alpaca.markets/reference/corporateactions-1>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorporateActionsType {
    /// Reverse split.
    ReverseSplit,
    /// Forward split.
    ForwardSplit,
    /// Unit split.
    UnitSplit,
    /// Cash dividend.
    CashDividend,
    /// Stock dividend.
    StockDividend,
    /// Spin off.
    SpinOff,
    /// Cash merger.
    CashMerger,
    /// Stock merger.
    StockMerger,
    /// Stock and cash merger.
    StockAndCashMerger,
    /// Redemption.
    Redemption,
    /// Name change.
    NameChange,
    /// Worthless removal.
    WorthlessRemoval,
    /// Rights distribution.
    RightsDistribution,
}

impl CorporateActionsType {
    /// The type as it appears in query parameters.
    pub fn as_str(&self) -> &'static str {
        match self {
            CorporateActionsType::ReverseSplit => "reverse_split",
            CorporateActionsType::ForwardSplit => "forward_split",
            CorporateActionsType::UnitSplit => "unit_split",
            CorporateActionsType::CashDividend => "cash_dividend",
            CorporateActionsType::StockDividend => "stock_dividend",
            CorporateActionsType::SpinOff => "spin_off",
            CorporateActionsType::CashMerger => "cash_merger",
            CorporateActionsType::StockMerger => "stock_merger",
            CorporateActionsType::StockAndCashMerger => "stock_and_cash_merger",
            CorporateActionsType::Redemption => "redemption",
            CorporateActionsType::NameChange => "name_change",
            CorporateActionsType::WorthlessRemoval => "worthless_removal",
            CorporateActionsType::RightsDistribution => "rights_distribution",
        }
    }
}

/// The unit of a [`TimeFrame`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeFrameUnit {
    /// Minutes.
    Min,
    /// Hours.
    Hour,
    /// Days.
    Day,
    /// Weeks.
    Week,
    /// Months.
    Month,
}

impl fmt::Display for TimeFrameUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TimeFrameUnit::Min => "Min",
            TimeFrameUnit::Hour => "Hour",
            TimeFrameUnit::Day => "Day",
            TimeFrameUnit::Week => "Week",
            TimeFrameUnit::Month => "Month",
        };
        f.write_str(s)
    }
}

/// A bar timeframe such as `5Min` or `1Day`.
///
/// Serializes to the API string form (e.g. `"5Min"`). Valid amounts per unit:
/// minutes 1–59, hours 1–23, day/week exactly 1, months one of
/// 1, 2, 3, 6, 12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeFrame {
    /// Number of units.
    pub amount: u32,
    /// The unit.
    pub unit: TimeFrameUnit,
}

impl TimeFrame {
    /// One minute.
    pub const MINUTE: TimeFrame = TimeFrame {
        amount: 1,
        unit: TimeFrameUnit::Min,
    };
    /// One hour.
    pub const HOUR: TimeFrame = TimeFrame {
        amount: 1,
        unit: TimeFrameUnit::Hour,
    };
    /// One day.
    pub const DAY: TimeFrame = TimeFrame {
        amount: 1,
        unit: TimeFrameUnit::Day,
    };
    /// One week.
    pub const WEEK: TimeFrame = TimeFrame {
        amount: 1,
        unit: TimeFrameUnit::Week,
    };
    /// One month.
    pub const MONTH: TimeFrame = TimeFrame {
        amount: 1,
        unit: TimeFrameUnit::Month,
    };

    /// Creates a timeframe after validating the amount for the unit.
    pub fn new(amount: u32, unit: TimeFrameUnit) -> Result<Self> {
        let valid = match unit {
            TimeFrameUnit::Min => (1..=59).contains(&amount),
            TimeFrameUnit::Hour => (1..=23).contains(&amount),
            TimeFrameUnit::Day | TimeFrameUnit::Week => amount == 1,
            TimeFrameUnit::Month => matches!(amount, 1 | 2 | 3 | 6 | 12),
        };
        if valid {
            Ok(Self { amount, unit })
        } else {
            Err(Error::InvalidTimeFrame(format!("{amount}{unit}")))
        }
    }
}

impl fmt::Display for TimeFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.amount, self.unit)
    }
}

impl Serialize for TimeFrame {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TimeFrame {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let (amount, unit) = s.split_at(s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len()));
        let amount: u32 = amount.parse().map_err(serde::de::Error::custom)?;
        let unit = match unit {
            "Min" => TimeFrameUnit::Min,
            "Hour" => TimeFrameUnit::Hour,
            "Day" => TimeFrameUnit::Day,
            "Week" => TimeFrameUnit::Week,
            "Month" => TimeFrameUnit::Month,
            other => {
                return Err(serde::de::Error::custom(format!(
                    "unknown timeframe unit: {other}"
                )));
            }
        };
        TimeFrame::new(amount, unit).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeframe_formatting() {
        assert_eq!(
            TimeFrame::new(5, TimeFrameUnit::Min).unwrap().to_string(),
            "5Min"
        );
        assert_eq!(TimeFrame::HOUR.to_string(), "1Hour");
        assert_eq!(TimeFrame::DAY.to_string(), "1Day");
        assert_eq!(
            serde_json::to_string(&TimeFrame::WEEK).unwrap(),
            "\"1Week\""
        );
    }

    #[test]
    fn timeframe_invalid_amounts_rejected() {
        assert!(TimeFrame::new(0, TimeFrameUnit::Min).is_err());
        assert!(TimeFrame::new(60, TimeFrameUnit::Min).is_err());
        assert!(TimeFrame::new(24, TimeFrameUnit::Hour).is_err());
        assert!(TimeFrame::new(2, TimeFrameUnit::Day).is_err());
        assert!(TimeFrame::new(5, TimeFrameUnit::Month).is_err());
        assert!(TimeFrame::new(12, TimeFrameUnit::Month).is_ok());
    }

    #[test]
    fn timeframe_deserialize() {
        let tf: TimeFrame = serde_json::from_str("\"15Min\"").unwrap();
        assert_eq!(tf, TimeFrame::new(15, TimeFrameUnit::Min).unwrap());
        assert!(serde_json::from_str::<TimeFrame>("\"7Day\"").is_err());
    }

    #[test]
    fn screener_enums_serde_round_trip() {
        assert_eq!(
            serde_json::to_string(&MostActivesBy::Volume).unwrap(),
            "\"volume\""
        );
        assert_eq!(
            serde_json::to_string(&MostActivesBy::Trades).unwrap(),
            "\"trades\""
        );
        let by: MostActivesBy = serde_json::from_str("\"trades\"").unwrap();
        assert_eq!(by, MostActivesBy::Trades);

        assert_eq!(
            serde_json::to_string(&MarketType::Stocks).unwrap(),
            "\"stocks\""
        );
        let market_type: MarketType = serde_json::from_str("\"crypto\"").unwrap();
        assert_eq!(market_type, MarketType::Crypto);
        assert_eq!(market_type.as_str(), "crypto");
    }

    #[test]
    fn meta_enums_serde_round_trip() {
        assert_eq!(serde_json::to_string(&Tape::A).unwrap(), "\"A\"");
        assert_eq!(Tape::B.as_str(), "B");
        let tape: Tape = serde_json::from_str("\"C\"").unwrap();
        assert_eq!(tape, Tape::C);

        assert_eq!(
            serde_json::to_string(&TickType::Trade).unwrap(),
            "\"trade\""
        );
        assert_eq!(TickType::Quote.as_str(), "quote");
        let tick_type: TickType = serde_json::from_str("\"quote\"").unwrap();
        assert_eq!(tick_type, TickType::Quote);
    }

    #[test]
    fn corporate_actions_type_serde_round_trip() {
        let cases = [
            (CorporateActionsType::ReverseSplit, "reverse_split"),
            (CorporateActionsType::ForwardSplit, "forward_split"),
            (CorporateActionsType::UnitSplit, "unit_split"),
            (CorporateActionsType::CashDividend, "cash_dividend"),
            (CorporateActionsType::StockDividend, "stock_dividend"),
            (CorporateActionsType::SpinOff, "spin_off"),
            (CorporateActionsType::CashMerger, "cash_merger"),
            (CorporateActionsType::StockMerger, "stock_merger"),
            (
                CorporateActionsType::StockAndCashMerger,
                "stock_and_cash_merger",
            ),
            (CorporateActionsType::Redemption, "redemption"),
            (CorporateActionsType::NameChange, "name_change"),
            (CorporateActionsType::WorthlessRemoval, "worthless_removal"),
            (
                CorporateActionsType::RightsDistribution,
                "rights_distribution",
            ),
        ];
        for (ca_type, json) in cases {
            assert_eq!(ca_type.as_str(), json);
            assert_eq!(
                serde_json::to_string(&ca_type).unwrap(),
                format!("\"{json}\"")
            );
            let parsed: CorporateActionsType =
                serde_json::from_str(&format!("\"{json}\"")).unwrap();
            assert_eq!(parsed, ca_type);
        }
    }
}
