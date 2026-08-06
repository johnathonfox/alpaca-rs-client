# Sprint Plan — Wave 2: beyond alpaca-py parity

Reference: [ADR 0002](adr/0002-beyond-parity-coverage.md) (scope decision),
[ADR 0001](adr/0001-alpaca-py-parity-scope-and-architecture.md) (base
architecture). Same conventions and verification bar as wave 1
(`cargo build` / `test` / `clippy --all-targets -- -D warnings` / `fmt`,
no live API in tests, unit tests next to code, wiremock for HTTP-level tests).

Ordering: 7→8→9 first (highest value, all trading-side), then 10–13 in any
order (independent modules), 14 last (grab-bag). Deferred per ADR 0002: crypto
wallets/funding, tokenization, fixed income, crypto perps.

## Sprint 7 — Fix multi-leg options orders

**Bug**: `OrderClass::Mleg` exists (`src/trading/enums.rs`) but
`OrderRequest` (`src/trading/requests.rs`) has no `legs`/`position_intent`, so
spreads can't be submitted.

- `src/trading/requests.rs`: add `OrderLeg { symbol, side: OrderSide,
  ratio_qty: Decimal, position_intent: Option<PositionIntent> }` (PositionIntent
  already exists in enums) and `position_intent: Option<PositionIntent>` on
  `OrderRequest`; add `legs: Option<Vec<OrderLeg>>` (max 4, validate in a
  constructor or `submit_order`). Serialize per the spec's CreateOrderRequest.
- `src/trading/mod.rs`: validate — `legs` required iff `order_class == Mleg`,
  absent otherwise; reject >4 legs.
- Also add the documented `GetOrdersRequest` filters: `asset_class`,
  `before_order_id`, `after_order_id`.
- Tests: serde round-trip of a 2-leg spread request (ratio_qty as JSON string);
  validation errors; new query filters serialize. Wiremock: POST `/v2/orders`
  with mleg body asserted.

## Sprint 8 — Account activities

- `src/trading/activities.rs` (or extend mod.rs): `get_account_activities(
  &AccountActivitiesRequest)` → `GET /v2/account/activities`;
  `get_account_activities_by_type(type, &req)` → `GET
  /v2/account/activities/{activity_type}`.
- `ActivityType` enum (~37 values: FILL, TRANS, MISC, ACATC, ACATS, CFEE, CGD,
  CSD, CSW, DIV, DIVCGL/S, DIVFEE, DIVFT, DIVNRA, DIVROC, DIVTW, DIVTXEX, FEE,
  INT, INTNRA, INTTW, JNL, JNLC, JNLS, MA, NC, OPASN, OPCA, OPCSH, OPEXC,
  OPEXP, OPTRD, PTC, PTR, REORG, SPIN, SPLIT, FOPT) — verify against the spec.
- Models: `TradeActivity` and `NonTradeActivity` (per-type fields; use
  `#[serde(untagged)]` or a tagged enum keyed on `activity_type` — pick by
  spec shape), paged response with `page_token` support via the existing
  `PaginatedResponse` machinery if the shape fits, else manual token pass-through.
- Request: `activity_types`, `date`/`after`/`until`, `direction`,
  `page_size`, `page_token`.
- Tests: fixture round-trips for both activity families; query serialization;
  wiremock happy path + one error.

## Sprint 9 — Do-not-exercise + asset model refresh

- `src/trading/mod.rs`: `do_not_exercise_option(symbol_or_contract_id)` →
  `POST /v2/positions/{symbol_or_contract_id}/do-not-exercise` (204/empty
  response — reuse `RestClient` empty-body handling).
- `src/trading/models.rs` `Asset`: add `borrow_status`, `margin_requirement_long`,
  `margin_requirement_short` (spec deprecates `easy_to_borrow` /
  `maintenance_margin_requirement` — keep those but mark doc-deprecated);
  type `attributes: Vec<AssetAttribute>` as a real enum
  (`ptp_no_exception`, `ptp_with_exception`, `ipo`, `has_options`,
  `options_late_close`, `fractional_eh_enabled`, `overnight_tradable`,
  `overnight_halted`) with a catch-all `Other(String)` for forward compat.
- `GetAssetsRequest`: add `attributes` filter (comma-joined).
- Tests: fixture with new fields + unknown attribute → `Other`; wiremock
  do-not-exercise 204.

## Sprint 10 — v3 clock + calendar (multi-market)

- `src/trading/mod.rs`: `get_clock_v3(markets: &[Market])` → `GET /v3/clock`;
  `get_calendar_v3(market, &CalendarRequest)` → `GET /v3/calendar/{market}`.
- New `Market` enum (`src/trading/enums.rs`): NYSE, NASDAQ, OPRA, BOATS, OCEA,
  CRYPTO (verify exact wire values in spec) — serde snake/screaming per spec.
- Models: `ClockV3 { phase (enum: Closed/Pre/Core/Lunch/Post), phase_until,
  is_market_day, ... }` per market; calendar entries keyed per market.
- Keep `/v2/clock` + `/v2/calendar` untouched (legacy still served).
- Tests: fixture round-trips incl. overnight BOATS phase; query serialization.

## Sprint 11 — Stock auctions + single-symbol + meta endpoints

- `src/data/stock.rs`:
  - `auctions(&AuctionsRequest)` → `GET /v2/stocks/auctions`, and
    `auctions_for_symbol(symbol, &req)` → `GET /v2/stocks/{symbol}/auctions`
    (paginated; SIP/Algo Trader Plus only — doc-comment this).
  - Models: `Auction` (symbol, date, open/close price fields, imbalance,
    paired shares — verify against spec), `AuctionsResponse`.
  - Single-symbol variants: `bars_for_symbol`, `trades_for_symbol`,
    `quotes_for_symbol`, `latest_*_for_symbol`, `snapshot(symbol)` →
    `/v2/stocks/{symbol}/...`, reusing existing request structs.
  - Meta: `trade_conditions()` / `quote_conditions()` →
    `/v2/stocks/meta/conditions/{ticktype}`; `exchanges()` →
    `/v2/stocks/meta/exchanges`. Models: condition-code + exchange tables.
- `src/data/option.rs`: `conditions(ticktype)` →
  `/v1beta1/options/meta/conditions/{ticktype}` (companion to existing
  `exchanges()`).
- Tests: fixture round-trips; wiremock two-page auctions; path assertions for
  single-symbol variants.

## Sprint 12 — Short-sale locates

- `src/trading/locates.rs` (new, `/v1/...` paths on the trading host — second
  base path; `RestClient` already takes full paths, verify join semantics):
  - `get_locate_quotes(symbols: &[String])` → `GET /v1/locates/quotes` (≤100
    symbols)
  - `create_locate(&CreateLocateRequest)` → `POST /v1/locates` (round lots of
    100; optional `limit_price`, `all_or_none`)
  - `get_locates(&GetLocatesRequest)` → `GET /v1/locates` (status/symbol/
    start/end filters, paged)
  - `get_locate(locate_id)` → `GET /v1/locates/{locate_id}`
- Models: `LocateQuote` (symbol, available qty, fee rate), `Locate` (id,
  status active/expired/rejected, qty, fee...), enums `LocateStatus`. Verify
  all fields against the spec.
- Doc-comment: requires live account + HTB enablement; paper support unclear —
  test accordingly.
- Tests: fixtures, 400 on non-round-lot qty (client-side validation), wiremock
  happy paths.

## Sprint 13 — SSE event streams

New transport (ADR 0002 ruling: reqwest byte stream + small line parser; only
add `eventsource-stream` if the parser exceeds ~100 lines).

- `src/stream/events.rs` (new): shared `EventStream<T>` — connects with auth,
  parses `data:`/`id:`/`event:` frames, honors `Last-Event-Id` reconnect, maps
  JSON payloads to `T`.
- `ActivityEventsClient` → `GET /v2beta1/events/activities`
  (`since`/`until`/`since_id`/`until_id` params; replay support).
- `CorporateActionEventsClient` → `GET /v1beta1/events/corporate-actions`
  (insert/update/delete mutations for the 15 CA types, `since`).
- Reuse Sprint 8 activity models and Sprint 2's CA types for payloads.
- Tests: parser unit tests from canned SSE byte streams (frame splitting,
  multi-line data, Last-Event-Id bookkeeping); no live network.

## Sprint 14 — Low-priority grab-bag

- Watchlists by name: `GET/PUT/POST/DELETE /v2/watchlists:by_name` + add/remove
  asset by name (4–6 methods mirroring the by-ID ones).
- Forex: `ForexClient` (new `src/data/forex.rs`) → `GET /v1beta1/forex/rates`
  (historical, paged) + `GET /v1beta1/forex/latest/rates`. Model `ForexRate`.
- Logos: `logo(symbol)` → `GET /v1beta1/logos/{symbol}` on a data client
  (returns image bytes + content-type, not JSON — handle as `Vec<u8>`).
- Tests: fixtures + wiremock path/query assertions.

## Wave-2 closeout (after 7–14)

- Full verification suite + `cargo doc` warning-free.
- README feature list and `docs/diagrams/architecture.mmd` updated (new
  clients: locates, events/SSE, forex).
- Mark ADR 0002 accepted.
