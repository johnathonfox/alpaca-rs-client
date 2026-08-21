//! Integration tests for the market-data clients against a local mock
//! server.
//!
//! Hermetic: the server is wiremock on localhost, credentials are dummy
//! values, and no env vars or live Alpaca endpoints are touched.

use alpaca_rs_client::data::{
    AuctionsRequest, BarsRequest, CorporateActionsClient, CorporateActionsRequest, CryptoFeed,
    CryptoHistoricalDataClient, CryptoPerpDataClient, CryptoPerpLatestRequest, DataFeed,
    FixedIncomeDataClient, FixedIncomeLatestQuotesRequest, FixedIncomeLatestRequest, ForexClient,
    ForexRatesRequest, LatestForexRatesRequest, LatestRequest, LogoClient, MarketMoversRequest,
    MarketType, MostActivesBy, MostActivesRequest, NewsClient, NewsRequest,
    OptionHistoricalDataClient, OptionsFeed, ScreenerClient, StockHistoricalDataClient, Tape,
    TickType, TimeFrame, TimeFrameUnit,
};
use alpaca_rs_client::rest::Credentials;
use alpaca_rs_client::{Error, Result};
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

    // `GET /v2/stocks/snapshots` returns the symbol-keyed map at the top
    // level, with no `snapshots` wrapper and no `next_page_token` — verified
    // against the live endpoint. Crypto and options nest theirs instead.
    let body = json!({
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

fn auction_json(date: &str, open: f64, close: f64) -> serde_json::Value {
    json!({
        "d": date,
        "o": [{"t": format!("{date}T13:30:00.048Z"), "x": "P", "p": open, "s": 1000, "c": "Q"}],
        "c": [{"t": format!("{date}T20:00:00.106Z"), "x": "P", "p": close, "s": 2000, "c": "M"}]
    })
}

/// Stock auctions across two pages: the client must follow the
/// `next_page_token` and merge the per-symbol lists.
#[tokio::test]
async fn stock_auctions_follows_two_pages_and_merges() -> Result<()> {
    let server = MockServer::start().await;

    let page1 = json!({
        "auctions": {"AAPL": [auction_json("2024-07-23", 223.0, 223.9)]},
        "next_page_token": "page2token"
    });
    auth(Mock::given(method("GET")))
        .and(path("/v2/stocks/auctions"))
        .and(query_param("symbols", "AAPL"))
        .and(query_param_is_missing("page_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&page1))
        .expect(1)
        .mount(&server)
        .await;

    let page2 = json!({
        "auctions": {"AAPL": [auction_json("2024-07-24", 224.0, 224.62)]},
        "next_page_token": null
    });
    auth(Mock::given(method("GET")))
        .and(path("/v2/stocks/auctions"))
        .and(query_param("symbols", "AAPL"))
        .and(query_param("page_token", "page2token"))
        .and(query_param("limit", "10000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&page2))
        .expect(1)
        .mount(&server)
        .await;

    let client = StockHistoricalDataClient::with_base_url(test_credentials(), &server.uri())?;
    let resp = client.auctions(&AuctionsRequest::new(["AAPL"])).await?;

    let days = &resp.auctions["AAPL"];
    assert_eq!(days.len(), 2, "both pages must be merged");
    assert_eq!(days[1].closing[0].price, 224.62);
    assert_eq!(days[0].opening[0].size, Some(1000.0));
    assert_eq!(resp.next_page_token, None);

    server.verify().await;
    Ok(())
}

/// The single-symbol variants put the symbol in the path and must not repeat
/// it as a `symbols` query parameter.
#[tokio::test]
async fn stock_single_symbol_endpoints_use_the_symbol_path() -> Result<()> {
    let server = MockServer::start().await;

    let bars = json!({
        "symbol": "AAPL",
        "bars": [bar_json("2024-07-24T13:30:00Z", 224.62)],
        "next_page_token": null
    });
    auth(Mock::given(method("GET")))
        .and(path("/v2/stocks/AAPL/bars"))
        .and(query_param("timeframe", "1Min"))
        .and(query_param_is_missing("symbols"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&bars))
        .expect(1)
        .mount(&server)
        .await;

    // No data in the interval: the API answers with a null list.
    let auctions = json!({"symbol": "AAPL", "auctions": null, "next_page_token": null});
    auth(Mock::given(method("GET")))
        .and(path("/v2/stocks/AAPL/auctions"))
        .and(query_param_is_missing("symbols"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&auctions))
        .expect(1)
        .mount(&server)
        .await;

    let trade = json!({
        "symbol": "AAPL",
        "trade": {"t": "2024-07-24T19:59:59.639Z", "p": 224.62, "s": 4.0,
                  "x": "Q", "i": 52983525029461_i64, "c": ["@"], "z": "C"}
    });
    auth(Mock::given(method("GET")))
        .and(path("/v2/stocks/AAPL/trades/latest"))
        .and(query_param_is_missing("symbols"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&trade))
        .expect(1)
        .mount(&server)
        .await;

    let snapshot = json!({
        "symbol": "AAPL",
        "latestTrade": {"t": "2024-07-24T19:59:59.639Z", "p": 224.62, "s": 4.0,
                        "x": "Q", "i": 52983525029461_i64, "c": ["@"], "z": "C"},
        "dailyBar": bar_json("2024-07-24T04:00:00Z", 224.62),
        "currency": "USD"
    });
    auth(Mock::given(method("GET")))
        .and(path("/v2/stocks/AAPL/snapshot"))
        .and(query_param("feed", "sip"))
        .and(query_param_is_missing("symbols"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&snapshot))
        .expect(1)
        .mount(&server)
        .await;

    let client = StockHistoricalDataClient::with_base_url(test_credentials(), &server.uri())?;

    let bars = client
        .bars_for_symbol("AAPL", &BarsRequest::new(["AAPL"], TimeFrame::MINUTE))
        .await?;
    assert_eq!(bars.symbol, "AAPL");
    assert_eq!(bars.bars.len(), 1);
    assert_eq!(bars.bars[0].close, 224.62);

    let auctions = client
        .auctions_for_symbol("AAPL", &AuctionsRequest::new(["AAPL"]))
        .await?;
    assert!(auctions.auctions.is_empty(), "null list must be empty");

    let trade = client
        .latest_trade_for_symbol("AAPL", &LatestRequest::new(["AAPL"]))
        .await?;
    assert_eq!(trade.trade.price, 224.62);

    let mut req = LatestRequest::new(["AAPL"]);
    req.feed = Some(DataFeed::Sip);
    let snapshot = client.snapshot("AAPL", &req).await?;
    assert_eq!(snapshot.symbol, "AAPL");
    assert_eq!(
        snapshot.snapshot.daily_bar.as_ref().map(|b| b.close),
        Some(224.62)
    );

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn stock_meta_conditions_and_exchanges() -> Result<()> {
    let server = MockServer::start().await;

    let conditions = json!({"@": "Regular Sale", "A": "Acquisition"});
    auth(Mock::given(method("GET")))
        .and(path("/v2/stocks/meta/conditions/trade"))
        .and(query_param("tape", "A"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&conditions))
        .expect(1)
        .mount(&server)
        .await;

    let quote_conditions = json!({"R": "Regular", "C": "Closing"});
    auth(Mock::given(method("GET")))
        .and(path("/v2/stocks/meta/conditions/quote"))
        .and(query_param("tape", "C"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&quote_conditions))
        .expect(1)
        .mount(&server)
        .await;

    let exchanges = json!({"N": "New York Stock Exchange", "V": "IEX"});
    auth(Mock::given(method("GET")))
        .and(path("/v2/stocks/meta/exchanges"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&exchanges))
        .expect(1)
        .mount(&server)
        .await;

    let client = StockHistoricalDataClient::with_base_url(test_credentials(), &server.uri())?;

    let trade_conditions = client.trade_conditions(Tape::A).await?;
    assert_eq!(trade_conditions["@"], "Regular Sale");

    let quote_conditions = client.quote_conditions(Tape::C).await?;
    assert_eq!(quote_conditions["R"], "Regular");

    let exchanges = client.exchanges().await?;
    assert_eq!(exchanges["V"], "IEX");

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn option_meta_conditions_uses_ticktype_path() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({"a": "SLAN - Single Leg Auction Non ISO"});
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/options/meta/conditions/quote"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = OptionHistoricalDataClient::with_base_url(test_credentials(), &server.uri())?;
    let conditions = client.conditions(TickType::Quote).await?;
    assert_eq!(conditions["a"], "SLAN - Single Leg Auction Non ISO");

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

fn forex_rate_json(timestamp: &str, mid: f64) -> serde_json::Value {
    json!({"t": timestamp, "bp": mid - 0.1, "ap": mid + 0.1, "mp": mid})
}

/// Forex rates across two pages: the client must send the currency pairs and
/// timeframe, pass the token back, and merge both pages per currency pair.
#[tokio::test]
async fn forex_rates_follows_two_pages_and_merges() -> Result<()> {
    let server = MockServer::start().await;

    let page1 = json!({
        "rates": {"USDJPY": [forex_rate_json("2024-07-24T00:00:00Z", 153.8)]},
        "next_page_token": "page2token"
    });
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/forex/rates"))
        .and(query_param("currency_pairs", "USDJPY,USDMXN"))
        .and(query_param("timeframe", "1Day"))
        .and(query_param_is_missing("page_token"))
        // No default page size is imposed on forex rates.
        .and(query_param_is_missing("limit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&page1))
        .expect(1)
        .mount(&server)
        .await;

    let page2 = json!({
        "rates": {
            "USDJPY": [forex_rate_json("2024-07-25T00:00:00Z", 154.2)],
            "USDMXN": [forex_rate_json("2024-07-25T00:00:00Z", 18.2)]
        },
        "next_page_token": null
    });
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/forex/rates"))
        .and(query_param("page_token", "page2token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&page2))
        .expect(1)
        .mount(&server)
        .await;

    let client = ForexClient::with_base_url(test_credentials(), &server.uri())?;
    let mut req = ForexRatesRequest::new(["USDJPY", "USDMXN"]);
    req.timeframe = Some(TimeFrame::new(1, TimeFrameUnit::Day)?);
    let resp = client.rates(&req).await?;

    let usdjpy = &resp.rates["USDJPY"];
    assert_eq!(usdjpy.len(), 2);
    assert_eq!(usdjpy[0].mid_price, 153.8);
    assert_eq!(usdjpy[1].bid_price, 154.1);
    assert_eq!(resp.rates["USDMXN"].len(), 1);
    assert_eq!(resp.next_page_token, None);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn forex_latest_rates_parses_per_pair_map() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({
        "rates": {"USDJPY": forex_rate_json("2024-07-24T12:00:00Z", 153.8)}
    });
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/forex/latest/rates"))
        .and(query_param("currency_pairs", "USDJPY"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = ForexClient::with_base_url(test_credentials(), &server.uri())?;
    let resp = client
        .latest_rates(&LatestForexRatesRequest::new(["USDJPY"]))
        .await?;

    assert_eq!(resp.rates["USDJPY"].ask_price, 153.9);

    server.verify().await;
    Ok(())
}

/// The logos endpoint answers with image bytes, not JSON: the client must
/// hand them back verbatim together with the content type, and must only send
/// `placeholder` when it is explicitly turned off.
#[tokio::test]
async fn logo_returns_image_bytes_and_content_type() -> Result<()> {
    let server = MockServer::start().await;

    // A one-pixel PNG stands in for the real 300x300 image.
    let png: Vec<u8> = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0xff];
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/logos/AAPL"))
        .and(query_param_is_missing("placeholder"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(png.clone())
                .insert_header("content-type", "image/png"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = LogoClient::with_base_url(test_credentials(), &server.uri())?;
    let logo = client.logo("AAPL").await?;

    assert_eq!(logo.bytes, png);
    assert_eq!(logo.content_type.as_deref(), Some("image/png"));

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn logo_without_placeholder_sends_flag_and_maps_404() -> Result<()> {
    let server = MockServer::start().await;

    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/logos/NOLOGO"))
        .and(query_param("placeholder", "false"))
        .respond_with(ResponseTemplate::new(404).set_body_string("logo not found"))
        .expect(1)
        .mount(&server)
        .await;

    let client = LogoClient::with_base_url(test_credentials(), &server.uri())?;
    let err = client
        .logo_without_placeholder("NOLOGO")
        .await
        .expect_err("404 must surface as an error");

    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 404);
            assert!(message.contains("logo not found"), "message: {message}");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }

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

#[tokio::test]
async fn fixed_income_latest_prices_parses_isin_map() -> Result<()> {
    let server = MockServer::start().await;

    // Verbatim fixture from the API docs.
    let body = json!({"prices":{"US912797KJ59":{"p":99.6459,"t":"2025-02-14T20:58:00.648Z","ytm":4.249,"ytw":4.249}}});
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/fixed_income/latest/prices"))
        .and(query_param("isins", "US912797KJ59,US912797SX61"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = FixedIncomeDataClient::with_base_url(test_credentials(), &server.uri())?;
    let resp = client
        .latest_prices(&FixedIncomeLatestRequest::new([
            "US912797KJ59",
            "US912797SX61",
        ])?)
        .await?;

    let price = &resp.prices["US912797KJ59"];
    assert_eq!(price.price, 99.6459);
    assert_eq!(price.yield_to_maturity, Some(4.249));

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn fixed_income_latest_quotes_parses_isin_map() -> Result<()> {
    let server = MockServer::start().await;

    // Verbatim fixture from the API docs.
    let body = json!({"quotes":{"US912797SX61":{"ams":1000,"ap":99.91958333,"as":1000000,"aytm":2.226923,"aytw":2.226923,"bms":1000,"bp":99.81091667,"bs":1000000,"bytm":5.236154,"bytw":5.236154,"t":"2026-05-21T06:56:01.882466873Z"}}});
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/fixed_income/latest/quotes"))
        .and(query_param("isins", "US912797SX61"))
        .and(query_param("trade_size", "1000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = FixedIncomeDataClient::with_base_url(test_credentials(), &server.uri())?;
    let mut req = FixedIncomeLatestQuotesRequest::new(["US912797SX61"])?;
    req.trade_size = Some(1_000_000);
    let resp = client.latest_quotes(&req).await?;

    let quote = &resp.quotes["US912797SX61"];
    assert_eq!(quote.bid_price, 99.81091667);
    assert_eq!(quote.ask_size, 1_000_000);
    assert_eq!(quote.ask_yield_to_maturity, Some(2.226923));

    server.verify().await;
    Ok(())
}

#[test]
fn fixed_income_requests_validate_isin_limits() {
    // Empty lists and lists over the endpoint's ISIN cap are rejected
    // client-side, before any request goes out.
    assert!(matches!(
        FixedIncomeLatestRequest::new(Vec::<String>::new()),
        Err(Error::InvalidRequest(_))
    ));
    let too_many_prices: Vec<String> = (0..1001).map(|i| format!("ISIN{i}")).collect();
    assert!(matches!(
        FixedIncomeLatestRequest::new(&too_many_prices),
        Err(Error::InvalidRequest(_))
    ));
    let too_many_quotes: Vec<String> = (0..101).map(|i| format!("ISIN{i}")).collect();
    assert!(matches!(
        FixedIncomeLatestQuotesRequest::new(&too_many_quotes),
        Err(Error::InvalidRequest(_))
    ));
}

#[tokio::test]
async fn crypto_perp_latest_pricing_uses_global_path() -> Result<()> {
    let server = MockServer::start().await;

    // Verbatim fixture from the API spec.
    let body = json!({"pricing":{"BTC-PERP":{"t":"2022-05-27T10:18:00Z","ft":"2022-05-27T10:18:00Z","oi":90.7367,"ip":50702.8,"mp":50652.3553,"fr":0.000565699}}});
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/crypto-perps/global/latest/pricing"))
        .and(query_param("symbols", "BTC-PERP,ETH-PERP"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = CryptoPerpDataClient::with_base_url(test_credentials(), &server.uri())?;
    let resp = client
        .latest_pricing(&CryptoPerpLatestRequest::new(["BTC-PERP", "ETH-PERP"]))
        .await?;

    let pricing = &resp.pricing["BTC-PERP"];
    assert_eq!(pricing.open_interest, 90.7367);
    assert_eq!(pricing.funding_rate, 0.000565699);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn crypto_perp_latest_orderbooks_uses_global_path() -> Result<()> {
    let server = MockServer::start().await;

    // Verbatim fixture from the API spec.
    let body = json!({"orderbooks":{"BTC-PERP":{"t":"2022-06-24T08:00:14.137774336Z","b":[{"p":20846,"s":0.1902},{"p":20350,"s":0}],"a":[{"p":20902,"s":0.0097},{"p":21444,"s":0}]}}});
    auth(Mock::given(method("GET")))
        .and(path("/v1beta1/crypto-perps/global/latest/orderbooks"))
        .and(query_param("symbols", "BTC-PERP"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = CryptoPerpDataClient::with_base_url(test_credentials(), &server.uri())?;
    let resp = client
        .latest_orderbooks(&CryptoPerpLatestRequest::new(["BTC-PERP"]))
        .await?;

    let book = &resp.orderbooks["BTC-PERP"];
    assert_eq!(book.bids.len(), 2);
    assert_eq!(book.bids[0].price, 20846.0);
    assert_eq!(book.asks[1].size, 0.0);

    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn fixed_income_forbidden_maps_to_error_api() -> Result<()> {
    let server = MockServer::start().await;

    let body = json!({"code": 40310000, "message": "forbidden"});
    Mock::given(method("GET"))
        .and(path("/v1beta1/fixed_income/latest/prices"))
        .respond_with(ResponseTemplate::new(403).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = FixedIncomeDataClient::with_base_url(test_credentials(), &server.uri())?;
    let err = client
        .latest_prices(&FixedIncomeLatestRequest::new(["US912797KJ59"])?)
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
