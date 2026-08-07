//! Models for the market data API.
//!
//! Market data uses plain JSON numbers, so numeric fields are `f64`. The API
//! uses short single/double-letter keys (`"o"`, `"bp"`, ...); the Rust fields
//! have full names with serde renames.

use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use super::enums::MarketType;
use crate::rest::PaginatedResponse;

/// A single OHLCV bar (candle).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bar {
    /// Bar timestamp (RFC3339).
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Open price.
    #[serde(rename = "o")]
    pub open: f64,
    /// High price.
    #[serde(rename = "h")]
    pub high: f64,
    /// Low price.
    #[serde(rename = "l")]
    pub low: f64,
    /// Close price.
    #[serde(rename = "c")]
    pub close: f64,
    /// Volume.
    #[serde(rename = "v")]
    pub volume: f64,
    /// Number of trades in the bar.
    #[serde(rename = "n")]
    pub trade_count: Option<u64>,
    /// Volume-weighted average price.
    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
}

/// A single NBBO quote.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quote {
    /// Quote timestamp (RFC3339).
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Bid exchange code.
    #[serde(rename = "bx")]
    pub bid_exchange: Option<String>,
    /// Bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,
    /// Bid size.
    #[serde(rename = "bs")]
    pub bid_size: f64,
    /// Ask exchange code.
    #[serde(rename = "ax")]
    pub ask_exchange: Option<String>,
    /// Ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,
    /// Ask size.
    #[serde(rename = "as")]
    pub ask_size: f64,
    /// Quote condition codes.
    #[serde(rename = "c", default)]
    pub conditions: Vec<String>,
    /// Tape.
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// A trade id: numeric for stocks/options, string for crypto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TradeId {
    /// Numeric trade id.
    Int(i64),
    /// String trade id.
    Str(String),
}

/// A single trade.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trade {
    /// Trade timestamp (RFC3339).
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Trade price.
    #[serde(rename = "p")]
    pub price: f64,
    /// Trade size.
    #[serde(rename = "s")]
    pub size: f64,
    /// Exchange code.
    #[serde(rename = "x")]
    pub exchange: Option<String>,
    /// Trade id.
    #[serde(rename = "i")]
    pub id: Option<TradeId>,
    /// Trade condition codes.
    #[serde(rename = "c", default)]
    pub conditions: Vec<String>,
    /// Tape.
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// A snapshot of the latest market state of a symbol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// The latest trade.
    #[serde(rename = "latestTrade")]
    pub latest_trade: Option<Trade>,
    /// The latest quote.
    #[serde(rename = "latestQuote")]
    pub latest_quote: Option<Quote>,
    /// The current minute bar.
    #[serde(rename = "minuteBar")]
    pub minute_bar: Option<Bar>,
    /// The current daily bar.
    #[serde(rename = "dailyBar")]
    pub daily_bar: Option<Bar>,
    /// The previous daily bar.
    #[serde(rename = "prevDailyBar")]
    pub prev_daily_bar: Option<Bar>,
}

/// One level of an orderbook.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderbookEntry {
    /// Price.
    #[serde(rename = "p")]
    pub price: f64,
    /// Size.
    #[serde(rename = "s")]
    pub size: f64,
}

/// A full orderbook snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Orderbook {
    /// Book timestamp (RFC3339).
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Bid levels.
    #[serde(rename = "b", default)]
    pub bids: Vec<OrderbookEntry>,
    /// Ask levels.
    #[serde(rename = "a", default)]
    pub asks: Vec<OrderbookEntry>,
}

/// A single opening or closing auction print.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuctionPrice {
    /// Auction timestamp (RFC3339).
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Exchange code.
    #[serde(rename = "x")]
    pub exchange: Option<String>,
    /// Auction price.
    #[serde(rename = "p")]
    pub price: f64,
    /// Auction size, when reported.
    #[serde(rename = "s")]
    pub size: Option<f64>,
    /// Auction condition code.
    #[serde(rename = "c")]
    pub condition: Option<String>,
}

/// The opening and closing auctions of one symbol on one trading day.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Auction {
    /// Trading day the auctions belong to.
    #[serde(rename = "d")]
    pub date: NaiveDate,
    /// Opening auctions.
    #[serde(rename = "o", default)]
    pub opening: Vec<AuctionPrice>,
    /// Closing auctions.
    #[serde(rename = "c", default)]
    pub closing: Vec<AuctionPrice>,
}

/// A news article.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsArticle {
    /// Article id.
    pub id: u64,
    /// Headline.
    pub headline: String,
    /// Author.
    pub author: Option<String>,
    /// When the article was created.
    pub created_at: DateTime<Utc>,
    /// When the article was last updated.
    pub updated_at: Option<DateTime<Utc>>,
    /// Summary text.
    pub summary: Option<String>,
    /// URL of the full article.
    pub url: Option<String>,
    /// Symbols the article mentions.
    #[serde(default)]
    pub symbols: Vec<String>,
    /// Source of the article.
    pub source: String,
}

/// Paginated per-symbol bars response (`{"bars": {"AAPL": [...]}}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BarsResponse {
    /// Bars keyed by symbol.
    pub bars: HashMap<String, Vec<Bar>>,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

/// Paginated per-symbol trades response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradesResponse {
    /// Trades keyed by symbol.
    pub trades: HashMap<String, Vec<Trade>>,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

/// Paginated per-symbol quotes response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuotesResponse {
    /// Quotes keyed by symbol.
    pub quotes: HashMap<String, Vec<Quote>>,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

/// Paginated per-symbol auctions response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuctionsResponse {
    /// Daily auctions keyed by symbol.
    pub auctions: HashMap<String, Vec<Auction>>,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

/// Per-symbol latest bars response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatestBarsResponse {
    /// Latest bar per symbol.
    pub bars: HashMap<String, Bar>,
}

/// Per-symbol latest trades response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatestTradesResponse {
    /// Latest trade per symbol.
    pub trades: HashMap<String, Trade>,
}

/// Per-symbol latest quotes response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatestQuotesResponse {
    /// Latest quote per symbol.
    pub quotes: HashMap<String, Quote>,
}

/// Per-symbol snapshots response. Only the options snapshots endpoints
/// paginate, so `next_page_token` is always `None` for stocks and crypto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotsResponse {
    /// Snapshot per symbol.
    pub snapshots: HashMap<String, Snapshot>,
    /// Token for the next page, if any.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

/// Per-symbol orderbooks response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderbooksResponse {
    /// Orderbook per symbol.
    pub orderbooks: HashMap<String, Orderbook>,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

/// Deserializes a possibly-`null` JSON array as an empty vector: the
/// single-symbol endpoints send `null` rather than `[]` when the symbol has
/// no data in the requested interval.
fn null_as_empty_vec<'de, D, T>(deserializer: D) -> std::result::Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

/// Paginated bars response of the single-symbol endpoint, where the data is
/// a flat list and the symbol is echoed at the top level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolBarsResponse {
    /// The symbol the bars belong to.
    pub symbol: String,
    /// The bars in this page.
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub bars: Vec<Bar>,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

/// Paginated trades response of the single-symbol endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolTradesResponse {
    /// The symbol the trades belong to.
    pub symbol: String,
    /// The trades in this page.
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub trades: Vec<Trade>,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

/// Paginated quotes response of the single-symbol endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolQuotesResponse {
    /// The symbol the quotes belong to.
    pub symbol: String,
    /// The quotes in this page.
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub quotes: Vec<Quote>,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

/// Paginated auctions response of the single-symbol endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolAuctionsResponse {
    /// The symbol the auctions belong to.
    pub symbol: String,
    /// The daily auctions in this page.
    #[serde(default, deserialize_with = "null_as_empty_vec")]
    pub auctions: Vec<Auction>,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

/// Latest bar response of the single-symbol endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolBarResponse {
    /// The symbol the bar belongs to.
    pub symbol: String,
    /// The latest bar.
    pub bar: Bar,
}

/// Latest trade response of the single-symbol endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolTradeResponse {
    /// The symbol the trade belongs to.
    pub symbol: String,
    /// The latest trade.
    pub trade: Trade,
}

/// Latest quote response of the single-symbol endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolQuoteResponse {
    /// The symbol the quote belongs to.
    pub symbol: String,
    /// The latest quote.
    pub quote: Quote,
}

/// Snapshot response of the single-symbol endpoint: the snapshot fields sit
/// at the top level next to the symbol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolSnapshot {
    /// The symbol the snapshot belongs to.
    pub symbol: String,
    /// The snapshot itself.
    #[serde(flatten)]
    pub snapshot: Snapshot,
}

/// Paginated news response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsResponse {
    /// The articles in this page.
    #[serde(default)]
    pub news: Vec<NewsArticle>,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

/// One asset on the most actives screener endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MostActive {
    /// Symbol of the asset.
    pub symbol: String,
    /// Cumulative volume for the current trading day.
    pub volume: f64,
    /// Cumulative trade count for the current trading day.
    pub trade_count: f64,
}

/// Response of the most actives screener endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MostActives {
    /// The top N most active symbols.
    #[serde(default)]
    pub most_actives: Vec<MostActive>,
    /// When the most actives were last computed (RFC3339).
    pub last_updated: DateTime<Utc>,
}

/// One asset on the market movers screener endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mover {
    /// Symbol of the asset.
    pub symbol: String,
    /// Percentage change for the day.
    pub percent_change: f64,
    /// Price change for the day.
    pub change: f64,
    /// Current price of the asset.
    pub price: f64,
}

/// Response of the market movers screener endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Movers {
    /// The top N gainers.
    #[serde(default)]
    pub gainers: Vec<Mover>,
    /// The top N losers.
    #[serde(default)]
    pub losers: Vec<Mover>,
    /// The market screened (stocks or crypto).
    pub market_type: MarketType,
    /// When the movers were last computed (RFC3339).
    pub last_updated: DateTime<Utc>,
}

/// A forward split.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForwardSplit {
    /// Corporate action id.
    pub id: String,
    /// Symbol.
    pub symbol: String,
    /// CUSIP.
    pub cusip: String,
    /// New rate.
    pub new_rate: f64,
    /// Old rate.
    pub old_rate: f64,
    /// Process date.
    pub process_date: NaiveDate,
    /// Ex date.
    pub ex_date: NaiveDate,
    /// Record date.
    #[serde(default)]
    pub record_date: Option<NaiveDate>,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Due bill redemption date.
    #[serde(default)]
    pub due_bill_redemption_date: Option<NaiveDate>,
}

/// A reverse split.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReverseSplit {
    /// Corporate action id.
    pub id: String,
    /// Symbol.
    pub symbol: String,
    /// Old CUSIP.
    pub old_cusip: String,
    /// New CUSIP.
    pub new_cusip: String,
    /// New rate.
    pub new_rate: f64,
    /// Old rate.
    pub old_rate: f64,
    /// Process date.
    pub process_date: NaiveDate,
    /// Ex date.
    pub ex_date: NaiveDate,
    /// Record date.
    #[serde(default)]
    pub record_date: Option<NaiveDate>,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
}

/// A unit split.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitSplit {
    /// Corporate action id.
    pub id: String,
    /// Old symbol.
    pub old_symbol: String,
    /// Old CUSIP.
    pub old_cusip: String,
    /// Old rate.
    pub old_rate: f64,
    /// New symbol.
    pub new_symbol: String,
    /// New CUSIP.
    pub new_cusip: String,
    /// New rate.
    pub new_rate: f64,
    /// Alternate symbol.
    pub alternate_symbol: String,
    /// Alternate CUSIP.
    pub alternate_cusip: String,
    /// Alternate rate.
    pub alternate_rate: f64,
    /// Process date.
    pub process_date: NaiveDate,
    /// Effective date.
    pub effective_date: NaiveDate,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
}

/// A stock dividend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockDividend {
    /// Corporate action id.
    pub id: String,
    /// Symbol.
    pub symbol: String,
    /// CUSIP.
    pub cusip: String,
    /// Rate.
    pub rate: f64,
    /// Process date.
    pub process_date: NaiveDate,
    /// Ex date.
    pub ex_date: NaiveDate,
    /// Record date.
    #[serde(default)]
    pub record_date: Option<NaiveDate>,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
}

/// A cash dividend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CashDividend {
    /// Corporate action id.
    pub id: String,
    /// Symbol.
    pub symbol: String,
    /// CUSIP.
    pub cusip: String,
    /// Rate.
    pub rate: f64,
    /// Whether the dividend is special.
    pub special: bool,
    /// Whether the dividend is foreign.
    pub foreign: bool,
    /// Process date.
    pub process_date: NaiveDate,
    /// Ex date.
    pub ex_date: NaiveDate,
    /// Record date.
    #[serde(default)]
    pub record_date: Option<NaiveDate>,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Due bill on date.
    #[serde(default)]
    pub due_bill_on_date: Option<NaiveDate>,
    /// Due bill off date.
    #[serde(default)]
    pub due_bill_off_date: Option<NaiveDate>,
}

/// A spin off.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpinOff {
    /// Corporate action id.
    pub id: String,
    /// Source symbol.
    pub source_symbol: String,
    /// Source CUSIP.
    pub source_cusip: String,
    /// Source rate.
    pub source_rate: f64,
    /// New symbol.
    pub new_symbol: String,
    /// New CUSIP.
    pub new_cusip: String,
    /// New rate.
    pub new_rate: f64,
    /// Process date.
    pub process_date: NaiveDate,
    /// Ex date.
    pub ex_date: NaiveDate,
    /// Record date.
    #[serde(default)]
    pub record_date: Option<NaiveDate>,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Due bill redemption date.
    #[serde(default)]
    pub due_bill_redemption_date: Option<NaiveDate>,
}

/// A cash merger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CashMerger {
    /// Corporate action id.
    pub id: String,
    /// Acquirer symbol.
    #[serde(default)]
    pub acquirer_symbol: Option<String>,
    /// Acquirer CUSIP.
    #[serde(default)]
    pub acquirer_cusip: Option<String>,
    /// Acquiree symbol.
    pub acquiree_symbol: String,
    /// Acquiree CUSIP.
    pub acquiree_cusip: String,
    /// Rate.
    pub rate: f64,
    /// Process date.
    pub process_date: NaiveDate,
    /// Effective date.
    pub effective_date: NaiveDate,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
}

/// A stock merger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockMerger {
    /// Corporate action id.
    pub id: String,
    /// Acquirer symbol.
    pub acquirer_symbol: String,
    /// Acquirer CUSIP.
    pub acquirer_cusip: String,
    /// Acquirer rate.
    pub acquirer_rate: f64,
    /// Acquiree symbol.
    pub acquiree_symbol: String,
    /// Acquiree CUSIP.
    pub acquiree_cusip: String,
    /// Acquiree rate.
    pub acquiree_rate: f64,
    /// Process date.
    pub process_date: NaiveDate,
    /// Effective date.
    pub effective_date: NaiveDate,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
}

/// A stock and cash merger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockAndCashMerger {
    /// Corporate action id.
    pub id: String,
    /// Acquirer symbol.
    pub acquirer_symbol: String,
    /// Acquirer CUSIP.
    pub acquirer_cusip: String,
    /// Acquirer rate.
    pub acquirer_rate: f64,
    /// Acquiree symbol.
    pub acquiree_symbol: String,
    /// Acquiree CUSIP.
    pub acquiree_cusip: String,
    /// Acquiree rate.
    pub acquiree_rate: f64,
    /// Cash rate.
    pub cash_rate: f64,
    /// Process date.
    pub process_date: NaiveDate,
    /// Effective date.
    pub effective_date: NaiveDate,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
}

/// A redemption.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Redemption {
    /// Corporate action id.
    pub id: String,
    /// Symbol.
    pub symbol: String,
    /// CUSIP.
    pub cusip: String,
    /// Rate.
    pub rate: f64,
    /// Process date.
    pub process_date: NaiveDate,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
}

/// A name change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NameChange {
    /// Corporate action id.
    pub id: String,
    /// Old symbol.
    pub old_symbol: String,
    /// Old CUSIP.
    pub old_cusip: String,
    /// New symbol.
    pub new_symbol: String,
    /// New CUSIP.
    pub new_cusip: String,
    /// Process date.
    pub process_date: NaiveDate,
}

/// A worthless removal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorthlessRemoval {
    /// Corporate action id.
    pub id: String,
    /// Symbol.
    pub symbol: String,
    /// CUSIP.
    pub cusip: String,
    /// Process date.
    pub process_date: NaiveDate,
}

/// A rights distribution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RightsDistribution {
    /// Corporate action id.
    pub id: String,
    /// Source symbol.
    pub source_symbol: String,
    /// Source CUSIP.
    pub source_cusip: String,
    /// New symbol.
    pub new_symbol: String,
    /// New CUSIP.
    pub new_cusip: String,
    /// Rate.
    pub rate: f64,
    /// Process date.
    pub process_date: NaiveDate,
    /// Ex date.
    pub ex_date: NaiveDate,
    /// Record date.
    #[serde(default)]
    pub record_date: Option<NaiveDate>,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Expiration date.
    #[serde(default)]
    pub expiration_date: Option<NaiveDate>,
}

/// Corporate actions grouped by type, as returned in the
/// `corporate_actions` object of the response. Missing groups deserialize
/// as empty lists.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CorporateActions {
    /// Reverse splits.
    #[serde(default)]
    pub reverse_splits: Vec<ReverseSplit>,
    /// Forward splits.
    #[serde(default)]
    pub forward_splits: Vec<ForwardSplit>,
    /// Unit splits.
    #[serde(default)]
    pub unit_splits: Vec<UnitSplit>,
    /// Cash dividends.
    #[serde(default)]
    pub cash_dividends: Vec<CashDividend>,
    /// Stock dividends.
    #[serde(default)]
    pub stock_dividends: Vec<StockDividend>,
    /// Spin offs.
    #[serde(default)]
    pub spin_offs: Vec<SpinOff>,
    /// Cash mergers.
    #[serde(default)]
    pub cash_mergers: Vec<CashMerger>,
    /// Stock mergers.
    #[serde(default)]
    pub stock_mergers: Vec<StockMerger>,
    /// Stock and cash mergers.
    #[serde(default)]
    pub stock_and_cash_mergers: Vec<StockAndCashMerger>,
    /// Redemptions.
    #[serde(default)]
    pub redemptions: Vec<Redemption>,
    /// Name changes.
    #[serde(default)]
    pub name_changes: Vec<NameChange>,
    /// Worthless removals.
    #[serde(default)]
    pub worthless_removals: Vec<WorthlessRemoval>,
    /// Rights distributions.
    #[serde(default)]
    pub rights_distributions: Vec<RightsDistribution>,
}

/// Paginated corporate actions response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorporateActionsResponse {
    /// The corporate actions, grouped by type.
    pub corporate_actions: CorporateActions,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

/// A forex rate for a currency pair, either at the end of a timeframe
/// (historical) or as of now (latest).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForexRate {
    /// Rate timestamp (RFC3339).
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,
    /// Ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,
    /// Mid price.
    #[serde(rename = "mp")]
    pub mid_price: f64,
}

/// Paginated per-currency-pair historical forex rates response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForexRatesResponse {
    /// Rates keyed by currency pair (e.g. `USDJPY`).
    pub rates: HashMap<String, Vec<ForexRate>>,
    /// Token for the next page, if any.
    pub next_page_token: Option<String>,
}

/// Latest forex rate per currency pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatestForexRatesResponse {
    /// Latest rate keyed by currency pair (e.g. `USDJPY`).
    pub rates: HashMap<String, ForexRate>,
}

/// A company or crypto logo image.
///
/// The logos endpoint answers with the raw image (PNG, 300x300) rather than
/// JSON, so the payload is kept as bytes alongside the `Content-Type` the
/// server reported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Logo {
    /// Raw image bytes.
    pub bytes: Vec<u8>,
    /// The response's `Content-Type` header (e.g. `image/png`), if it sent
    /// one.
    pub content_type: Option<String>,
}

/// Extends per-symbol vectors with the next page's entries.
fn merge_maps<T>(map: &mut HashMap<String, Vec<T>>, next: HashMap<String, Vec<T>>) {
    for (symbol, items) in next {
        map.entry(symbol).or_default().extend(items);
    }
}

impl PaginatedResponse for BarsResponse {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    fn merge(&mut self, next: Self) {
        merge_maps(&mut self.bars, next.bars);
        self.next_page_token = next.next_page_token;
    }
}

impl PaginatedResponse for TradesResponse {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    fn merge(&mut self, next: Self) {
        merge_maps(&mut self.trades, next.trades);
        self.next_page_token = next.next_page_token;
    }
}

impl PaginatedResponse for QuotesResponse {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    fn merge(&mut self, next: Self) {
        merge_maps(&mut self.quotes, next.quotes);
        self.next_page_token = next.next_page_token;
    }
}

impl PaginatedResponse for AuctionsResponse {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    fn merge(&mut self, next: Self) {
        merge_maps(&mut self.auctions, next.auctions);
        self.next_page_token = next.next_page_token;
    }
}

impl PaginatedResponse for SymbolBarsResponse {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    fn merge(&mut self, next: Self) {
        self.bars.extend(next.bars);
        self.next_page_token = next.next_page_token;
    }
}

impl PaginatedResponse for SymbolTradesResponse {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    fn merge(&mut self, next: Self) {
        self.trades.extend(next.trades);
        self.next_page_token = next.next_page_token;
    }
}

impl PaginatedResponse for SymbolQuotesResponse {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    fn merge(&mut self, next: Self) {
        self.quotes.extend(next.quotes);
        self.next_page_token = next.next_page_token;
    }
}

impl PaginatedResponse for SymbolAuctionsResponse {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    fn merge(&mut self, next: Self) {
        self.auctions.extend(next.auctions);
        self.next_page_token = next.next_page_token;
    }
}

impl PaginatedResponse for SnapshotsResponse {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    fn merge(&mut self, next: Self) {
        self.snapshots.extend(next.snapshots);
        self.next_page_token = next.next_page_token;
    }
}

impl PaginatedResponse for OrderbooksResponse {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    fn merge(&mut self, next: Self) {
        self.orderbooks.extend(next.orderbooks);
        self.next_page_token = next.next_page_token;
    }
}

impl PaginatedResponse for NewsResponse {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    fn merge(&mut self, next: Self) {
        self.news.extend(next.news);
        self.next_page_token = next.next_page_token;
    }
}

impl PaginatedResponse for CorporateActionsResponse {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    fn merge(&mut self, next: Self) {
        let actions = &mut self.corporate_actions;
        let next_actions = next.corporate_actions;
        actions.reverse_splits.extend(next_actions.reverse_splits);
        actions.forward_splits.extend(next_actions.forward_splits);
        actions.unit_splits.extend(next_actions.unit_splits);
        actions.cash_dividends.extend(next_actions.cash_dividends);
        actions.stock_dividends.extend(next_actions.stock_dividends);
        actions.spin_offs.extend(next_actions.spin_offs);
        actions.cash_mergers.extend(next_actions.cash_mergers);
        actions.stock_mergers.extend(next_actions.stock_mergers);
        actions
            .stock_and_cash_mergers
            .extend(next_actions.stock_and_cash_mergers);
        actions.redemptions.extend(next_actions.redemptions);
        actions.name_changes.extend(next_actions.name_changes);
        actions
            .worthless_removals
            .extend(next_actions.worthless_removals);
        actions
            .rights_distributions
            .extend(next_actions.rights_distributions);
        self.next_page_token = next.next_page_token;
    }
}

impl PaginatedResponse for ForexRatesResponse {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    fn merge(&mut self, next: Self) {
        merge_maps(&mut self.rates, next.rates);
        self.next_page_token = next.next_page_token;
    }
}

/// Option exchange code → name mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionExchangesResponse {
    /// Exchange code to exchange name.
    pub exchanges: HashMap<String, String>,
}

/// Condition code → description table, as returned by the
/// `meta/conditions/{ticktype}` endpoints. The response body is the table
/// itself, not a wrapper object.
pub type ConditionsResponse = HashMap<String, String>;

/// Exchange code → exchange name table, as returned by
/// `GET /v2/stocks/meta/exchanges`. The response body is the table itself,
/// not a wrapper object.
pub type ExchangesResponse = HashMap<String, String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_bars_response() {
        let json = r#"{
            "bars": {
                "AAPL": [
                    {"t": "2024-07-24T07:56:00Z", "o": 132.65, "h": 136.0,
                     "l": 132.12, "c": 134.65, "v": 205, "n": 16, "vw": 133.7}
                ]
            },
            "next_page_token": null
        }"#;
        let resp: BarsResponse = serde_json::from_str(json).unwrap();
        let bars = &resp.bars["AAPL"];
        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].trade_count, Some(16));
        assert_eq!(bars[0].vwap, Some(133.7));
        assert_eq!(resp.next_page_token, None);
    }

    #[test]
    fn deserialize_forex_rates_response() {
        let json = r#"{
            "rates": {
                "USDJPY": [
                    {"t": "2024-07-24T00:00:00Z", "bp": 153.7, "ap": 153.9, "mp": 153.8}
                ]
            },
            "next_page_token": "next"
        }"#;
        let resp: ForexRatesResponse = serde_json::from_str(json).unwrap();
        let rates = &resp.rates["USDJPY"];
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].bid_price, 153.7);
        assert_eq!(rates[0].ask_price, 153.9);
        assert_eq!(rates[0].mid_price, 153.8);
        assert_eq!(resp.next_page_token.as_deref(), Some("next"));
    }

    #[test]
    fn deserialize_latest_forex_rates_response() {
        let json = r#"{
            "rates": {
                "USDMXN": {"t": "2024-07-24T12:00:00Z", "bp": 18.1, "ap": 18.3, "mp": 18.2}
            }
        }"#;
        let resp: LatestForexRatesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.rates["USDMXN"].mid_price, 18.2);
    }

    fn forex_rate(mid: f64) -> ForexRate {
        serde_json::from_value(serde_json::json!({
            "t": "2024-07-24T00:00:00Z", "bp": mid, "ap": mid, "mp": mid
        }))
        .unwrap()
    }

    #[test]
    fn forex_rates_response_merge_extends_per_pair_vecs() {
        let mut first = ForexRatesResponse {
            rates: HashMap::from([("USDJPY".to_string(), vec![forex_rate(1.0)])]),
            next_page_token: Some("t1".to_string()),
        };
        let second = ForexRatesResponse {
            rates: HashMap::from([
                ("USDJPY".to_string(), vec![forex_rate(2.0)]),
                ("USDMXN".to_string(), vec![forex_rate(3.0)]),
            ]),
            next_page_token: None,
        };
        first.merge(second);
        let mids: Vec<f64> = first.rates["USDJPY"].iter().map(|r| r.mid_price).collect();
        assert_eq!(mids, vec![1.0, 2.0]);
        assert_eq!(first.rates["USDMXN"].len(), 1);
        assert_eq!(first.next_page_token, None);
    }

    #[test]
    fn deserialize_snapshot() {
        let json = r#"{
            "latestTrade": {"t": "2024-07-24T07:56:53.639Z", "p": 133.85,
                            "s": 4, "x": "Q", "i": 52983525029461, "c": ["@"], "z": "C"},
            "latestQuote": {"t": "2024-07-24T07:56:53.639Z", "bp": 133.85, "bs": 4,
                            "ap": 135.77, "as": 5, "bx": "O", "ax": "R", "c": ["R"], "z": "A"},
            "minuteBar": {"t": "2024-07-24T07:56:00Z", "o": 132.65, "h": 136.0,
                          "l": 132.12, "c": 134.65, "v": 205, "n": 16, "vw": 133.7},
            "dailyBar": {"t": "2024-07-24T04:00:00Z", "o": 130.0, "h": 137.0,
                         "l": 129.5, "c": 134.65, "v": 1205, "n": 160, "vw": 133.1},
            "prevDailyBar": {"t": "2024-07-23T04:00:00Z", "o": 128.0, "h": 131.0,
                             "l": 127.5, "c": 130.5, "v": 1100, "n": 150, "vw": 129.7}
        }"#;
        let snap: Snapshot = serde_json::from_str(json).unwrap();
        assert_eq!(
            snap.latest_trade.unwrap().id,
            Some(TradeId::Int(52983525029461))
        );
        assert_eq!(snap.latest_quote.unwrap().bid_price, 133.85);
        assert!(snap.minute_bar.is_some());
    }

    fn bar(close: f64) -> Bar {
        serde_json::from_value(serde_json::json!({
            "t": "2024-07-24T07:56:00Z", "o": close, "h": close,
            "l": close, "c": close, "v": 1
        }))
        .unwrap()
    }

    #[test]
    fn bars_response_merge_extends_per_symbol_vecs() {
        let mut first = BarsResponse {
            bars: HashMap::from([("AAPL".to_string(), vec![bar(1.0)])]),
            next_page_token: Some("t1".to_string()),
        };
        let second = BarsResponse {
            bars: HashMap::from([
                ("AAPL".to_string(), vec![bar(2.0), bar(3.0)]),
                ("MSFT".to_string(), vec![bar(4.0)]),
            ]),
            next_page_token: None,
        };
        first.merge(second);
        let closes: Vec<f64> = first.bars["AAPL"].iter().map(|b| b.close).collect();
        assert_eq!(closes, vec![1.0, 2.0, 3.0]);
        assert_eq!(first.bars["MSFT"].len(), 1);
        assert_eq!(first.next_page_token, None);
    }

    #[test]
    fn snapshots_response_merge_extends_symbol_map() {
        let snapshot: Snapshot = serde_json::from_str("{}").unwrap();
        let mut first = SnapshotsResponse {
            snapshots: HashMap::from([("AAPL".to_string(), snapshot.clone())]),
            next_page_token: Some("t1".to_string()),
        };
        let second = SnapshotsResponse {
            snapshots: HashMap::from([("MSFT".to_string(), snapshot)]),
            next_page_token: None,
        };
        first.merge(second);
        assert_eq!(first.snapshots.len(), 2);
        assert_eq!(first.next_page_token, None);
    }

    #[test]
    fn news_response_merge_extends_articles() {
        let article: NewsArticle = serde_json::from_value(serde_json::json!({
            "id": 1, "headline": "h", "created_at": "2024-07-24T07:56:00Z",
            "source": "test"
        }))
        .unwrap();
        let mut first = NewsResponse {
            news: vec![article.clone()],
            next_page_token: Some("t1".to_string()),
        };
        let second = NewsResponse {
            news: vec![article.clone(), article],
            next_page_token: Some("t2".to_string()),
        };
        first.merge(second);
        assert_eq!(first.news.len(), 3);
        assert_eq!(first.next_page_token.as_deref(), Some("t2"));
    }

    #[test]
    fn deserialize_most_actives() {
        let json = r#"{
            "most_actives": [
                {"symbol": "AAPL", "volume": 70451379.0, "trade_count": 486549.0},
                {"symbol": "TSLA", "volume": 48613142.0, "trade_count": 321456.0}
            ],
            "last_updated": "2023-06-23T14:13:00.163514169Z"
        }"#;
        let resp: MostActives = serde_json::from_str(json).unwrap();
        assert_eq!(resp.most_actives.len(), 2);
        assert_eq!(resp.most_actives[0].symbol, "AAPL");
        assert_eq!(resp.most_actives[0].trade_count, 486549.0);
        // Round-trip.
        let serialized = serde_json::to_string(&resp).unwrap();
        let again: MostActives = serde_json::from_str(&serialized).unwrap();
        assert_eq!(resp, again);
    }

    #[test]
    fn deserialize_movers() {
        let json = r#"{
            "gainers": [
                {"change": 59.75, "percent_change": 1832.9, "price": 63.0125, "symbol": "BOIL"}
            ],
            "losers": [
                {"change": -0.0706, "percent_change": -95.02, "price": 0.0037, "symbol": "FMIVW"}
            ],
            "market_type": "stocks",
            "last_updated": "2023-06-23T14:13:00.163514169Z"
        }"#;
        let resp: Movers = serde_json::from_str(json).unwrap();
        assert_eq!(resp.market_type, MarketType::Stocks);
        assert_eq!(resp.gainers[0].symbol, "BOIL");
        assert_eq!(resp.losers[0].percent_change, -95.02);
        // Round-trip.
        let serialized = serde_json::to_string(&resp).unwrap();
        let again: Movers = serde_json::from_str(&serialized).unwrap();
        assert_eq!(resp, again);
    }

    /// A representative slice of the response shape used by the
    /// `GET /v1/corporate-actions` endpoint (mirrors alpaca-py's fixtures).
    const CORPORATE_ACTIONS_JSON: &str = r#"{
        "corporate_actions": {
            "reverse_splits": [
                {
                    "id": "a7932b15-f816-4f83-921e-998b524487b0",
                    "ex_date": "2023-08-24",
                    "new_rate": 1,
                    "old_rate": 50,
                    "new_cusip": "NEWCUSIP1",
                    "old_cusip": "OLDCUSIP1",
                    "process_date": "2023-08-24",
                    "record_date": "2023-08-24",
                    "symbol": "MNTS"
                }
            ],
            "forward_splits": [
                {
                    "cusip": "CUSIP1",
                    "id": "a7932b15-f816-4f83-921e-998b524487b1",
                    "due_bill_redemption_date": "2023-08-23",
                    "ex_date": "2023-08-22",
                    "new_rate": 2,
                    "old_rate": 1,
                    "payable_date": "2023-08-21",
                    "process_date": "2023-08-22",
                    "record_date": "2023-08-14",
                    "symbol": "SRE"
                }
            ],
            "cash_dividends": [
                {
                    "cusip": "CUSIP1",
                    "id": "a7932b15-f816-4f83-921e-998b524487b4",
                    "ex_date": "2023-05-04",
                    "foreign": false,
                    "payable_date": "2023-05-19",
                    "process_date": "2023-05-19",
                    "rate": 0.125,
                    "record_date": "2023-05-05",
                    "special": false,
                    "symbol": "FCF"
                }
            ],
            "cash_mergers": [
                {
                    "id": "a7932b15-f816-4f83-921e-998b524487b6",
                    "acquiree_symbol": "GLOP",
                    "acquiree_cusip": "ACQUIREECUSIP1",
                    "effective_date": "2023-07-17",
                    "payable_date": "2023-07-17",
                    "process_date": "2023-07-17",
                    "rate": 5.37
                }
            ],
            "name_changes": [
                {
                    "id": "a7932b15-f816-4f83-921e-998b524487c0",
                    "new_cusip": "NEWCUSIP1",
                    "new_symbol": "VFS",
                    "old_cusip": "OLDCUSIP1",
                    "old_symbol": "BSAQ",
                    "process_date": "2023-08-15"
                }
            ],
            "rights_distributions": [
                {
                    "id": "a7932b15-f816-4f83-921e-998b524487c2",
                    "source_cusip": "SOURCECUSIP1",
                    "source_symbol": "IFN",
                    "new_cusip": "NEWCUSIP1",
                    "new_symbol": "IFN.RTWI",
                    "rate": 1,
                    "ex_date": "2024-04-17",
                    "record_date": "2024-04-18",
                    "payable_date": "2024-04-19",
                    "process_date": "2024-04-19",
                    "expiration_date": "2024-05-14"
                }
            ]
        },
        "next_page_token": "token-1"
    }"#;

    #[test]
    fn deserialize_corporate_actions_response() {
        let resp: CorporateActionsResponse = serde_json::from_str(CORPORATE_ACTIONS_JSON).unwrap();
        let actions = &resp.corporate_actions;
        assert_eq!(resp.next_page_token.as_deref(), Some("token-1"));

        let reverse = &actions.reverse_splits[0];
        assert_eq!(reverse.symbol, "MNTS");
        assert_eq!(reverse.new_cusip, "NEWCUSIP1");
        assert_eq!(reverse.old_rate, 50.0);
        assert!(reverse.payable_date.is_none());

        let forward = &actions.forward_splits[0];
        assert_eq!(forward.symbol, "SRE");
        assert_eq!(
            forward.due_bill_redemption_date,
            Some(NaiveDate::from_ymd_opt(2023, 8, 23).unwrap())
        );

        let dividend = &actions.cash_dividends[0];
        assert!(!dividend.special);
        assert!(!dividend.foreign);
        assert!(dividend.due_bill_on_date.is_none());

        let merger = &actions.cash_mergers[0];
        assert_eq!(merger.acquiree_symbol, "GLOP");
        assert!(merger.acquirer_symbol.is_none());

        let name_change = &actions.name_changes[0];
        assert_eq!(name_change.old_symbol, "BSAQ");

        let rights = &actions.rights_distributions[0];
        assert_eq!(rights.new_symbol, "IFN.RTWI");
        assert_eq!(
            rights.expiration_date,
            Some(NaiveDate::from_ymd_opt(2024, 5, 14).unwrap())
        );

        // Groups absent from the payload deserialize as empty.
        assert!(actions.unit_splits.is_empty());
        assert!(actions.spin_offs.is_empty());
        assert!(actions.worthless_removals.is_empty());

        // Round-trip.
        let serialized = serde_json::to_string(&resp).unwrap();
        let again: CorporateActionsResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(resp, again);
    }

    #[test]
    fn corporate_actions_response_merge_extends_all_groups() {
        let mut first: CorporateActionsResponse =
            serde_json::from_str(CORPORATE_ACTIONS_JSON).unwrap();
        let second: CorporateActionsResponse = serde_json::from_value(serde_json::json!({
            "corporate_actions": {
                "cash_dividends": [
                    {
                        "cusip": "CUSIP1",
                        "id": "a7932b15-f816-4f83-921e-998b524487c3",
                        "ex_date": "2024-08-07",
                        "foreign": true,
                        "payable_date": "2024-08-26",
                        "process_date": "2024-08-26",
                        "rate": 0.086928,
                        "record_date": "2024-08-07",
                        "special": false,
                        "symbol": "ZMTBY"
                    }
                ],
                "redemptions": [
                    {
                        "id": "a7932b15-f816-4f83-921e-998b524487b9",
                        "payable_date": "2023-06-13",
                        "process_date": "2023-06-13",
                        "cusip": "CUSIP1",
                        "rate": 0.141134,
                        "symbol": "ORPHY"
                    }
                ]
            },
            "next_page_token": null
        }))
        .unwrap();
        first.merge(second);
        assert_eq!(first.corporate_actions.cash_dividends.len(), 2);
        assert_eq!(first.corporate_actions.cash_dividends[1].symbol, "ZMTBY");
        assert_eq!(first.corporate_actions.redemptions.len(), 1);
        assert_eq!(first.corporate_actions.reverse_splits.len(), 1);
        assert_eq!(first.next_page_token, None);
    }

    #[test]
    fn deserialize_auctions_response() {
        let json = r#"{
            "auctions": {
                "AAPL": [
                    {
                        "d": "2024-07-24",
                        "o": [
                            {"t": "2024-07-24T13:30:00.048Z", "x": "P", "p": 224.0,
                             "s": 1000, "c": "Q"}
                        ],
                        "c": [
                            {"t": "2024-07-24T20:00:00.106Z", "x": "P", "p": 224.62,
                             "s": 2000, "c": "M"},
                            {"t": "2024-07-24T20:00:00.106Z", "x": "V", "p": 224.62,
                             "c": "6"}
                        ]
                    }
                ]
            },
            "next_page_token": "token-1"
        }"#;
        let resp: AuctionsResponse = serde_json::from_str(json).unwrap();
        let days = &resp.auctions["AAPL"];
        assert_eq!(days.len(), 1);
        assert_eq!(days[0].date, NaiveDate::from_ymd_opt(2024, 7, 24).unwrap());
        assert_eq!(days[0].opening.len(), 1);
        assert_eq!(days[0].opening[0].price, 224.0);
        assert_eq!(days[0].opening[0].size, Some(1000.0));
        assert_eq!(days[0].opening[0].condition.as_deref(), Some("Q"));
        assert_eq!(days[0].closing.len(), 2);
        // Size is optional: not every venue reports it.
        assert_eq!(days[0].closing[1].size, None);
        assert_eq!(resp.next_page_token.as_deref(), Some("token-1"));

        // Round-trip.
        let serialized = serde_json::to_string(&resp).unwrap();
        let again: AuctionsResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(resp, again);
    }

    fn auction(date: &str, close: f64) -> Auction {
        serde_json::from_value(serde_json::json!({
            "d": date,
            "c": [{"t": "2024-07-24T20:00:00.106Z", "x": "P", "p": close, "c": "M"}]
        }))
        .unwrap()
    }

    #[test]
    fn auctions_response_merge_extends_per_symbol_vecs() {
        let mut first = AuctionsResponse {
            auctions: HashMap::from([("AAPL".to_string(), vec![auction("2024-07-23", 223.9)])]),
            next_page_token: Some("t1".to_string()),
        };
        let second = AuctionsResponse {
            auctions: HashMap::from([
                ("AAPL".to_string(), vec![auction("2024-07-24", 224.62)]),
                ("MSFT".to_string(), vec![auction("2024-07-24", 425.0)]),
            ]),
            next_page_token: None,
        };
        first.merge(second);
        assert_eq!(first.auctions["AAPL"].len(), 2);
        assert_eq!(first.auctions["MSFT"].len(), 1);
        // Missing `o` deserializes as an empty list.
        assert!(first.auctions["AAPL"][0].opening.is_empty());
        assert_eq!(first.next_page_token, None);
    }

    #[test]
    fn deserialize_symbol_bars_response() {
        let json = r#"{
            "symbol": "AAPL",
            "bars": [
                {"t": "2024-07-24T13:30:00Z", "o": 224.0, "h": 225.0, "l": 223.5,
                 "c": 224.62, "v": 1000, "n": 42, "vw": 224.3}
            ],
            "next_page_token": null
        }"#;
        let resp: SymbolBarsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.symbol, "AAPL");
        assert_eq!(resp.bars.len(), 1);
        assert_eq!(resp.bars[0].close, 224.62);

        // Round-trip.
        let serialized = serde_json::to_string(&resp).unwrap();
        let again: SymbolBarsResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(resp, again);
    }

    #[test]
    fn single_symbol_responses_accept_null_lists() {
        let bars: SymbolBarsResponse =
            serde_json::from_str(r#"{"symbol": "AAPL", "bars": null, "next_page_token": null}"#)
                .unwrap();
        assert!(bars.bars.is_empty());

        let trades: SymbolTradesResponse =
            serde_json::from_str(r#"{"symbol": "AAPL", "trades": null, "next_page_token": null}"#)
                .unwrap();
        assert!(trades.trades.is_empty());

        let quotes: SymbolQuotesResponse =
            serde_json::from_str(r#"{"symbol": "AAPL", "quotes": null, "next_page_token": null}"#)
                .unwrap();
        assert!(quotes.quotes.is_empty());

        let auctions: SymbolAuctionsResponse = serde_json::from_str(
            r#"{"symbol": "AAPL", "auctions": null, "next_page_token": null}"#,
        )
        .unwrap();
        assert!(auctions.auctions.is_empty());
    }

    #[test]
    fn symbol_bars_response_merge_extends_list() {
        let mut first = SymbolBarsResponse {
            symbol: "AAPL".to_string(),
            bars: vec![bar(1.0)],
            next_page_token: Some("t1".to_string()),
        };
        let second = SymbolBarsResponse {
            symbol: "AAPL".to_string(),
            bars: vec![bar(2.0), bar(3.0)],
            next_page_token: None,
        };
        first.merge(second);
        let closes: Vec<f64> = first.bars.iter().map(|b| b.close).collect();
        assert_eq!(closes, vec![1.0, 2.0, 3.0]);
        assert_eq!(first.next_page_token, None);
    }

    #[test]
    fn deserialize_symbol_latest_and_snapshot_responses() {
        let trade: SymbolTradeResponse = serde_json::from_str(
            r#"{"symbol": "AAPL",
                "trade": {"t": "2024-07-24T19:59:59.639Z", "p": 224.62, "s": 4,
                          "x": "Q", "i": 52983525029461, "c": ["@"], "z": "C"}}"#,
        )
        .unwrap();
        assert_eq!(trade.symbol, "AAPL");
        assert_eq!(trade.trade.price, 224.62);

        let quote: SymbolQuoteResponse = serde_json::from_str(
            r#"{"symbol": "AAPL",
                "quote": {"t": "2024-07-24T19:59:59.639Z", "bp": 224.6, "bs": 3,
                          "ap": 224.65, "as": 5, "bx": "P", "ax": "Q", "z": "C"}}"#,
        )
        .unwrap();
        assert_eq!(quote.quote.ask_price, 224.65);

        let bar: SymbolBarResponse = serde_json::from_str(
            r#"{"symbol": "AAPL",
                "bar": {"t": "2024-07-24T19:59:00Z", "o": 224.0, "h": 225.0,
                        "l": 223.5, "c": 224.62, "v": 1000}}"#,
        )
        .unwrap();
        assert_eq!(bar.bar.close, 224.62);

        // The snapshot fields sit next to `symbol` at the top level.
        let snapshot: SymbolSnapshot = serde_json::from_str(
            r#"{"symbol": "AAPL", "currency": "USD",
                "dailyBar": {"t": "2024-07-24T04:00:00Z", "o": 224.0, "h": 225.0,
                             "l": 223.5, "c": 224.62, "v": 1000}}"#,
        )
        .unwrap();
        assert_eq!(snapshot.symbol, "AAPL");
        assert_eq!(
            snapshot.snapshot.daily_bar.as_ref().map(|b| b.close),
            Some(224.62)
        );
        assert!(snapshot.snapshot.latest_trade.is_none());

        // Round-trip.
        let serialized = serde_json::to_string(&snapshot).unwrap();
        let again: SymbolSnapshot = serde_json::from_str(&serialized).unwrap();
        assert_eq!(snapshot, again);
    }

    #[test]
    fn deserialize_meta_tables() {
        let conditions: ConditionsResponse =
            serde_json::from_str(r#"{"@": "Regular Sale", "A": "Acquisition"}"#).unwrap();
        assert_eq!(conditions["@"], "Regular Sale");
        assert_eq!(conditions.len(), 2);

        let exchanges: ExchangesResponse =
            serde_json::from_str(r#"{"N": "New York Stock Exchange", "V": "IEX"}"#).unwrap();
        assert_eq!(exchanges["V"], "IEX");
    }
}
