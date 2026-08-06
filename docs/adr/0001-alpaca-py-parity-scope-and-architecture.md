# ADR 0001: Rust port of alpaca-py trading + market data APIs — scope and architecture

- Status: accepted
- Date: 2026-08-05

## Context / Problem

We need a Rust client library (`alpaca-rs`) that matches the scope of the Python
SDK `alpaca-py` for the **Trading API** and **Market Data APIs**, including their
WebSocket streams. Two adjacent Alpaca products — the **Connect API** (OAuth 2.0
for third-party apps) and the **FIX API** (institutional order routing) — had to
be verified as in or out of scope.

Research was done against `alpaca-py` master (`95ae23d`) and the official Alpaca
docs. Key findings:

- `alpaca-py` has exactly four top-level modules: `trading`, `data`, `broker`,
  `common`. There is **no Connect/OAuth client and no FIX code anywhere** in the
  package. Connect is an OAuth *program* (authorize/token/revoke flows) for apps
  acting on behalf of users; alpaca-py's only concession to it is accepting an
  optional `oauth_token` sent as `Authorization: Bearer`. The FIX API is a
  separate institutional product accessed through sales, not part of the SDK.
- The current `alpaca-rs` repo already compiles cleanly with: a complete-looking
  `TradingClient` (24 endpoint methods), four market-data clients (stock,
  crypto, option, news), and both WebSocket families (market data + trading
  stream). Real gaps: no auto-pagination, no `ScreenerClient`, no
  `CorporateActionsClient` (data API), no stream auto-reconnect, shallow
  trade-correction/cancel-error stream messages, no mock-based tests, no
  README/examples/crate metadata, no `base_url` override, no 429 retry.

## Options Considered

### Scope

1. **Exact alpaca-py parity (trading + data + streams)** — cover what the SDK
   covers, no more. Pros: clear finish line, no speculative surface. Cons: omits
   a few REST endpoints the API has but the SDK doesn't (stock auctions, option
   historical quotes).
2. **Parity plus REST-complete extras** — also wrap auctions, historical
   orderbooks, account activities, etc. Pros: more complete. Cons: moving
   target, unverifiable against a reference implementation, more code to
   maintain.
3. **Full Alpaca surface incl. Broker/Connect/FIX** — Pros: one crate for
   everything. Cons: Broker/Connect/FIX are distinct products with distinct
   credentials; huge scope; explicitly not requested.

### Market-data stream encoding

1. **JSON** (current implementation) — Alpaca's data WS accepts JSON; works
   today with `serde_json` + `tokio-tungstenite`. Pros: zero new deps.
2. **MessagePack** (what alpaca-py uses) — marginally smaller frames; requires
   `rmp-serde`. Cons: new dependency for no functional gain.

### Pagination

1. **Eager auto-pagination** like alpaca-py (loop `next_page_token`, return
   merged results). Pros: parity, simple. Cons: unbounded memory on huge
   ranges.
2. **Lazy stream/iterator of pages** — Pros: Rust-idiomatic, bounded memory.
   Cons: more complex; diverges from SDK ergonomics.
3. **Both**: eager convenience + explicit `page_token` passthrough on requests.

## Decision

1. **Scope = option 1, exact alpaca-py parity** for `trading` + `data` +
   streams. **Connect and FIX are verified OUT of scope** — they are not part of
   alpaca-py; Connect appears only as the already-implemented
   `Credentials::OAuth` bearer-token branch in `src/rest.rs`. Broker API is also
   out. Parity work is therefore limited to: add `ScreenerClient`
   (`/v1beta1/screener/...`: most-actives, movers) and `CorporateActionsClient`
   (`/v1/corporate-actions`), add auto-pagination, and harden streams.
2. **Streams stay JSON** (option 1). MessagePack adds a dependency for no
   user-visible benefit; revisit only if Alpaca deprecates JSON on the data WS.
3. **Pagination = option 3**: a shared `RestClient::get_paginated` helper that
   follows `next_page_token` and merges symbol-keyed maps (parity with the SDK),
   while request structs keep `limit`/`page_token` fields for manual control.
4. **Architecture stays as-is**: `src/trading/`, `src/data/`, `src/stream/`
   with shared `Credentials`/`RestClient`/`Error` infrastructure; `Decimal`
   (`serde-str`) for trading numbers, `f64` for market-data numbers; library
   code never panics; no new runtime dependencies except a dev-only HTTP mock
   (`wiremock`) for tests. Clients gain a `base_url` override (parity with
   `url_override` in alpaca-py) to enable mock-based testing.
5. **Stream hardening**: add opt-in auto-reconnect with resubscription, and
   fully model trade corrections (`c`) and cancel errors (`x`).
6. **Resilience**: `RestClient` retries HTTP 429 (and transient 5xx) with
   backoff, mirroring alpaca-py's default (3 attempts).

## Consequences

### Positive

- A verifiable finish line: every public client method can be checked against
  `alpaca-py` master.
- The existing ~3,600 lines of working code are reused; the work is gap closure,
  not a rewrite.
- Mock-based integration tests become possible without hitting Alpaca.

### Negative

- Users who need auctions, account activities, or Broker/Connect/FIX must look
  elsewhere or ask for a follow-up ADR.
- JSON-only streams diverge from alpaca-py's wire encoding (though not from
  anything Alpaca guarantees).
- Eager pagination can hold large result sets in memory; mitigated by manual
  `page_token` control.

### Follow-ups (out of scope, future ADRs)

- Broker API crate/module; Connect OAuth flow helpers; auctions and other
  REST-complete extras; MessagePack stream encoding.
