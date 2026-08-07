//! Corporate action event stream
//! (`GET /v1beta1/events/corporate-actions`).
//!
//! One long-lived `text/event-stream` connection delivers every mutation
//! (`insert`, `update`, `delete`) applied to Alpaca's corporate actions store,
//! across all 15 action types. The same records are available on demand from
//! [`CorporateActionsClient`](crate::data::CorporateActionsClient); the stream
//! is how you learn about corrections without re-polling.
//!
//! The envelope's `event_type` selects the shape of the `ca` payload, which
//! this module resolves into a typed [`CorporateActionPayload`].

use chrono::{DateTime, Utc};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

use crate::error::Result;
use crate::rest::Credentials;
use crate::stream::ca_payloads::{
    CashDividendEvent, CashMergerEvent, EquityPartialCallEvent, ForwardSplitEvent, NameChangeEvent,
    RedemptionEvent, ReorganizationEvent, ReverseSplitEvent, RightsDistributionEvent, SpinOffEvent,
    StockAndCashMergerEvent, StockDividendEvent, StockMergerEvent, UnitSplitEvent,
    WorthlessRemovalEvent,
};
use crate::stream::events::{EventStream, validate_replay_window};

const DATA_BASE: &str = "https://data.alpaca.markets";
const CORPORATE_ACTIONS_PATH: &str = "/v1beta1/events/corporate-actions";

/// The kind of mutation that produced a [`CorporateActionEvent`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorporateActionMutation {
    /// A new corporate action was published.
    Insert,
    /// An existing corporate action was corrected; `ca.id` is unchanged.
    Update,
    /// A previously published corporate action was withdrawn; `ca.id` is
    /// unchanged and later events for that id are fresh inserts.
    Delete,
    /// A mutation added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

/// Market classification of a [`CorporateActionEvent`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorporateActionEventRegion {
    /// A US-listed / US-regulated corporate action.
    Us,
    /// Everything else.
    NonUs,
    /// A region added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

/// The `region` filter accepted by [`CorporateActionEventsRequest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorporateActionRegionFilter {
    /// Every event, regardless of market (the API default).
    All,
    /// Only events with `region == us`.
    Us,
    /// Only events with `region == non_us`.
    NonUs,
}

/// Discriminator selecting the shape of a [`CorporateActionEvent`]'s
/// `ca` payload.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorporateActionEventType {
    /// A [`CashDividendEvent`].
    #[serde(rename = "cash_dividend_corporateaction_event")]
    CashDividend,
    /// A [`StockDividendEvent`].
    #[serde(rename = "stock_dividend_corporateaction_event")]
    StockDividend,
    /// A [`ForwardSplitEvent`].
    #[serde(rename = "forward_split_corporateaction_event")]
    ForwardSplit,
    /// A [`ReverseSplitEvent`].
    #[serde(rename = "reverse_split_corporateaction_event")]
    ReverseSplit,
    /// A [`UnitSplitEvent`].
    #[serde(rename = "unit_split_corporateaction_event")]
    UnitSplit,
    /// A [`SpinOffEvent`].
    #[serde(rename = "spin_off_corporateaction_event")]
    SpinOff,
    /// A [`CashMergerEvent`].
    #[serde(rename = "cash_merger_corporateaction_event")]
    CashMerger,
    /// A [`StockMergerEvent`].
    #[serde(rename = "stock_merger_corporateaction_event")]
    StockMerger,
    /// A [`StockAndCashMergerEvent`].
    #[serde(rename = "stock_and_cash_merger_corporateaction_event")]
    StockAndCashMerger,
    /// A [`RedemptionEvent`].
    #[serde(rename = "redemption_corporateaction_event")]
    Redemption,
    /// An [`EquityPartialCallEvent`].
    #[serde(rename = "equity_partial_call_corporateaction_event")]
    EquityPartialCall,
    /// A [`ReorganizationEvent`].
    #[serde(rename = "reorganization_corporateaction_event")]
    Reorganization,
    /// A [`NameChangeEvent`].
    #[serde(rename = "name_change_corporateaction_event")]
    NameChange,
    /// A [`WorthlessRemovalEvent`].
    #[serde(rename = "worthless_removal_corporateaction_event")]
    WorthlessRemoval,
    /// A [`RightsDistributionEvent`].
    #[serde(rename = "rights_distribution_corporateaction_event")]
    RightsDistribution,
    /// An event type added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

impl CorporateActionEventType {
    /// The type as it appears on the wire and in the `type` query filter.
    pub fn as_str(&self) -> &str {
        match self {
            Self::CashDividend => "cash_dividend_corporateaction_event",
            Self::StockDividend => "stock_dividend_corporateaction_event",
            Self::ForwardSplit => "forward_split_corporateaction_event",
            Self::ReverseSplit => "reverse_split_corporateaction_event",
            Self::UnitSplit => "unit_split_corporateaction_event",
            Self::SpinOff => "spin_off_corporateaction_event",
            Self::CashMerger => "cash_merger_corporateaction_event",
            Self::StockMerger => "stock_merger_corporateaction_event",
            Self::StockAndCashMerger => "stock_and_cash_merger_corporateaction_event",
            Self::Redemption => "redemption_corporateaction_event",
            Self::EquityPartialCall => "equity_partial_call_corporateaction_event",
            Self::Reorganization => "reorganization_corporateaction_event",
            Self::NameChange => "name_change_corporateaction_event",
            Self::WorthlessRemoval => "worthless_removal_corporateaction_event",
            Self::RightsDistribution => "rights_distribution_corporateaction_event",
            Self::Other(value) => value,
        }
    }
}

/// The `ca` payload of a [`CorporateActionEvent`], resolved from the
/// envelope's `event_type`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum CorporateActionPayload {
    /// A cash dividend.
    CashDividend(CashDividendEvent),
    /// A stock dividend.
    StockDividend(StockDividendEvent),
    /// A forward split.
    ForwardSplit(ForwardSplitEvent),
    /// A reverse split.
    ReverseSplit(ReverseSplitEvent),
    /// A unit split.
    UnitSplit(UnitSplitEvent),
    /// A spin off.
    SpinOff(SpinOffEvent),
    /// A cash merger.
    CashMerger(CashMergerEvent),
    /// A stock merger.
    StockMerger(StockMergerEvent),
    /// A stock and cash merger.
    StockAndCashMerger(StockAndCashMergerEvent),
    /// A redemption.
    Redemption(RedemptionEvent),
    /// An equity partial call.
    EquityPartialCall(EquityPartialCallEvent),
    /// A reorganization.
    Reorganization(ReorganizationEvent),
    /// A name change.
    NameChange(NameChangeEvent),
    /// A worthless removal.
    WorthlessRemoval(WorthlessRemovalEvent),
    /// A rights distribution.
    RightsDistribution(RightsDistributionEvent),
    /// A payload whose `event_type` this crate does not model, kept as raw
    /// JSON so an unexpected type cannot break the stream.
    Other(serde_json::Value),
}

impl CorporateActionPayload {
    /// Parses a raw `ca` object according to the envelope's `event_type`.
    fn from_value(
        event_type: &CorporateActionEventType,
        ca: serde_json::Value,
    ) -> serde_json::Result<Self> {
        use CorporateActionEventType as Type;
        Ok(match event_type {
            Type::CashDividend => Self::CashDividend(serde_json::from_value(ca)?),
            Type::StockDividend => Self::StockDividend(serde_json::from_value(ca)?),
            Type::ForwardSplit => Self::ForwardSplit(serde_json::from_value(ca)?),
            Type::ReverseSplit => Self::ReverseSplit(serde_json::from_value(ca)?),
            Type::UnitSplit => Self::UnitSplit(serde_json::from_value(ca)?),
            Type::SpinOff => Self::SpinOff(serde_json::from_value(ca)?),
            Type::CashMerger => Self::CashMerger(serde_json::from_value(ca)?),
            Type::StockMerger => Self::StockMerger(serde_json::from_value(ca)?),
            Type::StockAndCashMerger => Self::StockAndCashMerger(serde_json::from_value(ca)?),
            Type::Redemption => Self::Redemption(serde_json::from_value(ca)?),
            Type::EquityPartialCall => Self::EquityPartialCall(serde_json::from_value(ca)?),
            Type::Reorganization => Self::Reorganization(serde_json::from_value(ca)?),
            Type::NameChange => Self::NameChange(serde_json::from_value(ca)?),
            Type::WorthlessRemoval => Self::WorthlessRemoval(serde_json::from_value(ca)?),
            Type::RightsDistribution => Self::RightsDistribution(serde_json::from_value(ca)?),
            Type::Other(_) => Self::Other(ca),
        })
    }
}

/// The envelope as it appears on the wire, with `ca` still unresolved.
#[derive(Debug, Deserialize)]
struct CorporateActionEventWire {
    event_id: String,
    at: DateTime<Utc>,
    action: CorporateActionMutation,
    region: CorporateActionEventRegion,
    event_type: CorporateActionEventType,
    ca: serde_json::Value,
}

/// A single corporate action mutation pushed over the events stream.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CorporateActionEvent {
    /// Lexically sortable, monotonically increasing ULID identifying this
    /// emission. Unique per message — an `update` or `delete` for the same
    /// corporate action carries a fresh id. Use it as `since_id` (or the
    /// `Last-Event-Id` header) to resume the stream.
    pub event_id: String,
    /// When the streaming service emitted the event.
    pub at: DateTime<Utc>,
    /// Whether the corporate action was inserted, updated or deleted.
    pub action: CorporateActionMutation,
    /// Market the corporate action belongs to.
    pub region: CorporateActionEventRegion,
    /// Discriminator naming the shape of `ca`.
    pub event_type: CorporateActionEventType,
    /// The corporate action itself.
    pub ca: CorporateActionPayload,
}

impl<'de> Deserialize<'de> for CorporateActionEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let wire = CorporateActionEventWire::deserialize(deserializer)?;
        let ca = CorporateActionPayload::from_value(&wire.event_type, wire.ca)
            .map_err(D::Error::custom)?;
        Ok(Self {
            event_id: wire.event_id,
            at: wire.at,
            action: wire.action,
            region: wire.region,
            event_type: wire.event_type,
            ca,
        })
    }
}

/// Query parameters for [`CorporateActionEventsClient::subscribe`].
///
/// Without a `since` or `since_id` the stream carries live events only. With
/// one of them, matching history is replayed first and the stream then
/// continues live; adding `until`/`until_id` closes the stream at that point
/// instead.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CorporateActionEventsRequest {
    /// Comma-separated `event_type` values to restrict the stream to; unset
    /// delivers every type. Build it with
    /// [`with_types`](Self::with_types).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub types: Option<String>,
    /// Markets to receive events for; unset means
    /// [`CorporateActionRegionFilter::All`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<CorporateActionRegionFilter>,
    /// Replay events emitted at or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<DateTime<Utc>>,
    /// Close the stream after this time. Requires `since`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<DateTime<Utc>>,
    /// Replay events with an `event_id` at or after this ULID. Mutually
    /// exclusive with `since`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<String>,
    /// Close the stream once this `event_id` has been delivered. Requires
    /// `since_id`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until_id: Option<String>,
}

impl CorporateActionEventsRequest {
    /// Creates a request with no filters and no replay window: every live
    /// event, in every region.
    pub fn new() -> Self {
        Self::default()
    }

    /// Restricts the stream to the given event types, joining them into the
    /// comma-separated `type` parameter the API expects.
    pub fn with_types<I>(mut self, types: I) -> Self
    where
        I: IntoIterator<Item = CorporateActionEventType>,
    {
        let joined = types
            .into_iter()
            .map(|event_type| event_type.as_str().to_string())
            .collect::<Vec<_>>()
            .join(",");
        self.types = (!joined.is_empty()).then_some(joined);
        self
    }

    /// Rejects the parameter combinations the endpoint refuses.
    pub(crate) fn validate(&self) -> Result<()> {
        validate_replay_window(
            self.since.is_some(),
            self.until.is_some(),
            self.since_id.is_some(),
            self.until_id.is_some(),
        )
    }
}

/// Client for the corporate action event stream.
///
/// Lives on the market data host and requires the same data subscription as
/// [`CorporateActionsClient`](crate::data::CorporateActionsClient).
pub struct CorporateActionEventsClient {
    base: Url,
    creds: Credentials,
}

impl CorporateActionEventsClient {
    /// Creates a new corporate action events client.
    pub fn new(creds: Credentials) -> Result<Self> {
        Self::with_base_url(creds, DATA_BASE)
    }

    /// Creates a new client targeting a custom base URL instead of the
    /// default Alpaca endpoint (parity with alpaca-py's `url_override`).
    pub fn with_base_url(creds: Credentials, base_url: &str) -> Result<Self> {
        Ok(Self {
            base: Url::parse(base_url)?,
            creds,
        })
    }

    /// `GET /v1beta1/events/corporate-actions` — opens the corporate action
    /// event stream.
    ///
    /// The returned [`EventStream`] yields one [`CorporateActionEvent`] per
    /// call to [`next`](EventStream::next) until the server closes the
    /// connection.
    pub async fn subscribe(
        &self,
        req: &CorporateActionEventsRequest,
    ) -> Result<EventStream<CorporateActionEvent>> {
        req.validate()?;
        EventStream::connect(&self.base, CORPORATE_ACTIONS_PATH, req, &self.creds).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use rust_decimal::Decimal;

    const CASH_DIVIDEND_JSON: &str = r#"{
        "action": "insert",
        "at": "2026-03-20T12:24:58.807230Z",
        "ca": {
            "currency": "USD",
            "cusip": "037833100",
            "ex_date": "2026-05-09",
            "foreign": false,
            "id": "1dbc7685-9517-4a77-a236-8527d49cefdc",
            "payable_date": "2026-05-15",
            "process_date": "2026-05-15",
            "rate": "0.24",
            "record_date": "2026-05-12",
            "special": false,
            "symbol": "AAPL"
        },
        "event_id": "01J9RPMV5TKB8WX3M4F1KZ7QH2",
        "event_type": "cash_dividend_corporateaction_event",
        "region": "us"
    }"#;

    const FORWARD_SPLIT_JSON: &str = r#"{
        "action": "update",
        "at": "2026-03-19T22:50:11.729329Z",
        "ca": {
            "currency": "USD",
            "cusip": "67066G104",
            "ex_date": "2026-06-10",
            "id": "78467a10-9aa2-4222-8927-abcdef012345",
            "new_rate": "10",
            "old_rate": "1",
            "payable_date": "2026-06-09",
            "process_date": "2026-06-07",
            "record_date": "2026-06-06",
            "symbol": "NVDA"
        },
        "event_id": "01KM5CDQXQAE67Z5NHCJZHQ5XV",
        "event_type": "forward_split_corporateaction_event",
        "region": "us"
    }"#;

    #[test]
    fn cash_dividend_event_round_trip() {
        let event: CorporateActionEvent = serde_json::from_str(CASH_DIVIDEND_JSON).unwrap();
        assert_eq!(event.action, CorporateActionMutation::Insert);
        assert_eq!(event.region, CorporateActionEventRegion::Us);
        assert_eq!(event.event_type, CorporateActionEventType::CashDividend);
        let CorporateActionPayload::CashDividend(ca) = &event.ca else {
            panic!("expected a cash dividend, got {:?}", event.ca);
        };
        assert_eq!(ca.symbol, "AAPL");
        assert_eq!(ca.rate, Decimal::new(24, 2));
        assert!(!ca.special);
        assert_eq!(ca.sub_type, None);

        // Serializing keeps the envelope, and `ca` re-parses to the same
        // typed payload.
        let again: CorporateActionEvent =
            serde_json::from_str(&serde_json::to_string(&event).unwrap()).unwrap();
        assert_eq!(again, event);
    }

    #[test]
    fn forward_split_event_uses_decimal_rates() {
        let event: CorporateActionEvent = serde_json::from_str(FORWARD_SPLIT_JSON).unwrap();
        assert_eq!(event.action, CorporateActionMutation::Update);
        let CorporateActionPayload::ForwardSplit(ca) = &event.ca else {
            panic!("expected a forward split, got {:?}", event.ca);
        };
        assert_eq!(ca.old_rate, Decimal::ONE);
        assert_eq!(ca.new_rate, Decimal::from(10));
        assert_eq!(ca.due_bill_redemption_date, None);
    }

    #[test]
    fn unknown_event_type_is_kept_as_raw_json() {
        let json = CASH_DIVIDEND_JSON.replace(
            "cash_dividend_corporateaction_event",
            "quantum_dividend_corporateaction_event",
        );
        let event: CorporateActionEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(
            event.event_type,
            CorporateActionEventType::Other("quantum_dividend_corporateaction_event".to_string())
        );
        let CorporateActionPayload::Other(raw) = &event.ca else {
            panic!("expected a raw payload, got {:?}", event.ca);
        };
        assert_eq!(raw["symbol"], "AAPL");
    }

    #[test]
    fn unknown_action_and_region_survive() {
        let json = CASH_DIVIDEND_JSON
            .replace("\"insert\"", "\"upsert\"")
            .replace("\"region\": \"us\"", "\"region\": \"emea\"");
        let event: CorporateActionEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(
            event.action,
            CorporateActionMutation::Other("upsert".to_string())
        );
        assert_eq!(
            event.region,
            CorporateActionEventRegion::Other("emea".to_string())
        );
    }

    #[test]
    fn malformed_payload_for_a_known_type_is_an_error() {
        let json = CASH_DIVIDEND_JSON.replace("\"rate\": \"0.24\",", "");
        assert!(serde_json::from_str::<CorporateActionEvent>(&json).is_err());
    }

    #[test]
    fn event_type_strings_round_trip() {
        for event_type in [
            CorporateActionEventType::CashDividend,
            CorporateActionEventType::StockAndCashMerger,
            CorporateActionEventType::RightsDistribution,
            CorporateActionEventType::Other("custom_event".to_string()),
        ] {
            let json = serde_json::to_string(&event_type).unwrap();
            assert_eq!(json, format!("\"{}\"", event_type.as_str()));
            let parsed: CorporateActionEventType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, event_type);
        }
    }

    #[test]
    fn request_serializes_filters_as_the_api_expects() {
        let req = CorporateActionEventsRequest::new();
        assert_eq!(serde_json::to_value(&req).unwrap(), serde_json::json!({}));

        let mut req = CorporateActionEventsRequest::new().with_types([
            CorporateActionEventType::CashDividend,
            CorporateActionEventType::StockDividend,
        ]);
        req.region = Some(CorporateActionRegionFilter::NonUs);
        assert_eq!(
            serde_json::to_value(&req).unwrap(),
            serde_json::json!({
                "type": "cash_dividend_corporateaction_event,stock_dividend_corporateaction_event",
                "region": "non_us"
            })
        );

        // An empty type list means "no filter", not an empty parameter.
        let req = CorporateActionEventsRequest::new().with_types([]);
        assert_eq!(req.types, None);
    }

    #[test]
    fn request_validation_rejects_bad_replay_windows() {
        let mut req = CorporateActionEventsRequest::new();
        req.until_id = Some("01J9RVB6Y4ZK8M3N7QD2WX1RFP".to_string());
        assert!(matches!(req.validate(), Err(Error::InvalidRequest(_))));

        req.since_id = Some("01J9RPMV5TKB8WX3M4F1KZ7QH2".to_string());
        assert!(req.validate().is_ok());
    }
}
