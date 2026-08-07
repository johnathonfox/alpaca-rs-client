//! Integration tests for the Server-Sent Event stream clients against a local
//! mock server.
//!
//! Hermetic: the server is wiremock on localhost, credentials are dummy
//! values, and no env vars or live Alpaca endpoints are touched.

use alpaca_rs::rest::Credentials;
use alpaca_rs::stream::{
    ActivityEventDetails, ActivityEventsClient, ActivityEventsRequest, CorporateActionEventRegion,
    CorporateActionEventType, CorporateActionEventsClient, CorporateActionEventsRequest,
    CorporateActionMutation, CorporateActionPayload, CorporateActionRegionFilter, ReconnectOptions,
    TradeExecutionType,
};
use alpaca_rs::trading::ActivityType;
use alpaca_rs::{Error, Result};
use wiremock::matchers::{header, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockBuilder, MockServer, ResponseTemplate};

fn test_credentials() -> Credentials {
    Credentials::Key {
        key_id: "test-key".to_string(),
        secret_key: "test-secret".to_string(),
    }
}

/// The standard auth and content-negotiation matchers every event-stream
/// request must carry.
fn auth(mock: MockBuilder) -> MockBuilder {
    mock.and(header("APCA-API-KEY-ID", "test-key"))
        .and(header("APCA-API-SECRET-KEY", "test-secret"))
        .and(header("accept", "text/event-stream"))
}

/// An SSE body: a comment, two data frames with ids, and a heartbeat.
fn activities_body() -> String {
    concat!(
        ": heartbeat\n",
        "id: 01J9RPMV5TKB8WX3M4F1KZ7QH2\n",
        r#"data: {"event_id":"01J9RPMV5TKB8WX3M4F1KZ7QH2","at":"2026-03-20T12:24:58.807230Z","#,
        r#""activity_type":"FILL","executed_at":"2026-03-20T12:24:58.700000Z","status":"executed","#,
        r#""settle_date":"2026-03-22","currency":"USD","#,
        r#""ref_id":"33cbb614-bfc0-468b-b4d0-ccf08588ef77","price":"181.36","qty":"10","#,
        r#""details":{"order_id":"bb2403bc-88ec-430b-b41c-f9ee80c8f0e1","side":"buy","#,
        r#""symbol":"AAPL","asset_id":"b0b6dd9d-8b9b-48a9-ba46-b9d54906e415","#,
        r#""leaves_qty":"0","cum_qty":"10","order_status":"filled","execution_type":"fill"}}"#,
        "\n\n",
        ": heartbeat\n",
        "id: 01J9RQ5HNZQK7M3RDVJ8XBPCT1\n",
        // The spec types the payload as an array, so exercise that shape too.
        r#"data: [{"event_id":"01J9RQ5HNZQK7M3RDVJ8XBPCT1","at":"2026-03-20T15:42:11.118274Z","#,
        r#""activity_type":"DIV","activity_subtype":"CDIV","#,
        r#""executed_at":"2026-03-20T15:42:10.000000Z","status":"executed","#,
        r#""settle_date":"2026-03-20","currency":"USD","#,
        r#""ref_id":"f8489167-4e4b-431d-a0be-6017ae1cf08a","net_amount":"0.07","#,
        r#""details":{"system_date":"2026-03-20","symbol":"JEPQ","cash_payout":"0.07"}}]"#,
        "\n\n",
    )
    .to_string()
}

/// A corporate action SSE body spanning two frames, the second one split over
/// several `data:` lines to exercise the parser's line joining.
fn corporate_actions_body() -> String {
    concat!(
        "id: 01J9RPMV5TKB8WX3M4F1KZ7QH2\n",
        r#"data: {"action":"insert","at":"2026-03-20T12:24:58.807230Z","#,
        r#""ca":{"id":"1dbc7685-9517-4a77-a236-8527d49cefdc","process_date":"2026-05-15","#,
        r#""symbol":"AAPL","cusip":"037833100","rate":"0.24","special":false,"foreign":false,"#,
        r#""ex_date":"2026-05-09","record_date":"2026-05-12","payable_date":"2026-05-15","#,
        r#""currency":"USD"},"event_id":"01J9RPMV5TKB8WX3M4F1KZ7QH2","#,
        r#""event_type":"cash_dividend_corporateaction_event","region":"us"}"#,
        "\n\n",
        "id: 01KM5CDQXQAE67Z5NHCJZHQ5XV\n",
        r#"data: {"action":"update","at":"2026-03-19T22:50:11.729329Z","#,
        "\n",
        r#"data: "ca":{"id":"78467a10-9aa2-4222-8927-abcdef012345","process_date":"2026-06-07","#,
        "\n",
        r#"data: "symbol":"NVDA","cusip":"67066G104","old_rate":"1","new_rate":"10","#,
        "\n",
        r#"data: "ex_date":"2026-06-10"},"event_id":"01KM5CDQXQAE67Z5NHCJZHQ5XV","#,
        "\n",
        r#"data: "event_type":"forward_split_corporateaction_event","region":"us"}"#,
        "\n\n",
    )
    .to_string()
}

/// The activity stream replays from `since_id`, decodes both activity
/// families out of the SSE body, tracks the last event id, and reports the
/// end of the body as `Ok(None)`.
#[tokio::test]
async fn activity_events_stream_decodes_trade_and_non_trade_frames() -> Result<()> {
    let server = MockServer::start().await;
    auth(Mock::given(method("GET")))
        .and(path("/v2beta1/events/activities"))
        .and(query_param("since_id", "01J9RPMV5TKB8WX3M4F1KZ7QH1"))
        .and(query_param_is_missing("until"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(activities_body(), "text/event-stream"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = ActivityEventsClient::with_base_url(test_credentials(), &server.uri())?;
    let mut req = ActivityEventsRequest::new();
    req.since_id = Some("01J9RPMV5TKB8WX3M4F1KZ7QH1".to_string());
    let mut stream = client.subscribe(&req).await?;

    let fill = stream.next().await?.expect("first event");
    assert_eq!(fill.activity_type, ActivityType::Fill);
    let ActivityEventDetails::Trade(details) = &fill.details else {
        panic!("expected trade details, got {:?}", fill.details);
    };
    assert_eq!(details.symbol, "AAPL");
    assert_eq!(details.execution_type, TradeExecutionType::Fill);

    let dividend = stream.next().await?.expect("second event");
    assert_eq!(dividend.activity_type, ActivityType::Div);
    let ActivityEventDetails::NonTrade(details) = &dividend.details else {
        panic!("expected non-trade details, got {:?}", dividend.details);
    };
    assert_eq!(details.extra["symbol"], "JEPQ");

    assert_eq!(stream.last_event_id(), Some("01J9RQ5HNZQK7M3RDVJ8XBPCT1"));
    // The mock body ends, which closes the stream.
    assert!(stream.next().await?.is_none());
    Ok(())
}

/// A bounded subscription (`until_id`) ends when the server closes the body,
/// even with auto-reconnect enabled.
///
/// Regression test: treating that clean end-of-stream as a disconnect made the
/// stream reconnect and replay the same finite window forever instead of
/// terminating. `expect(1)` is the assertion that matters — a second request
/// means the bug is back.
#[tokio::test]
async fn bounded_stream_ends_instead_of_reconnecting() -> Result<()> {
    let server = MockServer::start().await;
    auth(Mock::given(method("GET")))
        .and(path("/v2beta1/events/activities"))
        .and(query_param("until_id", "01J9RQ5HNZQK7M3RDVJ8XBPCT1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(activities_body(), "text/event-stream"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = ActivityEventsClient::with_base_url(test_credentials(), &server.uri())?;
    let mut req = ActivityEventsRequest::new();
    req.since_id = Some("01J9RPMV5TKB8WX3M4F1KZ7QH1".to_string());
    req.until_id = Some("01J9RQ5HNZQK7M3RDVJ8XBPCT1".to_string());
    let mut stream = client.subscribe(&req).await?;
    stream.set_auto_reconnect(ReconnectOptions::default());

    assert!(stream.next().await?.is_some(), "first event");
    assert!(stream.next().await?.is_some(), "second event");
    // The window was delivered in full, so the stream is done — not dropped.
    assert!(stream.next().await?.is_none(), "bounded stream must end");
    Ok(())
}

/// A rejected subscription surfaces the API status and body, and never yields
/// a stream.
#[tokio::test]
async fn activity_events_subscribe_maps_forbidden_to_error_api() -> Result<()> {
    let server = MockServer::start().await;
    auth(Mock::given(method("GET")))
        .and(path("/v2beta1/events/activities"))
        .respond_with(ResponseTemplate::new(403).set_body_string("events not enabled"))
        .expect(1)
        .mount(&server)
        .await;

    let client = ActivityEventsClient::with_base_url(test_credentials(), &server.uri())?;
    let err = client
        .subscribe(&ActivityEventsRequest::new())
        .await
        .expect_err("expected an API error");
    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 403);
            assert_eq!(message, "events not enabled");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
    Ok(())
}

/// An invalid replay window is rejected client-side, before any request goes
/// out.
#[tokio::test]
async fn activity_events_reject_until_without_since_before_sending() -> Result<()> {
    let server = MockServer::start().await;
    // No mock is mounted: any request would fail the test.
    let client = ActivityEventsClient::with_base_url(test_credentials(), &server.uri())?;
    let mut req = ActivityEventsRequest::new();
    req.until = Some(chrono::Utc::now());
    assert!(matches!(
        client.subscribe(&req).await,
        Err(Error::InvalidRequest(_))
    ));
    Ok(())
}

/// The corporate action stream sends the `type`/`region` filters and resolves
/// each `ca` payload from the envelope's `event_type`.
#[tokio::test]
async fn corporate_action_events_stream_resolves_typed_payloads() -> Result<()> {
    let server = MockServer::start().await;
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/events/corporate-actions"))
        .and(query_param(
            "type",
            "cash_dividend_corporateaction_event,forward_split_corporateaction_event",
        ))
        .and(query_param("region", "us"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(corporate_actions_body(), "text/event-stream"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = CorporateActionEventsClient::with_base_url(test_credentials(), &server.uri())?;
    let mut req = CorporateActionEventsRequest::new().with_types([
        CorporateActionEventType::CashDividend,
        CorporateActionEventType::ForwardSplit,
    ]);
    req.region = Some(CorporateActionRegionFilter::Us);
    let mut stream = client.subscribe(&req).await?;

    let dividend = stream.next().await?.expect("first event");
    assert_eq!(dividend.action, CorporateActionMutation::Insert);
    assert_eq!(dividend.region, CorporateActionEventRegion::Us);
    let CorporateActionPayload::CashDividend(ca) = &dividend.ca else {
        panic!("expected a cash dividend, got {:?}", dividend.ca);
    };
    assert_eq!(ca.symbol, "AAPL");
    assert_eq!(ca.rate, rust_decimal::Decimal::new(24, 2));

    // The second frame arrives split across several `data:` lines.
    let split = stream.next().await?.expect("second event");
    assert_eq!(split.action, CorporateActionMutation::Update);
    let CorporateActionPayload::ForwardSplit(ca) = &split.ca else {
        panic!("expected a forward split, got {:?}", split.ca);
    };
    assert_eq!(ca.symbol, "NVDA");
    assert_eq!(ca.new_rate, rust_decimal::Decimal::from(10));

    assert_eq!(stream.last_event_id(), Some("01KM5CDQXQAE67Z5NHCJZHQ5XV"));
    assert!(stream.next().await?.is_none());
    Ok(())
}
