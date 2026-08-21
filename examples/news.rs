//! Fetches recent news for a symbol via the market data API.
//!
//! ```sh
//! export APCA_API_KEY_ID=... APCA_API_SECRET_KEY=...
//! cargo run --example news -- SOXL
//! ```

use alpaca_rs::data::{NewsClient, NewsRequest, Sort};
use alpaca_rs::rest::Credentials;
use chrono::{Duration, Utc};

#[tokio::main]
async fn main() -> alpaca_rs::Result<()> {
    let symbol = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "SOXL".to_string());
    let client = NewsClient::new(Credentials::from_env()?)?;

    // `news` auto-paginates until the result set is exhausted, so bound the
    // window — without `start` this would fetch the symbol's entire history.
    let request = NewsRequest {
        start: Some(Utc::now() - Duration::days(2)),
        sort: Some(Sort::Desc),
        ..NewsRequest::for_symbols([symbol.as_str()])
    };
    let news = client.news(&request).await?;

    println!(
        "{} article(s) mentioning {symbol} in the last 48h:",
        news.news.len()
    );
    for article in &news.news {
        let summary: String = article
            .summary
            .as_deref()
            .unwrap_or_default()
            .chars()
            .take(120)
            .collect();
        println!(
            "\n[{}] {} ({})\n  {summary}\n  {}",
            article.created_at.format("%Y-%m-%d %H:%M UTC"),
            article.headline,
            article.source,
            article.url.as_deref().unwrap_or("no url"),
        );
    }
    Ok(())
}
