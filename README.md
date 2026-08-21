# alpaca-rs

An async Rust client for the [Alpaca](https://alpaca.markets) **Trading API** and
**Market Data APIs**, including their WebSocket streams. The baseline scope
matches the Python SDK [`alpaca-py`](https://github.com/alpacahq/alpaca-py)
(trading + data + streams), extended with a curated set of endpoints the SDK
omits; see [docs/adr/0001](docs/adr/0001-alpaca-py-parity-scope-and-architecture.md),
[docs/adr/0002](docs/adr/0002-beyond-parity-coverage.md) and
[docs/adr/0003](docs/adr/0003-deferred-scope-coverage.md) for the scope
decisions — the Connect (OAuth flow) and FIX APIs are deliberately out of
scope, as is the Broker API beyond fixed-income asset discovery.

## Features

Add it with `cargo add alpaca-rs-client` (the crate is published as
[`alpaca-rs-client`](https://crates.io/crates/alpaca-rs-client); the library
name is `alpaca_rs_client`).

- **Trading** (`TradingClient`) — account + configurations, orders (market,
  limit, stop, stop-limit, trailing stop; simple/bracket/OCO/OTO classes, and
  multi-leg option spreads via `OrderClass::Mleg` with up to four `OrderLeg`s),
  positions (incl. partial close, option exercise and do-not-exercise),
  account activities, portfolio history, assets (typed `AssetAttribute`
  filter, `borrow_status`), clock/calendar (`/v2` plus the multi-market `/v3`
  clock and per-market calendar), short-sale locates (`/v1/locates` quotes,
  create, list, get), watchlists (by id or by name), corporate action
  announcements, option contracts, crypto wallets & funding (transfers,
  whitelisted addresses, fee estimates — requires Alpaca enablement),
  tokenization mints and request lookups (Authorized Participants only),
  and crypto perpetuals (beta: funding wallets, leverage, account vitals;
  perp orders go through the standard orders endpoints with
  `AssetClass::CryptoPerp`). Paper and live base URLs.
- **Market data** — historical + latest + snapshots for stocks
  (`StockHistoricalDataClient`, including opening/closing auctions,
  single-symbol variants and the `meta` condition-code/exchange tables),
  crypto (`CryptoHistoricalDataClient`), and options
  (`OptionHistoricalDataClient`, incl. `meta` conditions/exchanges), plus
  `NewsClient`, `ScreenerClient` (most actives, movers),
  `CorporateActionsClient`, `ForexClient` (historical + latest currency
  rates), `LogoClient` (raw logo images), `FixedIncomeDataClient` (beta:
  latest bond prices/quotes by ISIN), and `CryptoPerpDataClient` (beta:
  latest perp bars/trades/quotes/orderbooks/funding pricing). List endpoints
  auto-paginate
  (`next_page_token` is followed for you); request structs keep
  `limit`/`page_token` for manual control.
- **Broker** (`broker` module) — `FixedIncomeAssetsClient` for fixed-income
  asset discovery (US treasuries and corporates) on the Broker API; uses
  Basic Auth credentials and requires partner onboarding. The rest of the
  Broker API is out of scope.
- **Streaming** (`stream` module) — `MarketDataStream` for stocks, crypto,
  options, and news channels (trades, quotes, bars, orderbooks, statuses,
  corrections), and `TradingStream` for `trade_updates`. Full auth handshake,
  subscribe/unsubscribe, and opt-in auto-reconnect with resubscription.
- **Event streams** (`stream` module, Server-Sent Events) —
  `ActivityEventsClient` for account activities and
  `CorporateActionEventsClient` for corporate action insert/update/delete
  mutations. Both replay history from `since`/`since_id` before going live,
  and resume from the last event id on auto-reconnect.
- **Resilience** — automatic retry with backoff on HTTP 429 / transient 5xx;
  every client accepts a `base_url` override for testing or proxies.
- No panics in library code: everything returns `alpaca_rs_client::Result<T>`.
  Trading numbers are `rust_decimal::Decimal` (the API string-encodes them);
  market-data numbers are `f64`.

## Credentials

Set your API keys in the environment (paper keys work for everything below):

```sh
export APCA_API_KEY_ID=...
export APCA_API_SECRET_KEY=...
```

`Credentials::from_env()` reads both variables. OAuth bearer tokens
(`Credentials::oauth(...)`) are supported as an alternative. The broker
client reads `APCA_BROKER_API_KEY` / `APCA_BROKER_API_SECRET` via
`Credentials::from_broker_env()`.

## Quickstart

```rust,no_run
use alpaca_rs_client::data::{BarsRequest, StockHistoricalDataClient, TimeFrame, TimeFrameUnit};
use alpaca_rs_client::rest::Credentials;
use alpaca_rs_client::trading::TradingClient;

#[tokio::main]
async fn main() -> alpaca_rs_client::Result<()> {
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
stream, `examples/news.rs` for a minimal symbol-arg news fetch
(`cargo run --example news -- SOXL`), and `examples/crypto_price.rs` for the
latest crypto trade/quote (`cargo run --example crypto_price -- BTC/USD`).

## WebSocket streaming

```rust,no_run
# async fn demo(credentials: alpaca_rs_client::rest::Credentials) -> alpaca_rs_client::Result<()> {
use alpaca_rs_client::data::DataFeed;
use alpaca_rs_client::stream::{MarketDataStream, Subscription};

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
- `src/stream/` — WebSocket and SSE event streams
- `src/broker.rs` — Broker API fixed-income asset lists (Basic Auth)
- `src/rest.rs`, `src/error.rs` — shared infrastructure
- [docs/adr/](docs/adr/) — architecture decision records
- Build plans: [docs/sprints.md](docs/sprints.md) (wave 1, complete),
  [docs/sprints-wave2.md](docs/sprints-wave2.md) (wave 2, complete),
  [docs/sprints-wave3.md](docs/sprints-wave3.md) (wave 3, complete)
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
