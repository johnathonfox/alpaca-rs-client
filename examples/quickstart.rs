//! Quickstart: fetch recent daily bars, print the paper account, then stream
//! trades for the same symbol.
//!
//! Requires `APCA_API_KEY_ID` and `APCA_API_SECRET_KEY` in the environment
//! (paper keys work). Run with:
//!
//! ```sh
//! cargo run --example quickstart
//! ```

use alpaca_rs::data::{BarsRequest, DataFeed, StockHistoricalDataClient, TimeFrame};
use alpaca_rs::rest::Credentials;
use alpaca_rs::stream::{DataMessage, MarketDataStream, Subscription};
use alpaca_rs::trading::TradingClient;
use chrono::{Duration, Utc};

const SYMBOL: &str = "AAPL";

#[tokio::main]
async fn main() -> alpaca_rs::Result<()> {
    let credentials = match Credentials::from_env() {
        Ok(credentials) => credentials,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };

    // Market data: daily bars for the last 7 days (auto-paginated).
    let data = StockHistoricalDataClient::new(credentials.clone())?;
    let mut request = BarsRequest::new([SYMBOL], TimeFrame::DAY);
    request.start = Some(Utc::now() - Duration::days(7));
    request.feed = Some(DataFeed::Iex);
    let bars = data.bars(&request).await?;
    println!("{SYMBOL} daily bars (last 7 days):");
    for bar in bars.bars.get(SYMBOL).map(Vec::as_slice).unwrap_or_default() {
        println!(
            "  {} close={:.2} volume={:.0}",
            bar.timestamp.date_naive(),
            bar.close,
            bar.volume
        );
    }

    // Trading (paper): account snapshot.
    let trading = TradingClient::new(credentials.clone(), true)?;
    let account = trading.get_account().await?;
    println!(
        "paper account {}: status={} cash={} buying_power={}",
        account.account_number, account.status, account.cash, account.buying_power
    );

    // Streaming: subscribe to trades and print the first few messages.
    let mut stream = MarketDataStream::stocks(DataFeed::Iex, &credentials).await?;
    stream
        .subscribe(&Subscription {
            trades: vec![SYMBOL.into()],
            ..Default::default()
        })
        .await?;
    println!("streaming {SYMBOL} trades (first 5)...");
    let mut printed = 0;
    while printed < 5 {
        let Some(messages) = stream.next().await? else {
            println!("stream closed");
            break;
        };
        for message in messages {
            match &message {
                DataMessage::Trade(trade) => {
                    println!(
                        "  trade {}: {:.2} x {:.0}",
                        trade.symbol, trade.price, trade.size
                    );
                    printed += 1;
                }
                _ => println!("  {message:?}"),
            }
        }
    }

    println!("done");
    Ok(())
}
