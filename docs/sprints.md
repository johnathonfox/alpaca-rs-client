# Sprint Plan — alpaca-rs parity with alpaca-py (trading + data + streams)

Wave 1 (below) is complete. Wave 2 (beyond-parity additions) is planned in
[docs/sprints-wave2.md](sprints-wave2.md).

Reference: [ADR 0001](adr/0001-alpaca-py-parity-scope-and-architecture.md).
Each sprint is one subagent-sized chunk: independent, reviewable, and verifiable
with `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`,
`cargo fmt --check`. Sprints are ordered so earlier ones unblock later ones;
1→2→3 must run sequentially, 4 and 5 can run in parallel after 3, 6 is last.

Conventions (from AGENTS.md): library only, no `unwrap`/`expect`/`panic!` in
library paths, doc comments on public items, unit tests next to code, mock HTTP
for integration tests, `Decimal`+`serde-str` for trading numbers / `f64` for
market data.

## Sprint 1 — Pagination infrastructure

- Add `RestClient::get_paginated` (crate-private): follows `next_page_token`,
  merges symbol-keyed maps and flat lists, mirrors alpaca-py page sizes
  (10,000 bars/trades/quotes; 1,000 options snapshots & corporate actions;
  50 news).
- Apply it to all list-returning data client methods (`src/data/*.rs`) so they
  return complete result sets; keep `limit`/`page_token` on request structs for
  manual control.
- Acceptance: unit tests for the merge logic; `cargo test` green; no public API
  breakage beyond return-shape changes required by merging.

## Sprint 2 — ScreenerClient + CorporateActionsClient

- `src/data/screener.rs`: `ScreenerClient` —
  `GET /v1beta1/screener/stocks/most-actives` (`by` = volume|trades, `top`),
  `GET /v1beta1/screener/{stocks|crypto}/movers` (`top`). Models: `Movers`,
  `Mover`, `MostActives`, `MostActive`; enums `MarketType`, `MostActivesBy`.
- `src/data/corporate_actions.rs`: `CorporateActionsClient` —
  `GET /v1/corporate-actions` (symbols, cusips, types, start/end, ids),
  `CorporateAction` model + `CorporateActionsType` enum. Paginated via Sprint 1.
- Wire both into `src/data/mod.rs` and `src/lib.rs` exports.
- Acceptance: serde unit tests (round-trip from fixture JSON captured from
  alpaca-py model shapes); clippy/fmt clean.

## Sprint 3 — Resilience and configurability in `src/rest.rs`

- `base_url` override on all public clients (parity with alpaca-py
  `url_override`), keeping current constructors working.
- Retry with exponential backoff on HTTP 429 and transient 5xx (default 3
  attempts), tracing on retry.
- Acceptance: existing unit tests still pass; new tests for retry decision
  logic (no real HTTP); clippy/fmt clean.

## Sprint 4 — Market-data stream hardening (`src/stream/data.rs`)

- Fully model trade corrections (`c`) and cancel errors (`x`) with all Alpaca
  fields; parse typed subscription confirmations instead of lumping into
  `Unknown`.
- Opt-in auto-reconnect: on unexpected close/error, reconnect with backoff,
  re-auth, and resubscribe the last subscription set. Off by default;
  documented on `MarketDataStream`.
- Acceptance: unit tests for new message parsing and for the resubscribe
  bookkeeping; no behavior change when reconnect is disabled.

## Sprint 5 — Integration tests with mock HTTP (`tests/`)

- Add `wiremock` as a **dev-dependency** only.
- `tests/trading.rs`, `tests/data.rs`: mount mock endpoints for a
  representative method per client group (submit order, get account, bars
  incl. two-page pagination, snapshots, screener, corporate actions); assert
  request shape (path, query, auth headers) and response parsing, using the
  Sprint 3 `base_url` override. No live API calls.
- Acceptance: `cargo test` green offline; coverage of every public client's
  happy path plus one error (non-2xx → `Error::Api`).

## Sprint 6 — Packaging and docs

- ~~Cargo.toml metadata~~ (done: description, license, repository, keywords,
  categories) and ~~`LICENSE-MIT` / `LICENSE-APACHE` files~~ (done).
- README.md: feature overview, install, quickstart (trading + data + stream),
  paper-vs-live, credential env vars, links to docs/adr and docs/diagrams.
- One runnable example (`examples/quickstart.rs`) behind env-var credentials.
- `cargo doc --no-deps` builds warning-free.
