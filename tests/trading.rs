//! Integration tests for `TradingClient` against a local mock server.
//!
//! Hermetic: the server is wiremock on localhost, credentials are dummy
//! values, and no env vars or live Alpaca endpoints are touched.

use alpaca_rs::data::enums::Sort;
use alpaca_rs::rest::Credentials;
use alpaca_rs::trading::{
    AccountActivitiesRequest, AccountActivity, ActivityType, AddAssetToWatchlistRequest,
    AssetAttribute, BorrowStatus, CalendarTimezone, CalendarV3Request, CreateLocateRequest,
    CreateWhitelistedAddressRequest, CryptoChain, CryptoTransferDirection, GetAssetsRequest,
    GetLocatesRequest, GetTokenizationRequestsRequest, GetWalletsRequest, LocateQuoteErrorCode,
    LocateStatus, Market, MarketPhase, OrderRequest, OrderSide, TimeInForce, TokenizationIssuer,
    TokenizationMintRequest, TokenizationNetwork, TokenizationRequestStatus,
    TokenizationRequestType, TradingClient, TransferFeeEstimateRequest, UpdateWatchlistRequest,
    WhitelistStatus,
};
use alpaca_rs::{Error, Result};
use rust_decimal::Decimal;
use serde_json::json;
use wiremock::matchers::{body_json, body_string, header, method, path, query_param};
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

#[tokio::test]
async fn do_not_exercise_option_posts_to_contract_path() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("POST")))
        .and(path("/v2/positions/AAPL251219C00250000/do-not-exercise"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    // An empty 204 body must not be treated as a decoding failure.
    client.do_not_exercise_option("AAPL251219C00250000").await?;

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn do_not_exercise_option_maps_forbidden_to_error_api() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({"code": 40310001, "message": "cannot submit DNE for short position"});
    Mock::given(method("POST"))
        .and(path("/v2/positions/AAPL251219C00250000/do-not-exercise"))
        .respond_with(ResponseTemplate::new(403).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let err = client
        .do_not_exercise_option("AAPL251219C00250000")
        .await
        .expect_err("403 must surface as an error");

    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 403);
            assert!(message.contains("short position"), "message: {message}");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_assets_sends_attributes_filter_and_parses_new_fields() -> Result<()> {
    let server = MockServer::start().await;

    let assets = json!([{
        "id": "b0b6dd9d-8b9b-48a9-ba46-b9d54906e415",
        "class": "us_equity",
        "exchange": "NASDAQ",
        "symbol": "AAPL",
        "name": "Apple Inc. Common Stock",
        "status": "active",
        "tradable": true,
        "marginable": true,
        "shortable": true,
        "easy_to_borrow": true,
        "borrow_status": "hard_to_borrow",
        "fractionable": true,
        "maintenance_margin_requirement": 30,
        "margin_requirement_long": "30",
        "margin_requirement_short": "35.5",
        "attributes": ["has_options", "warp_drive_enabled"]
    }]);
    auth(Mock::given(method("GET")))
        .and(path("/v2/assets"))
        .and(query_param("status", "active"))
        .and(query_param("attributes", "has_options,overnight_tradable"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&assets))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let assets = client
        .get_assets(&GetAssetsRequest {
            status: Some(alpaca_rs::trading::AssetStatus::Active),
            attributes: Some(vec![
                AssetAttribute::HasOptions,
                AssetAttribute::OvernightTradable,
            ]),
            ..Default::default()
        })
        .await?;

    assert_eq!(assets.len(), 1);
    let asset = &assets[0];
    assert_eq!(asset.symbol, "AAPL");
    assert_eq!(asset.borrow_status, Some(BorrowStatus::HardToBorrow));
    assert_eq!(asset.margin_requirement_long, Some(Decimal::new(30, 0)));
    assert_eq!(asset.margin_requirement_short, Some(Decimal::new(355, 1)));
    assert_eq!(
        asset.attributes,
        vec![
            AssetAttribute::HasOptions,
            AssetAttribute::Other("warp_drive_enabled".into()),
        ]
    );

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_clock_v3_sends_markets_and_parses_phases() -> Result<()> {
    let server = MockServer::start().await;

    let clocks = json!({
        "clocks": [
            {
                "market": {
                    "acronym": "NYSE",
                    "name": "New York Stock Exchange",
                    "timezone": "America/New_York",
                    "mic": "XNYS"
                },
                "timestamp": "2025-06-24T02:15:22-04:00",
                "is_market_day": true,
                "next_market_open": "2025-06-24T09:30:00-04:00",
                "next_market_close": "2025-06-24T16:00:00-04:00",
                "phase": "closed",
                "phase_until": "2025-06-24T04:00:00-04:00"
            },
            {
                "market": {
                    "acronym": "BOATS",
                    "name": "Blue Ocean Alternative Trading System",
                    "timezone": "America/New_York"
                },
                "timestamp": "2025-06-24T02:15:22-04:00",
                "is_market_day": true,
                "next_market_open": "2025-06-24T20:00:00-04:00",
                "next_market_close": "2025-06-24T04:00:00-04:00",
                "phase": "core",
                "phase_until": "2025-06-24T04:00:00-04:00"
            }
        ]
    });
    auth(Mock::given(method("GET")))
        .and(path("/v3/clock"))
        .and(query_param("markets", "NYSE,BOATS"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&clocks))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let clock = client.get_clock_v3(&[Market::NYSE, Market::BOATS]).await?;

    assert_eq!(clock.clocks.len(), 2);
    assert_eq!(clock.clocks[0].market.acronym, Market::NYSE);
    assert_eq!(clock.clocks[0].phase, MarketPhase::Closed);
    // Overnight: the regular market is closed while BOATS is in its core
    // session.
    assert_eq!(clock.clocks[1].market.acronym, Market::BOATS);
    assert_eq!(clock.clocks[1].phase, MarketPhase::Core);
    assert!(clock.clocks[1].is_market_day);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_calendar_v3_uses_market_path_and_range_query() -> Result<()> {
    let server = MockServer::start().await;

    let calendar = json!({
        "market": {
            "acronym": "OPRA",
            "name": "Options Price Reporting Authority",
            "timezone": "America/New_York"
        },
        "calendar": [{
            "date": "2025-01-02",
            "core_start": "2025-01-02T14:30:00Z",
            "core_end": "2025-01-02T21:00:00Z",
            "settlement_date": "2025-01-03"
        }]
    });
    auth(Mock::given(method("GET")))
        .and(path("/v3/calendar/OPRA"))
        .and(query_param("start", "2025-01-02"))
        .and(query_param("end", "2025-01-09"))
        .and(query_param("timezone", "UTC"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&calendar))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let calendar = client
        .get_calendar_v3(
            &Market::OPRA,
            &CalendarV3Request {
                start: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 2).unwrap()),
                end: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 9).unwrap()),
                timezone: Some(CalendarTimezone::Utc),
            },
        )
        .await?;

    assert_eq!(calendar.market.acronym, Market::OPRA);
    assert_eq!(calendar.calendar.len(), 1);
    let day = &calendar.calendar[0];
    assert_eq!(
        day.date,
        chrono::NaiveDate::from_ymd_opt(2025, 1, 2).unwrap()
    );
    assert_eq!(day.core_start.to_rfc3339(), "2025-01-02T14:30:00+00:00");
    assert_eq!(day.pre_start, None);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_calendar_v3_error_maps_to_error_api() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({"code": 40010001, "message": "unknown market"});
    Mock::given(method("GET"))
        .and(path("/v3/calendar/XPHL"))
        .respond_with(ResponseTemplate::new(400).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let err = client
        .get_calendar_v3(&Market::Other("XPHL".into()), &CalendarV3Request::default())
        .await
        .expect_err("400 must surface as an error");

    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 400);
            assert!(message.contains("unknown market"), "message: {message}");
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

const LOCATE_JSON: &str = r#"{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "symbol": "TSLA",
    "requested_qty": 200,
    "all_or_none": true,
    "status": "active",
    "created_at": "2026-01-02T15:04:05Z",
    "located_qty": 200,
    "located_price": "0.05",
    "total_fee": "10.00",
    "limit_price": "0.05",
    "expires_at": "2026-01-03T01:00:00Z"
}"#;

#[tokio::test]
async fn get_locate_quotes_sends_joined_symbols() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({
        "quotes": [{
            "symbol": "TSLA",
            "available_qty": 1000,
            "price": "0.0123",
            "quoted_at": "2026-01-02T15:04:05Z"
        }],
        "errors": [{
            "symbol": "AAPL",
            "code": "easy_to_borrow",
            "message": "symbol is easy to borrow"
        }]
    });
    auth(Mock::given(method("GET")))
        .and(path("/v1/locates/quotes"))
        .and(query_param("symbols", "TSLA,AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let quotes = client
        .get_locate_quotes(&["TSLA".to_string(), "AAPL".to_string()])
        .await?;

    assert_eq!(quotes.quotes.len(), 1);
    assert_eq!(quotes.quotes[0].symbol, "TSLA");
    assert_eq!(quotes.quotes[0].available_qty, 1000);
    assert_eq!(quotes.quotes[0].price, Some(Decimal::new(123, 4)));
    assert_eq!(quotes.errors[0].code, LocateQuoteErrorCode::EasyToBorrow);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn create_locate_posts_json_body() -> Result<()> {
    let server = MockServer::start().await;

    let expected_body = json!({
        "symbol": "TSLA",
        "qty": 200,
        "limit_price": "0.05",
        "all_or_none": true
    });
    auth(Mock::given(method("POST")))
        .and(path("/v1/locates"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(201).set_body_raw(LOCATE_JSON, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let req = CreateLocateRequest {
        limit_price: Some(Decimal::new(5, 2)),
        all_or_none: Some(true),
        ..CreateLocateRequest::new("TSLA", 200)
    };
    let locate = client.create_locate(&req).await?;

    assert_eq!(locate.symbol, "TSLA");
    assert_eq!(locate.status, LocateStatus::Active);
    assert_eq!(locate.located_qty, Some(200));
    assert_eq!(locate.total_fee, Some(Decimal::new(1000, 2)));

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn create_locate_rejects_non_round_lot_qty_before_sending() -> Result<()> {
    let server = MockServer::start().await;

    // The API answers a non-round-lot quantity with HTTP 400; the client
    // catches it first, so no request must reach the server at all.
    Mock::given(method("POST"))
        .and(path("/v1/locates"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "code": "invalid_input",
            "message": "invalid input: quantity must be in round lots of 100"
        })))
        .expect(0)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let err = client
        .create_locate(&CreateLocateRequest::new("TSLA", 150))
        .await
        .expect_err("a non-round-lot quantity must be rejected client-side");

    match err {
        Error::InvalidRequest(message) => {
            assert!(message.contains("round lot"), "message: {message}");
        }
        other => panic!("expected Error::InvalidRequest, got {other:?}"),
    }

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_locates_sends_filters_and_returns_page_token() -> Result<()> {
    let server = MockServer::start().await;

    let body = format!(r#"{{"locates": [{LOCATE_JSON}], "next_page_token": "tok456"}}"#);
    auth(Mock::given(method("GET")))
        .and(path("/v1/locates"))
        .and(query_param("status", "active"))
        .and(query_param("symbol", "TSLA"))
        .and(query_param("start", "2026-01-02"))
        .and(query_param("end", "2026-01-09"))
        .and(query_param("limit", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let req = GetLocatesRequest {
        status: Some(LocateStatus::Active),
        symbol: Some("TSLA".into()),
        start: Some(chrono::NaiveDate::from_ymd_opt(2026, 1, 2).unwrap()),
        end: Some(chrono::NaiveDate::from_ymd_opt(2026, 1, 9).unwrap()),
        limit: Some(50),
        ..Default::default()
    };
    let page = client.get_locates(&req).await?;

    assert_eq!(page.locates.len(), 1);
    assert_eq!(page.locates[0].requested_qty, 200);
    assert_eq!(page.next_page_token.as_deref(), Some("tok456"));

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_locate_uses_id_path() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("GET")))
        .and(path("/v1/locates/550e8400-e29b-41d4-a716-446655440000"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(LOCATE_JSON, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let locate = client
        .get_locate("550e8400-e29b-41d4-a716-446655440000")
        .await?;

    assert_eq!(locate.id, "550e8400-e29b-41d4-a716-446655440000");
    assert!(locate.all_or_none);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn create_locate_error_maps_to_error_api() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({
        "code": "easy_to_borrow",
        "message": "security is easy-to-borrow and does not require a locate"
    });
    Mock::given(method("POST"))
        .and(path("/v1/locates"))
        .respond_with(ResponseTemplate::new(422).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let err = client
        .create_locate(&CreateLocateRequest::new("AAPL", 100))
        .await
        .expect_err("422 must surface as an error");

    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 422);
            assert!(message.contains("easy-to-borrow"), "message: {message}");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }

    server.verify().await;
    Ok(())
}

const WATCHLIST_JSON: &str = r#"{
    "id": "fba8fc10-1f5c-4a56-9a11-4f7e8e0e6f9e",
    "account_id": "5fc0795e-3d8c-4e91-9a86-6a0e1a1c4a41",
    "name": "Primary Watchlist",
    "created_at": "2024-07-24T07:56:53.123456789Z",
    "updated_at": "2024-07-24T07:56:53.123456789Z",
    "assets": []
}"#;

/// The `:by_name` watchlist endpoints all hang off one path and carry the
/// watchlist name in the `name` query parameter; the bodies match the by-id
/// variants.
#[tokio::test]
async fn watchlist_by_name_get_and_delete_send_name_query() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("GET")))
        .and(path("/v2/watchlists:by_name"))
        .and(query_param("name", "Primary Watchlist"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(WATCHLIST_JSON, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    auth(Mock::given(method("DELETE")))
        .and(path("/v2/watchlists:by_name"))
        .and(query_param("name", "Primary Watchlist"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;

    let watchlist = client.get_watchlist_by_name("Primary Watchlist").await?;
    assert_eq!(watchlist.name, "Primary Watchlist");
    assert_eq!(watchlist.assets.map(|assets| assets.len()), Some(0));

    client.delete_watchlist_by_name("Primary Watchlist").await?;

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn watchlist_by_name_update_puts_body_with_name_query() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("PUT")))
        .and(path("/v2/watchlists:by_name"))
        .and(query_param("name", "Primary Watchlist"))
        .and(body_json(
            json!({"name": "Renamed", "symbols": ["AAPL", "MSFT"]}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_raw(WATCHLIST_JSON, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let req = UpdateWatchlistRequest {
        name: Some("Renamed".to_string()),
        symbols: Some(vec!["AAPL".to_string(), "MSFT".to_string()]),
    };
    let watchlist = client
        .update_watchlist_by_name("Primary Watchlist", &req)
        .await?;

    assert_eq!(watchlist.id, "fba8fc10-1f5c-4a56-9a11-4f7e8e0e6f9e");

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn watchlist_by_name_add_asset_posts_symbol_body() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("POST")))
        .and(path("/v2/watchlists:by_name"))
        .and(query_param("name", "Primary Watchlist"))
        .and(body_json(json!({"symbol": "TSLA"})))
        .respond_with(ResponseTemplate::new(200).set_body_raw(WATCHLIST_JSON, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let req = AddAssetToWatchlistRequest {
        symbol: "TSLA".to_string(),
    };
    let watchlist = client
        .add_asset_to_watchlist_by_name("Primary Watchlist", &req)
        .await?;

    assert_eq!(watchlist.name, "Primary Watchlist");

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn watchlist_by_name_unknown_name_maps_to_error_api() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({"code": 40410000, "message": "watchlist not found"});
    Mock::given(method("GET"))
        .and(path("/v2/watchlists:by_name"))
        .respond_with(ResponseTemplate::new(404).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let err = client
        .get_watchlist_by_name("No Such List")
        .await
        .expect_err("404 must surface as an error");

    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 404);
            assert!(
                message.contains("watchlist not found"),
                "message: {message}"
            );
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }

    server.verify().await;
    Ok(())
}

// --- Crypto wallets & funding (`/v2/wallets*`) -----------------------------

const WALLET_JSON: &str = r#"{"asset_id":"5d0de74f-827b-41a7-9f74-9c07c08fe55f","address":"0x42a76C83014e886e639768D84EAF3573b1876844","created_at":"2025-08-07T08:52:40.656166Z"}"#;

const TRANSFER_JSON: &str = r#"{"id":"876b1c4f-df5e-4d1b-beaa-81af7f7bd02c","tx_hash":"0xaca1f6ba105a68771d966b4ce17e0992ad3d8030d127cdb18e113efa3a864992","direction":"INCOMING","amount":"10","usd_value":"9.99707","chain":"ETH","asset":"USDC","from_address":"0x3C3380cdFb94dFEEaA41cAD9F58254AE380d752D","to_address":"0x42a76C83014e886e639768D84EAF3573b1876844","status":"COMPLETE","created_at":"2025-08-07T10:31:41.964121Z","network_fee":"0","fees":"0"}"#;

const WHITELIST_JSON: &str = r#"{"id":"45efdedd-28cd-4665-98b4-601d5f34ae0a","chain":"ETH","asset":"USDC","address":"0xf38Ecf5764fD2dEcB0dd9C1E7513a0b6eC0dD08a","created_at":"2025-08-07T13:16:46.49111Z","status":"PENDING"}"#;

#[tokio::test]
async fn get_wallets_parses_array_shape() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("GET")))
        .and(path("/v2/wallets"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(format!("[{WALLET_JSON}]"), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let wallets = client.get_wallets(&GetWalletsRequest::default()).await?;

    assert_eq!(wallets.len(), 1);
    assert_eq!(
        wallets[0].address,
        "0x42a76C83014e886e639768D84EAF3573b1876844"
    );
    assert_eq!(
        wallets[0].asset_id.as_deref(),
        Some("5d0de74f-827b-41a7-9f74-9c07c08fe55f")
    );

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_wallets_parses_single_object_shape_when_asset_filtered() -> Result<()> {
    let server = MockServer::start().await;

    // Quirk: with the `asset` filter the endpoint answers with one wallet
    // object instead of an array.
    auth(Mock::given(method("GET")))
        .and(path("/v2/wallets"))
        .and(query_param("asset", "USDC"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(WALLET_JSON, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let wallets = client
        .get_wallets(&GetWalletsRequest {
            asset: Some("USDC".into()),
            chain: Some(CryptoChain::Eth),
        })
        .await?;

    assert_eq!(wallets.len(), 1);
    assert_eq!(
        wallets[0].asset_id.as_deref(),
        Some("5d0de74f-827b-41a7-9f74-9c07c08fe55f")
    );

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_wallet_transfer_parses_fixture() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("GET")))
        .and(path(
            "/v2/wallets/transfers/876b1c4f-df5e-4d1b-beaa-81af7f7bd02c",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_raw(TRANSFER_JSON, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let transfer = client
        .get_wallet_transfer("876b1c4f-df5e-4d1b-beaa-81af7f7bd02c")
        .await?;

    assert_eq!(transfer.direction, Some(CryptoTransferDirection::Incoming));
    assert_eq!(transfer.amount, Some(Decimal::new(10, 0)));

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn create_whitelisted_address_parses_array_wrapped_response() -> Result<()> {
    let server = MockServer::start().await;

    let expected_body = json!({
        "address": "0xf38Ecf5764fD2dEcB0dd9C1E7513a0b6eC0dD08a",
        "asset": "USDC",
        "chain": "ETH"
    });
    auth(Mock::given(method("POST")))
        .and(path("/v2/wallets/whitelists"))
        .and(body_json(&expected_body))
        .respond_with(
            // The guide shows the response array-wrapped; the schema says a
            // single object. Exercise the lenient shape here.
            ResponseTemplate::new(200)
                .set_body_raw(format!("[{WHITELIST_JSON}]"), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let address = client
        .create_whitelisted_address(&CreateWhitelistedAddressRequest {
            address: "0xf38Ecf5764fD2dEcB0dd9C1E7513a0b6eC0dD08a".into(),
            asset: "USDC".into(),
            chain: Some(CryptoChain::Eth),
        })
        .await?;

    assert_eq!(address.id, "45efdedd-28cd-4665-98b4-601d5f34ae0a");
    assert_eq!(address.status, WhitelistStatus::Pending);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn delete_whitelisted_address_handles_empty_200() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("DELETE")))
        .and(path(
            "/v2/wallets/whitelists/45efdedd-28cd-4665-98b4-601d5f34ae0a",
        ))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    client
        .delete_whitelisted_address("45efdedd-28cd-4665-98b4-601d5f34ae0a")
        .await?;

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_transfer_fee_estimate_parses_fee_and_network_fee() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("GET")))
        .and(path("/v2/wallets/fees/estimate"))
        .and(query_param("asset", "USDC"))
        .and(query_param("amount", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "fee": "0.25",
            "network_fee": "0.001"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let estimate = client
        .get_transfer_fee_estimate(&TransferFeeEstimateRequest {
            asset: "USDC".into(),
            from_address: "0x3C3380cdFb94dFEEaA41cAD9F58254AE380d752D".into(),
            to_address: "0x42a76C83014e886e639768D84EAF3573b1876844".into(),
            amount: Decimal::new(10, 0),
        })
        .await?;

    assert_eq!(estimate.fee, Decimal::new(25, 2));
    assert_eq!(estimate.network_fee, Some(Decimal::new(1, 3)));

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn wallet_endpoints_map_4xx_to_error_api() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({"code": 40310000, "message": "crypto funding is not enabled"});
    Mock::given(method("GET"))
        .and(path("/v2/wallets"))
        .respond_with(ResponseTemplate::new(403).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let err = client
        .get_wallets(&GetWalletsRequest::default())
        .await
        .expect_err("403 must surface as an error");

    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 403);
            assert!(
                message.contains("crypto funding is not enabled"),
                "message: {message}"
            );
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }

    server.verify().await;
    Ok(())
}

// --- Crypto perpetuals (`/v2/perpetuals/*`) --------------------------------

#[tokio::test]
async fn set_perp_leverage_posts_query_params_without_body() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("POST")))
        .and(path("/v2/perpetuals/leverage"))
        .and(query_param("symbol", "BTC-PERP"))
        .and(query_param("leverage", "5"))
        .and(body_string(""))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "symbol": "BTC-PERP",
            "leverage": 5
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let leverage = client.set_perp_leverage("BTC-PERP", 5).await?;

    assert_eq!(leverage.symbol, "BTC-PERP");
    assert_eq!(leverage.leverage, 5);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_perp_leverage_parses_response() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("GET")))
        .and(path("/v2/perpetuals/leverage"))
        .and(query_param("symbol", "BTC-PERP"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "symbol": "BTC-PERP",
            "leverage": 10
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let leverage = client.get_perp_leverage("BTC-PERP").await?;

    assert_eq!(leverage.leverage, 10);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_perp_account_vitals_accepts_numbers_and_strings() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("GET")))
        .and(path("/v2/perpetuals/account_vitals"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "maintenance_margin": "100.5",
            "collateral_balance": 200.25,
            "total_collateral": "300",
            "profit_loss": -10.5,
            "margin_ratio": "0.75"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let vitals = client.get_perp_account_vitals().await?;

    assert_eq!(vitals.maintenance_margin, Some(Decimal::new(1005, 1)));
    assert_eq!(vitals.collateral_balance, Some(Decimal::new(20_025, 2)));
    assert_eq!(vitals.total_collateral, Some(Decimal::new(300, 0)));
    assert_eq!(vitals.profit_loss, Some(Decimal::new(-105, 1)));
    assert_eq!(vitals.margin_ratio, Some(Decimal::new(75, 2)));

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn perp_wallet_transfer_posts_body() -> Result<()> {
    let server = MockServer::start().await;

    let expected_body = json!({
        "amount": "2.5",
        "address": "0x42a76C83014e886e639768D84EAF3573b1876844",
        "asset": "USDC"
    });
    auth(Mock::given(method("POST")))
        .and(path("/v2/perpetuals/wallets/transfers"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(TRANSFER_JSON, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let transfer = client
        .create_perp_wallet_transfer(&alpaca_rs::trading::CreatePerpTransferRequest {
            amount: Decimal::new(25, 1),
            address: "0x42a76C83014e886e639768D84EAF3573b1876844".into(),
            asset: "USDC".into(),
        })
        .await?;

    assert_eq!(transfer.asset.as_deref(), Some("USDC"));

    server.verify().await;
    Ok(())
}

// --- Tokenization (`/v2/tokenization/*`) -----------------------------------

const MINT_RESPONSE_JSON: &str = r#"{"tokenization_request_id":"14d484e3-46f9-4e11-99ac-6fee0d4455c7","created_at":"2025-09-12T17:28:48.642437-04:00","status":"pending","underlying_symbol":"AAPL","token_symbol":"AAPLx","qty":"3","issuer":"xstocks","network":"solana","client_request_id":"my-mint-ref-001"}"#;

const TOKENIZATION_REQUEST_JSON: &str = r#"{"created_at":"2025-11-04T18:38:01.942282Z","fees":"0.5","issuer":"xstocks","network":"solana","qty":"1.0","status":"completed","token_symbol":"TSLAx","tokenization_request_id":"5b1d6a3e-7f0a-4d2c-b8e1-9e6f1c0d2c4a","tx_hash":"5J7ZxK4QwR3vH2pY1aB6sN8cM9tD0fA2eXyVwUjL3kPqR4mZcF7nB1tA8sH6dG2pE","type":"mint","underlying_symbol":"TSLA","updated_at":"2025-11-04T18:38:42.117004Z","wallet_address":"5dXY1aH2tQpV3wXmJg6Z7c8B4nKvF9bA1pQrSt2uVwYxXz"}"#;

#[tokio::test]
async fn mint_tokenized_asset_sends_idempotency_key_header() -> Result<()> {
    let server = MockServer::start().await;

    let expected_body = json!({
        "underlying_symbol": "AAPL",
        "qty": "3",
        "issuer": "xstocks",
        "network": "solana",
        "wallet_address": "5dXY1aH2tQpV3wXmJg6Z7c8B4nKvF9bA1pQrSt2uVwYxXz",
        "client_request_id": "my-mint-ref-001"
    });
    auth(Mock::given(method("POST")))
        .and(path("/v2/tokenization/mint"))
        .and(header("Idempotency-Key", "mint-key-123"))
        .and(body_json(&expected_body))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(MINT_RESPONSE_JSON, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let mint = client
        .mint_tokenized_asset(
            &TokenizationMintRequest {
                underlying_symbol: "AAPL".into(),
                qty: Decimal::new(3, 0),
                issuer: TokenizationIssuer::XStocks,
                network: TokenizationNetwork::Solana,
                wallet_address: "5dXY1aH2tQpV3wXmJg6Z7c8B4nKvF9bA1pQrSt2uVwYxXz".into(),
                client_request_id: Some("my-mint-ref-001".into()),
            },
            Some("mint-key-123"),
        )
        .await?;

    assert_eq!(
        mint.tokenization_request_id,
        "14d484e3-46f9-4e11-99ac-6fee0d4455c7"
    );
    assert_eq!(mint.status, TokenizationRequestStatus::Pending);
    assert_eq!(mint.client_request_id.as_deref(), Some("my-mint-ref-001"));

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_tokenization_requests_lists_and_filters() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("GET")))
        .and(path("/v2/tokenization/requests"))
        .and(query_param("type", "mint"))
        .and(query_param("underlying_symbol", "TSLA"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(format!("[{TOKENIZATION_REQUEST_JSON}]"), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let requests = client
        .get_tokenization_requests(&GetTokenizationRequestsRequest {
            request_type: Some(TokenizationRequestType::Mint),
            underlying_symbol: Some("TSLA".into()),
            ..Default::default()
        })
        .await?;

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].request_type, TokenizationRequestType::Mint);
    assert_eq!(requests[0].status, TokenizationRequestStatus::Completed);
    assert_eq!(requests[0].qty, Decimal::new(10, 1));

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_tokenization_request_by_client_request_id_uses_colon_path() -> Result<()> {
    let server = MockServer::start().await;

    // The colon in the path is intentional and must not be percent-encoded.
    auth(Mock::given(method("GET")))
        .and(path("/v2/tokenization/requests:by_client_request_id"))
        .and(query_param("client_request_id", "my-mint-ref-001"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(TOKENIZATION_REQUEST_JSON, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = TradingClient::with_base_url(test_credentials(), &server.uri())?;
    let request = client
        .get_tokenization_request_by_client_request_id("my-mint-ref-001")
        .await?;

    assert_eq!(
        request.tokenization_request_id,
        "5b1d6a3e-7f0a-4d2c-b8e1-9e6f1c0d2c4a"
    );
    assert_eq!(request.issuer, TokenizationIssuer::XStocks);

    server.verify().await;
    Ok(())
}
