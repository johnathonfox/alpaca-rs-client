//! Fetches the latest trade and quote for a crypto pair.
//!
//! ```sh
//! export APCA_API_KEY_ID=... APCA_API_SECRET_KEY=...
//! cargo run --example crypto_price -- BTC/USD
//! ```

use alpaca_rs::data::{CryptoFeed, CryptoHistoricalDataClient, LatestRequest};
use alpaca_rs::rest::Credentials;

#[tokio::main]
async fn main() -> alpaca_rs::Result<()> {
    let symbol = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "BTC/USD".to_string());
    let client = CryptoHistoricalDataClient::new(Credentials::from_env()?, CryptoFeed::Us)?;

    let request = LatestRequest::new([symbol.as_str()]);
    let trades = client.latest_trades(&request).await?;
    let quotes = client.latest_quotes(&request).await?;

    if let Some(trade) = trades.trades.get(&symbol) {
        println!(
            "{symbol} last trade: ${:.2} (size {}, {})",
            trade.price, trade.size, trade.timestamp
        );
    }
    if let Some(quote) = quotes.quotes.get(&symbol) {
        println!(
            "{symbol} quote: ${:.2} bid x {} / ${:.2} ask x {} ({})",
            quote.bid_price, quote.bid_size, quote.ask_price, quote.ask_size, quote.timestamp
        );
    }
    Ok(())
}
