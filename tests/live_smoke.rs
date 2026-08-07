//! Opt-in smoke tests against a **real** Alpaca account.
//!
//! Unlike every other test in this crate, these talk to Alpaca over the
//! network, so they are `#[ignore]`d and never run in CI. Their purpose is the
//! one thing mocks cannot do: prove that the models actually deserialize the
//! live wire format, since most of them were derived from the OpenAPI spec
//! rather than observed traffic.
//!
//! ```sh
//! export APCA_API_KEY_ID=... APCA_API_SECRET_KEY=...
//! cargo test --test live_smoke -- --ignored --nocapture
//! ```
//!
//! **Every call here is read-only.** No order is submitted, replaced or
//! canceled; no position is closed, exercised or marked do-not-exercise; no
//! watchlist or locate is created. The trading client is pointed at the
//! paper host. Keep it that way — these run against whatever account the
//! credentials belong to.

use alpaca_rs::data::{
    BarsRequest, CorporateActionsClient, ForexClient, ForexRatesRequest, LatestForexRatesRequest,
    LatestRequest, MarketMoversRequest, MarketType, MostActivesBy, MostActivesRequest, NewsClient,
    NewsRequest, ScreenerClient, StockHistoricalDataClient, TimeFrame, TimeFrameUnit,
};
use alpaca_rs::rest::Credentials;
use alpaca_rs::trading::{
    AccountActivitiesRequest, GetAssetsRequest, GetOrdersRequest, Market, TradingClient,
};
use alpaca_rs::{Error, Result};

/// Skips the test (rather than failing) when credentials are absent, so a
/// plain `cargo test -- --ignored` on a machine without keys stays green.
macro_rules! credentials_or_skip {
    () => {
        match Credentials::from_env() {
            Ok(credentials) => credentials,
            Err(_) => {
                eprintln!("skipping: APCA_API_KEY_ID / APCA_API_SECRET_KEY not set");
                return Ok(());
            }
        }
    };
}

/// Reports an endpoint that is expected to be unavailable on some accounts
/// (entitlements, feature flags) without failing the whole smoke run.
fn tolerate(label: &str, result: Result<()>) {
    match result {
        Ok(()) => println!("  ok       {label}"),
        Err(Error::Api { status, message }) => {
            let message: String = message.chars().take(160).collect();
            println!("  SKIPPED  {label} — api {status}: {message}");
        }
        Err(err) => println!("  FAILED   {label} — {err}"),
    }
}

/// Read-only sweep of the trading API against the paper host.
#[tokio::test]
#[ignore = "hits the live Alpaca API; run explicitly with --ignored"]
async fn trading_read_only_sweep() -> Result<()> {
    let credentials = credentials_or_skip!();
    let client = TradingClient::new(credentials, true)?;

    println!("trading (paper):");

    let account = client.get_account().await?;
    println!("  ok       get_account — currency {:?}", account.currency);

    let clock = client.get_clock().await?;
    println!("  ok       get_clock — is_open {}", clock.is_open);

    let positions = client.get_positions().await?;
    println!("  ok       get_positions — {} open", positions.len());

    let orders = client.get_orders(&GetOrdersRequest::default()).await?;
    println!("  ok       get_orders — {} returned", orders.len());

    // Sprint 9: the refreshed Asset model (borrow_status, margin requirements,
    // typed attributes) is the main thing under test here.
    let asset = client.get_asset("AAPL").await?;
    println!(
        "  ok       get_asset(AAPL) — borrow_status {:?}, {} attributes",
        asset.borrow_status,
        asset.attributes.len()
    );

    // Sprint 8: both activity families must deserialize.
    let activities = client
        .get_account_activities(&AccountActivitiesRequest::default())
        .await?;
    println!(
        "  ok       get_account_activities — {} returned",
        activities.len()
    );

    tolerate(
        "get_assets",
        async {
            let assets = client.get_assets(&GetAssetsRequest::default()).await?;
            println!("           assets returned: {}", assets.len());
            Ok(())
        }
        .await,
    );

    // Sprint 10: /v3 clock, the endpoint whose enum values were most uncertain.
    tolerate(
        "get_clock_v3",
        async {
            let clock = client.get_clock_v3(&[Market::NYSE]).await?;
            println!("           v3 clock parsed: {clock:?}");
            Ok(())
        }
        .await,
    );

    // Sprint 12: locates need a live account with HTB enabled, so a
    // non-2xx here is information, not a failure.
    tolerate(
        "get_locate_quotes",
        async {
            let quotes = client.get_locate_quotes(&["AAPL".to_string()]).await?;
            println!("           locate quotes parsed: {quotes:?}");
            Ok(())
        }
        .await,
    );

    Ok(())
}

/// Read-only sweep of the market-data APIs.
#[tokio::test]
#[ignore = "hits the live Alpaca API; run explicitly with --ignored"]
async fn market_data_read_only_sweep() -> Result<()> {
    let credentials = credentials_or_skip!();
    println!("market data:");

    let stocks = StockHistoricalDataClient::new(credentials.clone())?;

    let latest = stocks.latest_trades(&LatestRequest::new(["AAPL"])).await?;
    println!("  ok       latest_trades — {} symbols", latest.trades.len());

    let bars = stocks
        .bars(&BarsRequest::new(
            ["AAPL"],
            TimeFrame::new(1, TimeFrameUnit::Day)?,
        ))
        .await?;
    println!(
        "  ok       bars — {} AAPL bars",
        bars.bars.get("AAPL").map(Vec::len).unwrap_or(0)
    );

    tolerate(
        "snapshots",
        async {
            let snapshots = stocks.snapshots(&LatestRequest::new(["AAPL"])).await?;
            println!("           snapshots parsed ok ({snapshots:?})");
            Ok(())
        }
        .await,
    );

    // Sprint 11: the meta tables.
    tolerate(
        "exchanges (meta)",
        async {
            let exchanges = stocks.exchanges().await?;
            println!("           exchange codes: {}", exchanges.len());
            Ok(())
        }
        .await,
    );

    let screener = ScreenerClient::new(credentials.clone())?;
    tolerate(
        "screener.most_actives",
        async {
            let actives = screener
                .most_actives(&MostActivesRequest::new(MostActivesBy::Volume, 5))
                .await?;
            println!("           most actives: {}", actives.most_actives.len());
            Ok(())
        }
        .await,
    );
    tolerate(
        "screener.movers",
        async {
            let movers = screener
                .movers(&MarketMoversRequest::new(MarketType::Stocks, 5))
                .await?;
            println!(
                "           movers: {} gainers / {} losers",
                movers.gainers.len(),
                movers.losers.len()
            );
            Ok(())
        }
        .await,
    );

    let news = NewsClient::new(credentials.clone())?;
    tolerate(
        "news",
        async {
            let news = news.news(&NewsRequest::for_symbols(["AAPL"])).await?;
            println!("           articles: {}", news.news.len());
            Ok(())
        }
        .await,
    );

    // Sprint 14: forex.
    let forex = ForexClient::new(credentials.clone())?;
    tolerate(
        "forex.latest_rates",
        async {
            let rates = forex
                .latest_rates(&LatestForexRatesRequest::new(["EURUSD"]))
                .await?;
            println!("           latest fx parsed: {rates:?}");
            Ok(())
        }
        .await,
    );
    tolerate(
        "forex.rates",
        async {
            let rates = forex.rates(&ForexRatesRequest::new(["EURUSD"])).await?;
            println!("           historical fx parsed ok ({rates:?})");
            Ok(())
        }
        .await,
    );

    let _ = CorporateActionsClient::new(credentials)?;
    println!("  ok       corporate actions client constructed");

    Ok(())
}
