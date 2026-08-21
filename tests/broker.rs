//! Integration tests for `broker::FixedIncomeAssetsClient` against a local
//! mock server.
//!
//! Hermetic: the server is wiremock on localhost, credentials are dummy
//! values, and no env vars or live Alpaca endpoints are touched.

use alpaca_rs_client::broker::{
    BondStatus, FixedIncomeAssetsClient, TreasurySubtype, UsCorporatesRequest, UsTreasuriesRequest,
};
use alpaca_rs_client::rest::Credentials;
use alpaca_rs_client::{Error, Result};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockBuilder, MockServer, ResponseTemplate};

fn test_credentials() -> Credentials {
    Credentials::BasicAuth {
        key: "broker-key".to_string(),
        secret: "broker-secret".to_string(),
    }
}

/// The Broker API authenticates with HTTP Basic Auth: `Authorization: Basic
/// base64(key:secret)` on every request.
fn auth(mock: MockBuilder) -> MockBuilder {
    mock.and(header(
        "Authorization",
        "Basic YnJva2VyLWtleTpicm9rZXItc2VjcmV0",
    ))
}

const TREASURY_JSON: &str = r#"{
    "us_treasuries": [
        {
            "cusip": "912797MU8",
            "isin": "US912797MU86",
            "bond_status": "outstanding",
            "tradable": true,
            "subtype": "bill",
            "issue_date": "2025-02-13",
            "maturity_date": "2025-03-27",
            "description": "United States Treasury 0.0%, 03/27/2025",
            "description_short": "UST 0.0% 03/27/2025",
            "close_price": 99.6839,
            "close_price_date": "2025-02-27",
            "close_yield_to_maturity": 4.214,
            "close_yield_to_worst": 4.214,
            "coupon": 0,
            "coupon_type": "zero",
            "coupon_frequency": "zero",
            "fractionable": false
        }
    ]
}"#;

#[tokio::test]
async fn get_us_treasuries_sends_basic_auth_and_filters() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("GET")))
        .and(path("/v1/assets/fixed_income/us_treasuries"))
        .and(query_param("subtype", "bill"))
        .and(query_param("bond_status", "outstanding"))
        .and(query_param("cusips", "912797MU8,912797KJ5"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(TREASURY_JSON, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = FixedIncomeAssetsClient::with_base_url(test_credentials(), &server.uri())?;
    let resp = client
        .get_us_treasuries(&UsTreasuriesRequest {
            subtype: Some(TreasurySubtype::Bill),
            bond_status: Some(BondStatus::Outstanding),
            cusips: vec!["912797MU8".to_string(), "912797KJ5".to_string()],
            isins: vec![],
        })
        .await?;

    assert_eq!(resp.us_treasuries.len(), 1);
    assert_eq!(resp.us_treasuries[0].isin, "US912797MU86");
    assert_eq!(resp.us_treasuries[0].subtype, TreasurySubtype::Bill);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn get_us_corporates_joins_isins_cusips_and_tickers() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("GET")))
        .and(path("/v1/assets/fixed_income/us_corporates"))
        .and(query_param("isins", "US037833DY22"))
        .and(query_param("cusips", "037833DY2"))
        .and(query_param("tickers", "AAPL,MSFT"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(r#"{"us_corporates": []}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = FixedIncomeAssetsClient::with_base_url(test_credentials(), &server.uri())?;
    let resp = client
        .get_us_corporates(&UsCorporatesRequest {
            isins: vec!["US037833DY22".to_string()],
            cusips: vec!["037833DY2".to_string()],
            tickers: vec!["AAPL".to_string(), "MSFT".to_string()],
            ..UsCorporatesRequest::default()
        })
        .await?;

    assert!(resp.us_corporates.is_empty());

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn forbidden_response_maps_to_api_error() -> Result<()> {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/assets/fixed_income/us_treasuries"))
        .respond_with(ResponseTemplate::new(403).set_body_raw(
            r#"{"code":40310000,"message":"forbidden."}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let client = FixedIncomeAssetsClient::with_base_url(test_credentials(), &server.uri())?;
    let err = client
        .get_us_treasuries(&UsTreasuriesRequest::default())
        .await
        .expect_err("a 403 must surface as Error::Api");
    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 403);
            assert!(message.contains("forbidden"));
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }

    server.verify().await;
    Ok(())
}
