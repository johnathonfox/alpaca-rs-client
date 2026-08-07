//! The `ca` payloads carried by [corporate action
//! events](super::ca_events::CorporateActionEvent).
//!
//! Each struct mirrors the matching group of the REST
//! [`CorporateActions`](crate::data::CorporateActions) response — the same
//! underlying records reach you either way — with two wire-level
//! differences, both dictated by the events schema:
//!
//! - Every decimal is sent as a JSON string to preserve precision, so rates
//!   and prices are [`Decimal`] here where the REST models use `f64`.
//! - Optional fields are omitted rather than sent as `null`, so they are
//!   `#[serde(default)] Option<_>` here. The fields left non-`Option` are the
//!   ones the events schema marks required for that payload type — `id` and
//!   `process_date` on every payload, plus each type's identifying core
//!   (`symbol`/`cusip`, its rate or ratio, and `ex_date`). If Alpaca ever
//!   omits one of those, decoding that payload fails rather than silently
//!   yielding a half-populated record.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Sub-classification of a [`CashDividendEvent`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CashDividendSubType {
    /// The distribution is interest rather than a dividend.
    Interest,
    /// The distribution is a return of capital.
    ReturnOfCapital,
    /// A sub-type added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

/// How the called shares of an [`EquityPartialCallEvent`] were allocated.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LotteryType {
    /// The original lottery drawing.
    Original,
    /// A supplemental drawing on top of the original one.
    Supplemental,
    /// A lottery type added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

/// A cash dividend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CashDividendEvent {
    /// Corporate action id, stable across `insert`/`update`/`delete`.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol paying the dividend.
    pub symbol: String,
    /// CUSIP.
    pub cusip: String,
    /// ISIN.
    #[serde(default)]
    pub isin: Option<String>,
    /// Cash paid per share.
    pub rate: Decimal,
    /// Whether this is a one-off dividend outside the regular schedule.
    pub special: bool,
    /// Whether the issuer is not US-based.
    pub foreign: bool,
    /// Ex date.
    pub ex_date: NaiveDate,
    /// Record date.
    #[serde(default)]
    pub record_date: Option<NaiveDate>,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Start of the due-bill period.
    #[serde(default)]
    pub due_bill_on_date: Option<NaiveDate>,
    /// End of the due-bill period.
    #[serde(default)]
    pub due_bill_off_date: Option<NaiveDate>,
    /// Optional sub-classification of the distribution.
    #[serde(default)]
    pub sub_type: Option<CashDividendSubType>,
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A stock dividend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockDividendEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol paying the dividend.
    pub symbol: String,
    /// CUSIP.
    pub cusip: String,
    /// ISIN.
    #[serde(default)]
    pub isin: Option<String>,
    /// Shares distributed per share held.
    pub rate: Decimal,
    /// Ex date.
    pub ex_date: NaiveDate,
    /// Record date.
    #[serde(default)]
    pub record_date: Option<NaiveDate>,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A forward stock split.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForwardSplitEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol being split.
    pub symbol: String,
    /// CUSIP.
    pub cusip: String,
    /// ISIN.
    #[serde(default)]
    pub isin: Option<String>,
    /// Shares before the split.
    pub old_rate: Decimal,
    /// Shares after the split.
    pub new_rate: Decimal,
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
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A reverse stock split.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReverseSplitEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol being split.
    pub symbol: String,
    /// CUSIP before the split.
    pub old_cusip: String,
    /// ISIN before the split.
    #[serde(default)]
    pub old_isin: Option<String>,
    /// Symbol after the split, when it changes.
    #[serde(default)]
    pub new_symbol: Option<String>,
    /// CUSIP after the split.
    pub new_cusip: String,
    /// ISIN after the split.
    #[serde(default)]
    pub new_isin: Option<String>,
    /// Shares before the split.
    pub old_rate: Decimal,
    /// Shares after the split.
    pub new_rate: Decimal,
    /// Ex date.
    pub ex_date: NaiveDate,
    /// Record date.
    #[serde(default)]
    pub record_date: Option<NaiveDate>,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A unit split: a unit is separated into its component securities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitSplitEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol of the unit being split.
    pub old_symbol: String,
    /// CUSIP of the unit being split.
    pub old_cusip: String,
    /// ISIN of the unit being split.
    #[serde(default)]
    pub old_isin: Option<String>,
    /// Units required for each `new_rate`.
    pub old_rate: Decimal,
    /// Symbol of the first component security.
    pub new_symbol: String,
    /// CUSIP of the first component security.
    pub new_cusip: String,
    /// ISIN of the first component security.
    #[serde(default)]
    pub new_isin: Option<String>,
    /// Shares of the first component security delivered.
    pub new_rate: Decimal,
    /// Symbol of the second component security.
    pub alternate_symbol: String,
    /// CUSIP of the second component security.
    pub alternate_cusip: String,
    /// ISIN of the second component security.
    #[serde(default)]
    pub alternate_isin: Option<String>,
    /// Shares of the second component security delivered.
    pub alternate_rate: Decimal,
    /// Effective date.
    pub effective_date: NaiveDate,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A spin off.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpinOffEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol of the parent company.
    pub source_symbol: String,
    /// CUSIP of the parent company.
    pub source_cusip: String,
    /// ISIN of the parent company.
    #[serde(default)]
    pub source_isin: Option<String>,
    /// Parent shares required for each `new_rate`.
    pub source_rate: Decimal,
    /// Symbol of the spun-off company.
    pub new_symbol: String,
    /// CUSIP of the spun-off company.
    pub new_cusip: String,
    /// ISIN of the spun-off company.
    #[serde(default)]
    pub new_isin: Option<String>,
    /// Shares of the spun-off company delivered.
    pub new_rate: Decimal,
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
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A cash merger: holders of the acquiree receive cash only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CashMergerEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol of the acquired company.
    pub acquiree_symbol: String,
    /// CUSIP of the acquired company.
    pub acquiree_cusip: String,
    /// ISIN of the acquired company.
    #[serde(default)]
    pub acquiree_isin: Option<String>,
    /// Symbol of the acquiring company, when there is one.
    #[serde(default)]
    pub acquirer_symbol: Option<String>,
    /// CUSIP of the acquiring company.
    #[serde(default)]
    pub acquirer_cusip: Option<String>,
    /// ISIN of the acquiring company.
    #[serde(default)]
    pub acquirer_isin: Option<String>,
    /// Cash paid per acquiree share.
    pub rate: Decimal,
    /// Effective date.
    pub effective_date: NaiveDate,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A stock merger: holders of the acquiree receive acquirer shares.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockMergerEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol of the acquired company.
    pub acquiree_symbol: String,
    /// CUSIP of the acquired company.
    pub acquiree_cusip: String,
    /// ISIN of the acquired company.
    #[serde(default)]
    pub acquiree_isin: Option<String>,
    /// Acquiree shares required for each `acquirer_rate`.
    pub acquiree_rate: Decimal,
    /// Symbol of the acquiring company.
    pub acquirer_symbol: String,
    /// CUSIP of the acquiring company.
    pub acquirer_cusip: String,
    /// ISIN of the acquiring company.
    #[serde(default)]
    pub acquirer_isin: Option<String>,
    /// Acquirer shares delivered per `acquiree_rate`.
    pub acquirer_rate: Decimal,
    /// Effective date.
    pub effective_date: NaiveDate,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A stock and cash merger: holders of the acquiree receive both.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockAndCashMergerEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol of the acquired company.
    pub acquiree_symbol: String,
    /// CUSIP of the acquired company.
    pub acquiree_cusip: String,
    /// ISIN of the acquired company.
    #[serde(default)]
    pub acquiree_isin: Option<String>,
    /// Acquiree shares required for each `acquirer_rate`.
    pub acquiree_rate: Decimal,
    /// Symbol of the acquiring company.
    pub acquirer_symbol: String,
    /// CUSIP of the acquiring company.
    pub acquirer_cusip: String,
    /// ISIN of the acquiring company.
    #[serde(default)]
    pub acquirer_isin: Option<String>,
    /// Acquirer shares delivered per `acquiree_rate`.
    pub acquirer_rate: Decimal,
    /// Cash paid per `acquiree_rate`.
    pub cash_rate: Decimal,
    /// Effective date.
    pub effective_date: NaiveDate,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A redemption: the issuer buys the whole issue back for cash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RedemptionEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol being redeemed.
    pub symbol: String,
    /// CUSIP.
    pub cusip: String,
    /// ISIN.
    #[serde(default)]
    pub isin: Option<String>,
    /// Cash paid per share.
    pub rate: Decimal,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A partial call: the issuer buys back only a fraction of the outstanding
/// shares, typically allocated to holders by lottery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquityPartialCallEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol being partially called.
    pub symbol: String,
    /// CUSIP.
    #[serde(default)]
    pub cusip: Option<String>,
    /// ISIN.
    #[serde(default)]
    pub isin: Option<String>,
    /// Price paid per called share.
    #[serde(default)]
    pub price: Option<Decimal>,
    /// Dividend rate associated with the called shares.
    #[serde(default)]
    pub dividend_rate: Option<Decimal>,
    /// When the lottery drawing happens.
    #[serde(default)]
    pub lottery_date: Option<NaiveDate>,
    /// How the called shares were allocated.
    #[serde(default)]
    pub lottery_type: Option<LotteryType>,
    /// When the lottery results are published.
    #[serde(default)]
    pub results_publication_date: Option<NaiveDate>,
    /// Record date.
    #[serde(default)]
    pub record_date: Option<NaiveDate>,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// One replacement-security leg of a [`ReorganizationEvent`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReorganizationStockMovement {
    /// Replacement symbol.
    pub symbol: String,
    /// Replacement CUSIP.
    pub cusip: String,
    /// Replacement ISIN.
    #[serde(default)]
    pub isin: Option<String>,
    /// Source shares required for each `new_rate`.
    pub source_rate: Decimal,
    /// Replacement shares delivered per `source_rate`.
    pub new_rate: Decimal,
}

/// A general corporate restructuring (e.g. emergence from Chapter 11) in
/// which holders receive cash and/or one or more replacement securities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReorganizationEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol undergoing the reorganization.
    pub symbol: String,
    /// CUSIP of the original security.
    pub cusip: String,
    /// ISIN of the original security.
    #[serde(default)]
    pub isin: Option<String>,
    /// Cash paid per source share.
    #[serde(default)]
    pub cash_rate: Option<Decimal>,
    /// One entry per replacement security delivered to holders.
    #[serde(default)]
    pub stock_movements: Vec<ReorganizationStockMovement>,
    /// Effective date.
    pub effective_date: NaiveDate,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A name and/or symbol change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NameChangeEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol before the change.
    pub old_symbol: String,
    /// CUSIP before the change.
    pub old_cusip: String,
    /// ISIN before the change.
    #[serde(default)]
    pub old_isin: Option<String>,
    /// Symbol after the change.
    pub new_symbol: String,
    /// CUSIP after the change.
    pub new_cusip: String,
    /// ISIN after the change.
    #[serde(default)]
    pub new_isin: Option<String>,
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A worthless removal: a security is written off the books.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorthlessRemovalEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol being removed.
    pub symbol: String,
    /// CUSIP.
    pub cusip: String,
    /// ISIN.
    #[serde(default)]
    pub isin: Option<String>,
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A rights distribution: holders receive subscription rights.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RightsDistributionEvent {
    /// Corporate action id.
    pub id: String,
    /// Process date.
    pub process_date: NaiveDate,
    /// Symbol the rights are distributed on.
    pub source_symbol: String,
    /// CUSIP the rights are distributed on.
    pub source_cusip: String,
    /// ISIN the rights are distributed on.
    #[serde(default)]
    pub source_isin: Option<String>,
    /// Symbol of the distributed right.
    pub new_symbol: String,
    /// CUSIP of the distributed right.
    pub new_cusip: String,
    /// ISIN of the distributed right.
    #[serde(default)]
    pub new_isin: Option<String>,
    /// Rights distributed per source share.
    pub rate: Decimal,
    /// Ex date.
    pub ex_date: NaiveDate,
    /// Record date.
    #[serde(default)]
    pub record_date: Option<NaiveDate>,
    /// Payable date.
    #[serde(default)]
    pub payable_date: Option<NaiveDate>,
    /// When the rights expire.
    #[serde(default)]
    pub expiration_date: Option<NaiveDate>,
    /// Currency code (ISO 4217).
    #[serde(default)]
    pub currency: Option<String>,
}
