//! Integration tests for `TradingClient` against a local mock server.
//!
//! Hermetic: the server is wiremock on localhost, credentials are dummy
//! values, and no env vars or live Alpaca endpoints are touched.

use alpaca_rs::data::enums::Sort;
use alpaca_rs::rest::Credentials;
use alpaca_rs::trading::{
    AccountActivitiesRequest, AccountActivity, ActivityType, OrderRequest, OrderSide, TimeInForce,
    TradingClient,
};
use alpaca_rs::{Error, Result};
use rust_decimal::Decimal;
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockBuilder, MockServer, ResponseTemplate};

fn test_credentials() -> Credentials {
    Credentials::Key {
        key_id: "test-key".to_string(),
        secret_key: "test-secret".to_string(),
    }
}

/// The standard auth-header matchers every trading request must carry.
fn auth(mock: MockBuilder) -> MockBuilder {
    mock.and(header("APCA-API-KEY-ID", "test-key"))
        .and(header("APCA-API-SECRET-KEY", "test-secret"))
}

const ORDER_JSON: &str = r#"{
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

#[tokio::test]
async fn submit_order_posts_json_with_auth_headers() -> Result<()> {
    let server = MockServer::start().await;

    let expected_body = json!({
        "symbol": "AAPL",
        "qty": "1",
        "side": "buy",
        "type": "market",
        "time_in_force": "day"
    });
    auth(Mock::given(method("POST")))
        .and(path("/v2/orders"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(ORDER_JSON, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let order = client
        .submit_order(&OrderRequest::market(
            "AAPL",
            OrderSide::Buy,
            Decimal::new(1, 0),
            TimeInForce::Day,
        ))
        .await?;

    assert_eq!(order.id, "61e69015-8549-4bfd-b9c3-01e75843f47d");
    assert_eq!(order.symbol, "AAPL");
    assert_eq!(order.qty, Some(Decimal::new(1, 0)));
    assert_eq!(order.filled_qty, Some(Decimal::ZERO));

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_account_parses_string_decimal_fields() -> Result<()> {
    let server = MockServer::start().await;

    let account = json!({
        "id": "5c56a945-89a0-4c5e-9d1f-2b3c4d5e6f70",
        "account_number": "PA3AB12CDEFG",
        "status": "ACTIVE",
        "crypto_status": "ACTIVE",
        "currency": "USD",
        "buying_power": "262556.88",
        "regt_buying_power": "262556.88",
        "daytrading_buying_power": "262556.88",
        "non_marginable_buying_power": "131276.78",
        "cash": "131276.78",
        "accrued_fees": "0",
        "portfolio_value": "131276.78",
        "pattern_day_trader": false,
        "trading_blocked": false,
        "transfers_blocked": false,
        "account_blocked": false,
        "created_at": "2024-01-02T15:04:05Z",
        "trade_suspended_by_user": false,
        "multiplier": "2",
        "shorting_enabled": true,
        "equity": "131276.78",
        "last_equity": "131276.78",
        "long_market_value": "0",
        "short_market_value": "0",
        "initial_margin": "0",
        "maintenance_margin": "0",
        "last_maintenance_margin": "0",
        "sma": "0",
        "daytrade_count": 0
    });
    auth(Mock::given(method("GET")))
        .and(path("/v2/account"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&account))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let account = client.get_account().await?;

    assert_eq!(account.account_number, "PA3AB12CDEFG");
    // Trading API numbers arrive as JSON strings and land in `Decimal`.
    assert_eq!(account.buying_power, Decimal::new(26_255_688, 2));
    assert_eq!(account.cash, Decimal::new(13_127_678, 2));
    assert_eq!(account.multiplier, "2");
    assert_eq!(account.pattern_day_trader, Some(false));

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn forbidden_response_maps_to_error_api() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({"code": 40310000, "message": "forbidden."});
    Mock::given(method("GET"))
        .and(path("/v2/account"))
        .respond_with(ResponseTemplate::new(403).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let err = client
        .get_account()
        .await
        .expect_err("403 must surface as an error");

    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 403);
            assert!(message.contains("forbidden"), "message: {message}");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }

    server.verify().await;
    Ok(())
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

#[tokio::test]
async fn get_account_activities_sends_filters_and_auth() -> Result<()> {
    let server = MockServer::start().await;

    let body = format!("[{FILL_ACTIVITY_JSON},{DIV_ACTIVITY_JSON}]");
    auth(Mock::given(method("GET")))
        .and(path("/v2/account/activities"))
        .and(query_param("activity_types", "FILL,DIV"))
        .and(query_param("date", "2022-03-07"))
        .and(query_param("direction", "desc"))
        .and(query_param("page_size", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let req = AccountActivitiesRequest {
        activity_types: Some(vec![ActivityType::Fill, ActivityType::Div]),
        date: Some(chrono::NaiveDate::from_ymd_opt(2022, 3, 7).unwrap()),
        direction: Some(Sort::Desc),
        page_size: Some(2),
        ..Default::default()
    };
    let activities = client.get_account_activities(&req).await?;

    assert_eq!(activities.len(), 2);
    match &activities[0] {
        AccountActivity::Trade(t) => {
            assert_eq!(t.symbol, "AAPL");
            assert_eq!(t.price, Decimal::new(12_781, 2));
            assert_eq!(t.activity_type, ActivityType::Fill);
        }
        other => panic!("expected trade activity, got {other:?}"),
    }
    match &activities[1] {
        AccountActivity::NonTrade(n) => {
            assert_eq!(n.net_amount, Decimal::new(112, 2));
            assert_eq!(n.activity_type, ActivityType::Div);
        }
        other => panic!("expected non-trade activity, got {other:?}"),
    }

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_account_activities_by_type_uses_type_path() -> Result<()> {
    let server = MockServer::start().await;

    let body = format!("[{FILL_ACTIVITY_JSON}]");
    auth(Mock::given(method("GET")))
        .and(path("/v2/account/activities/FILL"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let activities = client
        .get_account_activities_by_type(ActivityType::Fill, &AccountActivitiesRequest::default())
        .await?;

    assert_eq!(activities.len(), 1);
    assert!(matches!(activities[0], AccountActivity::Trade(_)));

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn account_activities_error_maps_to_error_api() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({"code": 42210000, "message": "invalid query parameters"});
    Mock::given(method("GET"))
        .and(path("/v2/account/activities"))
        .respond_with(ResponseTemplate::new(422).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let err = client
        .get_account_activities(&AccountActivitiesRequest::default())
        .await
        .expect_err("422 must surface as an error");

    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 422);
            assert!(message.contains("invalid query"), "message: {message}");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }

    server.verify().await;
    Ok(())
}
