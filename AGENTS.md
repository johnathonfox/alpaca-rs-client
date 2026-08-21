# AGENTS.md

Guidance for AI agents (Kimi Code CLI and others) working in this repository.

## Project

`alpaca-rs` — a Rust library crate for the [Alpaca](https://alpaca.markets) API
(market data and trading). Library only: source lives in `src/`, public API is
exported from `src/lib.rs`.

- Edition: 2024 (requires a recent stable toolchain)
- GitHub: `git@github.com:johnathonfox/alpaca-rs.git`, default branch `main`

## Layout

- `src/error.rs` — `Error` (thiserror) + `Result<T>`
- `src/rest.rs` — `Credentials` (env-var loading) + internal `RestClient`
  (auto-pagination via `next_page_token`, retry with backoff on 429/5xx)
- `src/trading/` — trading API (`TradingClient`, enums, models, requests;
  submodules add crypto wallets/funding, perpetuals, tokenization, locates,
  activities, watchlists-by-name)
- `src/data/` — market data API (`StockHistoricalDataClient`,
  `CryptoHistoricalDataClient`, `OptionHistoricalDataClient`, `NewsClient`,
  `ScreenerClient`, `CorporateActionsClient`, `ForexClient`, `LogoClient`,
  `FixedIncomeDataClient`, `CryptoPerpDataClient`, enums incl. `TimeFrame`,
  models, request params)
- `src/stream/` — WebSocket streams (`MarketDataStream`, `TradingStream`;
  opt-in auto-reconnect via `ReconnectOptions`) and SSE event streams
  (`ActivityEventsClient`, `CorporateActionEventsClient`)
- `src/broker.rs` — Broker API fixed-income asset lists
  (`FixedIncomeAssetsClient`; Basic Auth via `Credentials::from_broker_env`)
- `examples/quickstart.rs` — runnable demo behind env credentials;
  `examples/news.rs` — news fetch taking a symbol argument
- `tests/` — wiremock-based integration tests (offline; clients pointed at the
  mock server via each client's `with_base_url` constructor)
- `docs/adr/` — architecture decision records; `docs/sprints*.md` — build
  plans; `docs/diagrams/` — Mermaid sources

Dependencies: reqwest (rustls, json, query), serde/serde_json, thiserror,
tokio, tokio-tungstenite (rustls-tls-webpki-roots) + futures-util, url,
chrono, rust_decimal (`serde-str` — trading API numbers are JSON strings;
market-data numbers are plain `f64`), tracing. Dev-only: wiremock.

## Commands

- Build: `cargo build`
- Check (fast, no codegen): `cargo check`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Format: `cargo fmt` (check-only: `cargo fmt --check`)

Run `cargo fmt` and `cargo clippy` before considering any change done; both must
pass cleanly.

## Conventions

- Keep the crate dependency-light. Confirm a crate is genuinely needed before
  adding it to `Cargo.toml`, and prefer the version/feature set already common
  in the Rust ecosystem (e.g. `reqwest` for HTTP, `serde` for JSON, `tokio` for
  async runtime) — but only add them when the code that needs them is written.
- Never commit Alpaca API keys or secrets. Credentials belong in environment
  variables (`APCA_API_KEY_ID`, `APCA_API_SECRET_KEY`; broker client:
  `APCA_BROKER_API_KEY`, `APCA_BROKER_API_SECRET`); read them at runtime.
  `.env` files must stay out of git (add to `.gitignore` if one is introduced).
- Error handling: this is a library — return `Result` with a crate error type,
  never `panic!`/`unwrap`/`expect` in library code paths.
- Public items get doc comments (`///`) with an example where it helps; run
  `cargo doc --no-deps` to verify docs build.
- Tests: unit tests next to the code (`#[cfg(test)]` modules); integration
  tests in `tests/`. Do not hit the live Alpaca API in tests — mock or use the
  paper-trading base URL behind an ignored test.

## Git

- Do not commit, push, or otherwise mutate git history unless the user
  explicitly asks.
- Commit messages: short imperative summary (e.g. `Add market data client`).
