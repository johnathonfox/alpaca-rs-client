# ADR 0002: Coverage beyond alpaca-py parity

- Status: proposed
- Date: 2026-08-05
- Supersedes: the follow-ups section of [ADR 0001](0001-alpaca-py-parity-scope-and-architecture.md)

## Context / Problem

ADR 0001 fixed the crate's scope at exact alpaca-py parity (trading + market
data + streams). A post-completion audit against Alpaca's current OpenAPI specs
(Trading 2.0.1, Market Data 1.1 / docs versions 1.4 and 1.4.2) found documented
surface that parity excludes — including one latent defect: `OrderClass::Mleg`
exists but `OrderRequest` lacks `legs`/`position_intent`, so multi-leg option
orders cannot actually be submitted.

## Options Considered

1. **Stay at parity** — smallest surface, but leaves mleg broken and omits
   high-value endpoints (account activities) that typical users expect.
2. **Parity + all documented endpoints** — includes crypto wallets/funding,
   tokenization, fixed income, and crypto perps: niche or pre-GA products that
   would roughly double the surface for little algo-trading value.
3. **Parity + curated extension** — add everything of clear value to an
   algo-trading user, defer the niche/pre-GA products.

## Decision

Option 3. Implement, in priority order (specs in
[docs/sprints-wave2.md](../sprints-wave2.md)):

- **Fix**: mleg orders (`legs`, `position_intent` on `OrderRequest`).
- **Add (high)**: account activities (`GET /v2/account/activities[/{type}]`).
- **Add (medium)**: stock auctions, do-not-exercise, `GET /v3/clock` +
  `GET /v3/calendar/{market}`, short-sale locates (`/v1/locates*`), SSE event
  streams (activities, corporate-actions).
- **Add (low)**: single-symbol stock endpoints, meta/conditions +
  meta/exchanges tables, watchlists `:by_name`, forex rates, logos, extra
  order-list filters, asset model refresh (`borrow_status`,
  `margin_requirement_long/short`, `attributes` query filter, typed
  `AssetAttribute`).
- **Defer (revisit when GA or requested)**: crypto wallets/funding,
  tokenization, fixed income, crypto perpetuals.

Additional rulings:

- **SSE transport**: the events streams are SSE, not WebSocket. Implement with
  `reqwest` byte streaming + a small line parser; only adopt an
  `eventsource-stream` dependency if the parser grows past ~100 lines.
- **Locates live under `/v1/locates`** on the trading host — a second base
  path, handled inside `TradingClient` rather than a new client.
- **Verified non-existent** (no work): options historical quotes, historical
  orderbooks, market index values.

## Consequences

### Positive

- mleg orders work; account activities close the biggest functional hole.
- Scope stays defensible: each addition maps to a documented, generally
  available endpoint with clear user value.

### Negative

- The crate now exceeds alpaca-py; "what's in scope" is governed by this ADR
  rather than a simple parity rule, so future audits must diff against the
  OpenAPI specs, not the Python SDK.
- SSE adds a third transport style alongside REST and WebSocket.
