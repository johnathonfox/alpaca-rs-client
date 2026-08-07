//! Shared query parameters for the market data endpoints.
//!
//! All structs derive [`Serialize`]; `None` fields are omitted from the query
//! string. Multi-symbol requests take an iterator of symbols in their
//! constructor and store them comma-separated, as the API expects.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Serialize, Serializer};

use super::enums::{
    Adjustment, CorporateActionsType, DataFeed, MarketType, MostActivesBy, Sort, TimeFrame,
};

fn join_symbols<I, S>(symbols: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    symbols
        .into_iter()
        .map(|s| s.as_ref().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// Query parameters for historical bars endpoints.
#[derive(Debug, Clone, Serialize)]
pub struct BarsRequest {
    /// Comma-separated symbols.
    pub symbols: String,
    /// Bar timeframe.
    pub timeframe: TimeFrame,
    /// Start of the time range (RFC3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<DateTime<Utc>>,
    /// End of the time range (RFC3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<DateTime<Utc>>,
    /// Maximum number of bars per symbol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    /// Corporate action adjustment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment: Option<Adjustment>,
    /// Data feed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed: Option<DataFeed>,
    /// Sort direction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sort>,
    /// Currency for the prices (ISO 4217).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// As-of date for the data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asof: Option<String>,
}

impl BarsRequest {
    /// Creates a request for the given symbols and timeframe; all other
    /// parameters default to `None` and can be set directly.
    pub fn new<I, S>(symbols: I, timeframe: TimeFrame) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            symbols: join_symbols(symbols),
            timeframe,
            start: None,
            end: None,
            limit: None,
            page_token: None,
            adjustment: None,
            feed: None,
            sort: None,
            currency: None,
            asof: None,
        }
    }
}

/// Query parameters for historical trades endpoints.
#[derive(Debug, Clone, Serialize)]
pub struct TradesRequest {
    /// Comma-separated symbols.
    pub symbols: String,
    /// Start of the time range (RFC3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<DateTime<Utc>>,
    /// End of the time range (RFC3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<DateTime<Utc>>,
    /// Maximum number of trades per symbol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    /// Data feed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed: Option<DataFeed>,
    /// Sort direction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sort>,
    /// Currency for the prices (ISO 4217).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// As-of date for the data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asof: Option<String>,
}

impl TradesRequest {
    /// Creates a request for the given symbols; all other parameters default
    /// to `None`.
    pub fn new<I, S>(symbols: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            symbols: join_symbols(symbols),
            start: None,
            end: None,
            limit: None,
            page_token: None,
            feed: None,
            sort: None,
            currency: None,
            asof: None,
        }
    }
}

/// Query parameters for historical quotes endpoints (same shape as
/// [`TradesRequest`]).
pub type QuotesRequest = TradesRequest;

/// Query parameters for the historical auctions endpoints (same shape as
/// [`TradesRequest`]; only [`DataFeed::Sip`] is a valid `feed`).
pub type AuctionsRequest = TradesRequest;

/// Query parameters for the "latest" endpoints (trades, quotes, bars,
/// orderbooks, snapshots).
#[derive(Debug, Clone, Serialize)]
pub struct LatestRequest {
    /// Comma-separated symbols.
    pub symbols: String,
    /// Data feed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed: Option<DataFeed>,
    /// Currency for the prices (ISO 4217).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

impl LatestRequest {
    /// Creates a request for the given symbols.
    pub fn new<I, S>(symbols: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            symbols: join_symbols(symbols),
            feed: None,
            currency: None,
        }
    }
}

/// Query parameters for the news endpoint.
#[derive(Debug, Clone, Default, Serialize)]
pub struct NewsRequest {
    /// Comma-separated symbols to filter by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<String>,
    /// Start of the time range (RFC3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<DateTime<Utc>>,
    /// End of the time range (RFC3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<DateTime<Utc>>,
    /// Maximum number of articles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Sort direction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sort>,
    /// Whether to include article content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_content: Option<bool>,
    /// Whether to exclude articles without content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_contentless: Option<bool>,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

impl NewsRequest {
    /// Creates a request filtered by the given symbols.
    pub fn for_symbols<I, S>(symbols: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            symbols: Some(join_symbols(symbols)),
            ..Default::default()
        }
    }
}

/// Query parameters for the most actives screener endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct MostActivesRequest {
    /// The metric used to rank the most active stocks.
    pub by: MostActivesBy,
    /// Number of top most active stocks to fetch.
    pub top: u32,
}

impl MostActivesRequest {
    /// Creates a request ranked by the given metric, fetching `top` entries.
    pub fn new(by: MostActivesBy, top: u32) -> Self {
        Self { by, top }
    }
}

impl Default for MostActivesRequest {
    /// Top 10 by volume, mirroring the API defaults.
    fn default() -> Self {
        Self::new(MostActivesBy::Volume, 10)
    }
}

/// Query parameters for the market movers screener endpoint. The market
/// type selects the URL path (`stocks` or `crypto`) and is not sent as a
/// query parameter.
#[derive(Debug, Clone, Serialize)]
pub struct MarketMoversRequest {
    /// The market to screen.
    #[serde(skip)]
    pub market_type: MarketType,
    /// Number of top movers to fetch.
    pub top: u32,
}

impl MarketMoversRequest {
    /// Creates a request for the given market, fetching `top` gainers and
    /// losers.
    pub fn new(market_type: MarketType, top: u32) -> Self {
        Self { market_type, top }
    }
}

impl Default for MarketMoversRequest {
    /// Top 10 stock movers, mirroring the API defaults.
    fn default() -> Self {
        Self::new(MarketType::Stocks, 10)
    }
}

/// Query parameters for the historical forex rates endpoint.
///
/// Currency pairs are the concatenated ISO 4217 codes of the base and quote
/// currency (e.g. `USDJPY`).
#[derive(Debug, Clone, Serialize)]
pub struct ForexRatesRequest {
    /// Comma-separated currency pairs.
    pub currency_pairs: String,
    /// Rate timeframe (the API defaults to `1Day`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeframe: Option<TimeFrame>,
    /// Start of the time range (RFC3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<DateTime<Utc>>,
    /// End of the time range (RFC3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<DateTime<Utc>>,
    /// Maximum number of rates per currency pair.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Sort direction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sort>,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

impl ForexRatesRequest {
    /// Creates a request for the given currency pairs; all other parameters
    /// default to `None` and can be set directly.
    pub fn new<I, S>(currency_pairs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            currency_pairs: join_symbols(currency_pairs),
            timeframe: None,
            start: None,
            end: None,
            limit: None,
            sort: None,
            page_token: None,
        }
    }
}

/// Query parameters for the latest forex rates endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct LatestForexRatesRequest {
    /// Comma-separated currency pairs.
    pub currency_pairs: String,
}

impl LatestForexRatesRequest {
    /// Creates a request for the given currency pairs.
    pub fn new<I, S>(currency_pairs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            currency_pairs: join_symbols(currency_pairs),
        }
    }
}

/// Serializes a list of corporate action types as a comma-separated query
/// parameter value.
fn serialize_types<S: Serializer>(
    types: &Option<Vec<CorporateActionsType>>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    match types {
        Some(types) => {
            let joined = types
                .iter()
                .map(CorporateActionsType::as_str)
                .collect::<Vec<_>>()
                .join(",");
            serializer.serialize_str(&joined)
        }
        None => serializer.serialize_none(),
    }
}

/// Query parameters for the corporate actions endpoint.
///
/// `ids` is mutually exclusive with all other filters (symbols, cusips,
/// types, start, end).
#[derive(Debug, Clone, Default, Serialize)]
pub struct CorporateActionsRequest {
    /// Comma-separated symbols.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<String>,
    /// Comma-separated CUSIPs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cusips: Option<String>,
    /// The types of corporate actions to filter by (default: all types).
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_types"
    )]
    pub types: Option<Vec<CorporateActionsType>>,
    /// Inclusive start of the interval (YYYY-MM-DD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<NaiveDate>,
    /// Inclusive end of the interval (YYYY-MM-DD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<NaiveDate>,
    /// Comma-separated corporate action ids.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<String>,
    /// Maximum number of data points to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Sort direction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sort>,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

impl CorporateActionsRequest {
    /// Creates a request filtered by the given symbols.
    pub fn for_symbols<I, S>(symbols: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            symbols: Some(join_symbols(symbols)),
            ..Default::default()
        }
    }

    /// Creates a request filtered by the given CUSIPs.
    pub fn for_cusips<I, S>(cusips: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            cusips: Some(join_symbols(cusips)),
            ..Default::default()
        }
    }

    /// Creates a request for the given corporate action ids.
    pub fn for_ids<I, S>(ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            ids: Some(join_symbols(ids)),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::enums::TimeFrameUnit;

    #[test]
    fn bars_request_serializes_query_params() {
        let mut req = BarsRequest::new(
            ["AAPL", "MSFT"],
            TimeFrame::new(5, TimeFrameUnit::Min).unwrap(),
        );
        req.adjustment = Some(Adjustment::Split);
        req.sort = Some(Sort::Asc);
        req.limit = Some(100);
        let qs = serde_urlencoded_like(&req);
        assert!(qs.contains("symbols=AAPL%2CMSFT"), "qs: {qs}");
        assert!(qs.contains("timeframe=5Min"), "qs: {qs}");
        assert!(qs.contains("adjustment=split"), "qs: {qs}");
        assert!(qs.contains("sort=asc"), "qs: {qs}");
        assert!(qs.contains("limit=100"), "qs: {qs}");
        assert!(!qs.contains("page_token"), "qs: {qs}");
        assert!(!qs.contains("feed"), "qs: {qs}");
    }

    #[test]
    fn most_actives_request_serializes_query_params() {
        let req = MostActivesRequest::new(MostActivesBy::Trades, 5);
        let qs = serde_urlencoded_like(&req);
        assert!(qs.contains("by=trades"), "qs: {qs}");
        assert!(qs.contains("top=5"), "qs: {qs}");
    }

    #[test]
    fn market_movers_request_omits_market_type_from_query() {
        let req = MarketMoversRequest::new(MarketType::Crypto, 20);
        let qs = serde_urlencoded_like(&req);
        assert!(qs.contains("top=20"), "qs: {qs}");
        assert!(!qs.contains("market_type"), "qs: {qs}");
    }

    #[test]
    fn corporate_actions_request_serializes_query_params() {
        let mut req = CorporateActionsRequest::for_symbols(["AAPL", "TSLA"]);
        req.cusips = Some("CUSIP1,CUSIP2".to_string());
        req.types = Some(vec![
            CorporateActionsType::CashDividend,
            CorporateActionsType::ForwardSplit,
        ]);
        req.start = Some(NaiveDate::from_ymd_opt(2022, 2, 1).unwrap());
        req.ids = Some("a7932b15-f816-4f83-921e-998b524487b0".to_string());
        req.sort = Some(Sort::Asc);
        req.limit = Some(1000);
        let qs = serde_urlencoded_like(&req);
        assert!(qs.contains("symbols=AAPL%2CTSLA"), "qs: {qs}");
        assert!(qs.contains("cusips=CUSIP1%2CCUSIP2"), "qs: {qs}");
        assert!(
            qs.contains("types=cash_dividend%2Cforward_split"),
            "qs: {qs}"
        );
        assert!(qs.contains("start=2022-02-01"), "qs: {qs}");
        assert!(qs.contains("ids=a7932b15"), "qs: {qs}");
        assert!(qs.contains("sort=asc"), "qs: {qs}");
        assert!(qs.contains("limit=1000"), "qs: {qs}");
        assert!(!qs.contains("page_token"), "qs: {qs}");
        assert!(!qs.contains("end="), "qs: {qs}");
    }

    #[test]
    fn forex_rates_request_serializes_query_params() {
        let mut req = ForexRatesRequest::new(["USDJPY", "USDMXN"]);
        req.timeframe = Some(TimeFrame::new(1, TimeFrameUnit::Hour).unwrap());
        req.sort = Some(Sort::Desc);
        req.limit = Some(500);
        let qs = serde_urlencoded_like(&req);
        assert!(qs.contains("currency_pairs=USDJPY%2CUSDMXN"), "qs: {qs}");
        assert!(qs.contains("timeframe=1Hour"), "qs: {qs}");
        assert!(qs.contains("sort=desc"), "qs: {qs}");
        assert!(qs.contains("limit=500"), "qs: {qs}");
        assert!(!qs.contains("start"), "qs: {qs}");
        assert!(!qs.contains("page_token"), "qs: {qs}");
    }

    #[test]
    fn latest_forex_rates_request_serializes_currency_pairs() {
        let req = LatestForexRatesRequest::new(["USDJPY"]);
        assert_eq!(serde_urlencoded_like(&req), "currency_pairs=USDJPY");
    }

    #[test]
    fn corporate_actions_request_default_serializes_empty() {
        let qs = serde_urlencoded_like(&CorporateActionsRequest::default());
        assert_eq!(qs, "");
    }

    // Emulate the urlencoded query string that reqwest's `.query()` builds.
    fn serde_urlencoded_like<T: Serialize>(v: &T) -> String {
        let json = serde_json::to_value(v).unwrap();
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        if let serde_json::Value::Object(map) = json {
            for (k, val) in map {
                let val = match val {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };
                serializer.append_pair(&k, &val);
            }
        }
        serializer.finish()
    }
}
