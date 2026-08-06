//! Integration tests for the market-data clients against a local mock
//! server.
//!
//! Hermetic: the server is wiremock on localhost, credentials are dummy
//! values, and no env vars or live Alpaca endpoints are touched.

use alpaca_rs::data::{
    BarsRequest, CorporateActionsClient, CorporateActionsRequest, CryptoFeed,
    CryptoHistoricalDataClient, LatestRequest, MarketMoversRequest, MarketType, MostActivesBy,
    MostActivesRequest, NewsClient, NewsRequest, OptionHistoricalDataClient, OptionsFeed,
    ScreenerClient, StockHistoricalDataClient, TimeFrame,
};
use alpaca_rs::rest::Credentials;
use alpaca_rs::{Error, Result};
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockBuilder, MockServer, ResponseTemplate};

fn test_credentials() -> Credentials {
    Credentials::Key {
        key_id: "test-key".to_string(),
        secret_key: "test-secret".to_string(),
    }
}

/// The standard auth-header matchers every data request must carry.
fn auth(mock: MockBuilder) -> MockBuilder {
    mock.and(header("APCA-API-KEY-ID", "test-key"))
        .and(header("APCA-API-SECRET-KEY", "test-secret"))
}

fn bar_json(timestamp: &str, close: f64) -> serde_json::Value {
    json!({
        "t": timestamp,
        "o": close - 1.0,
        "h": close + 1.0,
        "l": close - 2.0,
        "c": close,
        "v": 1000.0,
        "n": 42,
        "vw": close
    })
}

/// Stock bars across two pages: the first response carries a
/// `next_page_token`, the second doesn't; the client must merge both pages
/// and pass the token back as the `page_token` query parameter.
#[tokio::test]
async fn stock_bars_follows_two_pages_and_merges() -> Result<()> {
    let server = MockServer::start().await;

    // Matches only the first request (no `page_token` yet); the page-2 mock
    // below matches the follow-up request that carries the token.
    let page1 = json!({
        "bars": {"AAPL": [bar_json("2024-07-24T13:30:00Z", 224.5), bar_json("2024-07-24T13:31:00Z", 224.75)]},
        "next_page_token": "page2token"
    });
    auth(Mock::given(method("GET")))
        .and(path("/v2/stocks/bars"))
        .and(query_param("symbols", "AAPL"))
        .and(query_param_is_missing("page_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&page1))
        .expect(1)
        .mount(&server)
        .await;

    let page2 = json!({
        "bars": {"AAPL": [bar_json("2024-07-24T13:32:00Z", 225.0)]},
        "next_page_token": null
    });
    auth(Mock::given(method("GET")))
        .and(path("/v2/stocks/bars"))
        .and(query_param("symbols", "AAPL"))
        .and(query_param("page_token", "page2token"))
        // The default page size (10_000) is applied to every page request.
        .and(query_param("limit", "10000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&page2))
        .expect(1)
        .mount(&server)
        .await;

    let client = StockHistoricalDataClient::with_base_url(test_credentials(), &server.uri())?;
    let resp = client
        .bars(&BarsRequest::new(["AAPL"], TimeFrame::MINUTE))
        .await?;

    let bars = &resp.bars["AAPL"];
    assert_eq!(bars.len(), 3, "both pages must be merged");
    assert_eq!(bars[2].close, 225.0);
    assert_eq!(bars[0].trade_count, Some(42));
    assert_eq!(resp.next_page_token, None);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn stock_latest_trades_parses_per_symbol_map() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({
        "trades": {
            "AAPL": {"t": "2024-07-24T19:59:59.639Z", "p": 224.62, "s": 4.0,
                     "x": "Q", "i": 52983525029461_i64, "c": ["@"], "z": "C"}
        }
    });
    auth(Mock::given(method("GET")))
        .and(path("/v2/stocks/trades/latest"))
        .and(query_param("symbols", "AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = StockHistoricalDataClient::with_base_url(test_credentials(), &server.uri())?;
    let resp = client.latest_trades(&LatestRequest::new(["AAPL"])).await?;

    let trade = &resp.trades["AAPL"];
    assert_eq!(trade.price, 224.62);
    assert_eq!(trade.conditions, vec!["@"]);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn stock_snapshots_parses_single_page() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({
        "snapshots": {
            "AAPL": {
                "latestTrade": {"t": "2024-07-24T19:59:59.639Z", "p": 224.62, "s": 4.0,
                                "x": "Q", "i": 52983525029461_i64, "c": ["@"], "z": "C"},
                "latestQuote": {"t": "2024-07-24T19:59:59.639Z", "bp": 224.6, "bs": 3.0,
                                "ap": 224.65, "as": 5.0, "bx": "P", "ax": "Q",
                                "c": ["R"], "z": "C"},
                "minuteBar": bar_json("2024-07-24T19:59:00Z", 224.62),
                "dailyBar": bar_json("2024-07-24T04:00:00Z", 224.62),
                "prevDailyBar": bar_json("2024-07-23T04:00:00Z", 223.9)
            }
        },
        "next_page_token": null
    });
    auth(Mock::given(method("GET")))
        .and(path("/v2/stocks/snapshots"))
        .and(query_param("symbols", "AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = StockHistoricalDataClient::with_base_url(test_credentials(), &server.uri())?;
    let resp = client.snapshots(&LatestRequest::new(["AAPL"])).await?;

    let snap = &resp.snapshots["AAPL"];
    assert_eq!(
        snap.latest_quote.as_ref().map(|q| q.ask_price),
        Some(224.65)
    );
    assert!(snap.minute_bar.is_some());
    assert_eq!(resp.next_page_token, None);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn screener_most_actives_and_movers() -> Result<()> {
    let server = MockServer::start().await;

    let most_actives = json!({
        "most_actives": [
            {"symbol": "AAPL", "volume": 70451379.0, "trade_count": 486549.0},
            {"symbol": "TSLA", "volume": 48613142.0, "trade_count": 321456.0}
        ],
        "last_updated": "2024-07-24T14:13:00.163514169Z"
    });
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/screener/stocks/most-actives"))
        .and(query_param("by", "volume"))
        .and(query_param("top", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&most_actives))
        .expect(1)
        .mount(&server)
        .await;

    let movers = json!({
        "gainers": [
            {"change": 59.75, "percent_change": 1832.9, "price": 63.0125, "symbol": "BOIL"}
        ],
        "losers": [
            {"change": -0.0706, "percent_change": -95.02, "price": 0.0037, "symbol": "FMIVW"}
        ],
        "market_type": "stocks",
        "last_updated": "2024-07-24T14:13:00.163514169Z"
    });
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/screener/stocks/movers"))
        .and(query_param("top", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&movers))
        .expect(1)
        .mount(&server)
        .await;

    let client = ScreenerClient::with_base_url(test_credentials(), &server.uri())?;

    let actives = client
        .most_actives(&MostActivesRequest::new(MostActivesBy::Volume, 2))
        .await?;
    assert_eq!(actives.most_actives.len(), 2);
    assert_eq!(actives.most_actives[0].symbol, "AAPL");

    let movers = client
        .movers(&MarketMoversRequest::new(MarketType::Stocks, 2))
        .await?;
    assert_eq!(movers.gainers[0].symbol, "BOIL");
    assert_eq!(movers.losers[0].percent_change, -95.02);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn corporate_actions_parses_grouped_response() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({
        "corporate_actions": {
            "cash_dividends": [
                {
                    "id": "a7932b15-f816-4f83-921e-998b524487b4",
                    "symbol": "FCF",
                    "cusip": "CUSIP1",
                    "rate": 0.125,
                    "special": false,
                    "foreign": false,
                    "process_date": "2023-05-19",
                    "ex_date": "2023-05-04",
                    "record_date": "2023-05-05",
                    "payable_date": "2023-05-19"
                }
            ],
            "forward_splits": [
                {
                    "id": "a7932b15-f816-4f83-921e-998b524487b1",
                    "symbol": "SRE",
                    "cusip": "CUSIP2",
                    "new_rate": 2.0,
                    "old_rate": 1.0,
                    "process_date": "2023-08-22",
                    "ex_date": "2023-08-22",
                    "record_date": "2023-08-14",
                    "payable_date": "2023-08-21",
                    "due_bill_redemption_date": "2023-08-23"
                }
            ]
        },
        "next_page_token": null
    });
    auth(Mock::given(method("GET")))
        .and(path("/v1/corporate-actions"))
        .and(query_param("symbols", "FCF"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = CorporateActionsClient::with_base_url(test_credentials(), &server.uri())?;
    let resp = client
        .corporate_actions(&CorporateActionsRequest::for_symbols(["FCF"]))
        .await?;

    let actions = &resp.corporate_actions;
    assert_eq!(actions.cash_dividends.len(), 1);
    assert_eq!(actions.cash_dividends[0].rate, 0.125);
    assert_eq!(actions.forward_splits.len(), 1);
    assert_eq!(actions.forward_splits[0].new_rate, 2.0);
    // Missing groups deserialize as empty lists.
    assert!(actions.reverse_splits.is_empty());
    assert_eq!(resp.next_page_token, None);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn crypto_latest_bars_parses_response() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({
        "bars": {"BTC/USD": bar_json("2024-07-24T19:59:00Z", 65750.5)}
    });
    auth(Mock::given(method("GET")))
        .and(path("/v1beta3/crypto/us/latest/bars"))
        .and(query_param("symbols", "BTC/USD"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = CryptoHistoricalDataClient::with_base_url(
        test_credentials(),
        CryptoFeed::Us,
        &server.uri(),
    )?;
    let resp = client.latest_bars(&LatestRequest::new(["BTC/USD"])).await?;

    assert_eq!(resp.bars["BTC/USD"].close, 65750.5);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn option_snapshots_parses_response() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({
        "snapshots": {
            "AAPL250117C00150000": {
                "latestTrade": {"t": "2024-07-24T19:59:58.1Z", "p": 74.35, "s": 1.0,
                                "x": "A", "c": ["aa"], "i": 100_i64},
                "latestQuote": {"t": "2024-07-24T19:59:59.9Z", "bp": 74.2, "bs": 10.0,
                                "ap": 74.5, "as": 12.0, "bx": "A", "ax": "B"}
            }
        },
        "next_page_token": null
    });
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/options/snapshots"))
        .and(query_param("symbols", "AAPL250117C00150000"))
        .and(query_param("feed", "opra"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = OptionHistoricalDataClient::with_base_url(test_credentials(), &server.uri())?;
    let resp = client
        .snapshots(
            &LatestRequest::new(["AAPL250117C00150000"]),
            Some(OptionsFeed::Opra),
        )
        .await?;

    let snap = &resp.snapshots["AAPL250117C00150000"];
    assert_eq!(snap.latest_trade.as_ref().map(|t| t.price), Some(74.35));
    assert_eq!(resp.next_page_token, None);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn news_parses_articles() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({
        "news": [
            {
                "id": 40858117_u64,
                "headline": "Apple Reports Third Quarter Results",
                "author": "Benzinga Newsdesk",
                "created_at": "2024-08-01T20:31:06Z",
                "updated_at": "2024-08-01T20:31:07Z",
                "summary": "Apple posted quarterly results.",
                "url": "https://example.com/article",
                "symbols": ["AAPL"],
                "source": "benzinga"
            }
        ],
        "next_page_token": null
    });
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/news"))
        .and(query_param("symbols", "AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = NewsClient::with_base_url(test_credentials(), &server.uri())?;
    let resp = client.news(&NewsRequest::for_symbols(["AAPL"])).await?;

    assert_eq!(resp.news.len(), 1);
    assert_eq!(resp.news[0].headline, "Apple Reports Third Quarter Results");
    assert_eq!(resp.news[0].symbols, vec!["AAPL"]);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn unprocessable_entity_maps_to_error_api() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({"code": 42210000, "message": "invalid timeframe"});
    Mock::given(method("GET"))
        .and(path("/v2/stocks/bars"))
        .respond_with(ResponseTemplate::new(422).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = StockHistoricalDataClient::with_base_url(test_credentials(), &server.uri())?;
    let err = client
        .bars(&BarsRequest::new(["AAPL"], TimeFrame::MINUTE))
        .await
        .expect_err("422 must surface as an error");

    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 422);
            assert!(message.contains("invalid timeframe"), "message: {message}");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }

    server.verify().await;
    Ok(())
}
