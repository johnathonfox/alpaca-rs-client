# alpaca-rs

An async Rust client for the [Alpaca](https://alpaca.markets) **Trading API** and
**Market Data APIs**, including their WebSocket streams. Scope matches the
Python SDK [`alpaca-py`](https://github.com/alpacahq/alpaca-py) (trading + data
+ streams); see [docs/adr/0001](docs/adr/0001-alpaca-py-parity-scope-and-architecture.md)
for the scope decision — the Broker, Connect (OAuth flow), and FIX APIs are
deliberately out of scope.

## Features

- **Trading** (`TradingClient`) — account + configurations, orders (market,
  limit, stop, stop-limit, trailing stop; simple/bracket/OCO/OTO classes),
  positions (incl. partial close and option exercise), portfolio history,
  assets, clock/calendar, watchlists, corporate action announcements, option
  contracts. Paper and live base URLs.
- **Market data** — historical + latest + snapshots for stocks
  (`StockHistoricalDataClient`), crypto (`CryptoHistoricalDataClient`), and
  options (`OptionHistoricalDataClient`), plus `NewsClient`,
  `ScreenerClient` (most actives, movers), and `CorporateActionsClient`.
  List endpoints auto-paginate (`next_page_token` is followed for you);
  request structs keep `limit`/`page_token` for manual control.
- **Streaming** (`stream` module) — `MarketDataStream` for stocks, crypto,
  options, and news channels (trades, quotes, bars, orderbooks, statuses,
  corrections), and `TradingStream` for `trade_updates`. Full auth handshake,
  subscribe/unsubscribe, and opt-in auto-reconnect with resubscription.
- **Resilience** — automatic retry with backoff on HTTP 429 / transient 5xx;
  every client accepts a `base_url` override for testing or proxies.
- No panics in library code: everything returns `alpaca_rs::Result<T>`.
  Trading numbers are `rust_decimal::Decimal` (the API string-encodes them);
  market-data numbers are `f64`.

## Credentials

Set your API keys in the environment (paper keys work for everything below):

```sh
export APCA_API_KEY_ID=...
export APCA_API_SECRET_KEY=...
```

`Credentials::from_env()` reads both variables. OAuth bearer tokens
(`Credentials::oauth(...)`) are supported as an alternative.

## Quickstart

```rust,no_run
use alpaca_rs::data::{BarsRequest, StockHistoricalDataClient, TimeFrame, TimeFrameUnit};
use alpaca_rs::rest::Credentials;
use alpaca_rs::trading::TradingClient;

#[tokio::main]
async fn main() -> alpaca_rs::Result<()> {
    let credentials = Credentials::from_env()?;

    // Market data: auto-paginated daily bars.
    let data = StockHistoricalDataClient::new(credentials.clone())?;
    let request = BarsRequest::new(["AAPL"], TimeFrame::new(1, TimeFrameUnit::Day)?);
    let bars = data.bars(&request).await?;
    println!("AAPL bars: {:?}", bars.bars.get("AAPL").map(Vec::len));

    // Trading (paper): account snapshot.
    let trading = TradingClient::new(credentials, true)?;
    let account = trading.get_account().await?;
    println!("buying power: {}", account.buying_power);
    Ok(())
}
```

See `examples/quickstart.rs` for a runnable version including a WebSocket
stream.

## WebSocket streaming

```rust,no_run
# async fn demo(credentials: alpaca_rs::rest::Credentials) -> alpaca_rs::Result<()> {
use alpaca_rs::data::DataFeed;
use alpaca_rs::stream::{MarketDataStream, Subscription};

let mut stream = MarketDataStream::stocks(DataFeed::Iex, &credentials).await?;
stream
    .subscribe(&Subscription {
        trades: vec!["AAPL".into()],
        ..Default::default()
    })
    .await?;
while let Some(messages) = stream.next().await? {
    for message in messages {
        println!("{message:?}");
    }
}
# Ok(())
# }
```

## Project layout and docs

- `src/trading/` — Trading API client, enums, models, requests
- `src/data/` — market data clients, enums (incl. `TimeFrame`), models
- `src/stream/` — WebSocket streams
- `src/rest.rs`, `src/error.rs` — shared infrastructure
- [docs/adr/](docs/adr/) — architecture decision records
- [docs/sprints.md](docs/sprints.md) — build plan
- [docs/diagrams/architecture.mmd](docs/diagrams/architecture.mmd) — module diagram

## Development

```sh
cargo build
cargo test                                  # no live API calls; mocked
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT license](LICENSE-MIT) at your option.
