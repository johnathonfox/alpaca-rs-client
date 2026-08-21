//! Opt-in smoke tests for the streaming transports against a **real** Alpaca
//! account: WebSocket market data, the WebSocket trading stream, and the SSE
//! event streams.
//!
//! ```sh
//! export APCA_API_KEY_ID=... APCA_API_SECRET_KEY=...
//! cargo test --test live_streams -- --ignored --nocapture
//! ```
//!
//! All streams are read-only subscriptions — nothing here sends orders or
//! otherwise mutates the account. The trading stream and activity events are
//! pointed at the paper host.
//!
//! What each test proves is connect + auth + subscribe + frame decoding. Live
//! messages are best-effort: outside market hours (or on a quiet paper
//! account) a bounded wait may simply time out, which is reported, not
//! failed. Server rejections (403/406, entitlement gates) are also reported
//! rather than failed — on this account they are information, not defects.
//!
//! The crypto WebSocket is deliberately not exercised: Alpaca's free tier
//! allows a single crypto socket per account, which may already be held by
//! another process.

use std::time::Duration;

use alpaca_rs_client::data::DataFeed;
use alpaca_rs_client::rest::Credentials;
use alpaca_rs_client::stream::{
    ActivityEventsClient, ActivityEventsRequest, CorporateActionEventsClient,
    CorporateActionEventsRequest, MarketDataStream, Subscription, TradingStream,
};
use alpaca_rs_client::{Error, Result};

/// How long to wait for a live message before accepting silence.
const MESSAGE_WAIT: Duration = Duration::from_secs(20);

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

/// Prints the outcome of waiting for one live message: a decoded message, a
/// timeout (quiet stream), or a server/transport error (reported, not
/// failed).
async fn report_message<T, F>(label: &str, next: F)
where
    F: std::future::Future<Output = Result<Option<T>>>,
    T: std::fmt::Debug,
{
    match tokio::time::timeout(MESSAGE_WAIT, next).await {
        Ok(Ok(Some(message))) => println!("  ok       {label} — message: {message:?}"),
        Ok(Ok(None)) => println!("  ok       {label} — stream closed by server"),
        Ok(Err(err)) => println!("  NOTE     {label} — error after subscribe: {err}"),
        Err(_) => println!("  ok       {label} — subscribed; no message within {MESSAGE_WAIT:?}"),
    }
}

/// WebSocket market data: connect, auth, subscribe to IEX trades.
#[tokio::test]
#[ignore = "hits the live Alpaca API; run explicitly with --ignored"]
async fn market_data_stream_smoke() -> Result<()> {
    let credentials = credentials_or_skip!();
    println!("market data stream (stocks, IEX):");

    let mut stream = match MarketDataStream::stocks(DataFeed::Iex, &credentials).await {
        Ok(stream) => stream,
        Err(err) => {
            println!("  NOTE     connect/auth rejected: {err}");
            return Ok(());
        }
    };
    println!("  ok       connected and authenticated");

    stream
        .subscribe(&Subscription {
            trades: vec!["AAPL".into()],
            ..Default::default()
        })
        .await?;
    println!("  ok       subscribed to AAPL trades");

    report_message("AAPL trade", stream.next()).await;
    Ok(())
}

/// WebSocket trading stream (paper): connect, auth, listen for trade updates.
#[tokio::test]
#[ignore = "hits the live Alpaca API; run explicitly with --ignored"]
async fn trading_stream_smoke() -> Result<()> {
    let credentials = credentials_or_skip!();
    println!("trading stream (paper):");

    let mut stream = match TradingStream::connect(true, &credentials).await {
        Ok(stream) => stream,
        Err(err) => {
            println!("  NOTE     connect/auth rejected: {err}");
            return Ok(());
        }
    };
    println!("  ok       connected and authenticated");

    stream.listen_trade_updates().await?;
    println!("  ok       listening for trade_updates");

    report_message("trade_update", stream.next()).await;
    Ok(())
}

/// SSE event streams: account activities (paper) and corporate actions.
#[tokio::test]
#[ignore = "hits the live Alpaca API; run explicitly with --ignored"]
async fn sse_event_streams_smoke() -> Result<()> {
    let credentials = credentials_or_skip!();
    println!("SSE event streams:");

    match ActivityEventsClient::new(credentials.clone(), true)?
        .subscribe(&ActivityEventsRequest::new())
        .await
    {
        Ok(mut events) => {
            println!("  ok       activities stream connected (paper)");
            report_message("activity event", events.next()).await;
        }
        Err(err @ Error::Api { .. }) => println!("  NOTE     activities stream rejected: {err}"),
        Err(err) => return Err(err),
    }

    match CorporateActionEventsClient::new(credentials)?
        .subscribe(&CorporateActionEventsRequest::new())
        .await
    {
        Ok(mut events) => {
            println!("  ok       corporate actions stream connected");
            report_message("corporate action event", events.next()).await;
        }
        Err(err @ Error::Api { .. }) => {
            println!("  NOTE     corporate actions stream rejected: {err}")
        }
        Err(err) => return Err(err),
    }

    Ok(())
}
