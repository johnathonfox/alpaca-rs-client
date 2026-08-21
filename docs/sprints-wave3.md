# Sprint Plan — Wave 3: deferred-scope coverage

Reference: [ADR 0003](adr/0003-deferred-scope-coverage.md) (scope decision;
supersedes the ADR 0002 deferral list). Same conventions and verification bar
as waves 1–2.

## Status

| Sprint | State |
| --- | --- |
| 15 — crypto wallets & funding (`/v2/wallets*`) | done |
| 16 — crypto perpetuals (`/v2/perpetuals/*` + perp market data) | done |
| 17 — tokenization (`/v2/tokenization/*`) | done |
| 18 — fixed income (data host + broker asset lists) | done |

All four landed together (2026-08-21); they are listed as separate sprints
only to mirror the ADR's structure.

## Sprint 15 — Crypto wallets & funding *(done)*

- `src/trading/wallets.rs`: `get_wallets` (lenient single-object-vs-array
  decoding), `get_wallet_transfers` / `get_wallet_transfer`,
  `create_wallet_transfer` (doc-deprecated; sunset 2026-10-09), whitelist
  list/create/delete, `get_transfer_fee_estimate`.
- Models in `src/trading/models.rs` (`CryptoWallet`, `CryptoTransfer`,
  `WhitelistedAddress`, `TransferFeeEstimate`), enums in
  `src/trading/enums.rs` (`CryptoChain`, transfer direction/status,
  `WhitelistStatus` — all with catch-alls).
- Availability gate (Alpaca enablement required) doc-commented; paper
  supported.

## Sprint 16 — Crypto perpetuals *(done, beta)*

- `src/trading/perpetuals.rs`: `/v2/perpetuals/wallets*` mirrors of the
  sprint-15 endpoints (reusing its models), `get_perp_leverage` /
  `set_perp_leverage` (query-param POST via `RestClient::post_query`),
  `get_perp_account_vitals` (lenient number-or-string decimals).
- `src/data/crypto_perp.rs`: `CryptoPerpDataClient` — latest
  bars/trades/quotes/orderbooks (existing crypto models) + funding `pricing`
  under `/v1beta1/crypto-perps/global/latest/*` (`loc` hardcoded to the only
  published value).
- `AssetClass::CryptoPerp` added; perp orders flow through `POST /v2/orders`.

## Sprint 17 — Tokenization *(done)*

- `src/trading/tokenization.rs`: `mint_tokenized_asset` (optional
  `Idempotency-Key` header via `RestClient::post_with_idempotency_key`),
  `get_tokenization_requests` (filters, no pagination),
  `get_tokenization_request`,
  `get_tokenization_request_by_client_request_id` (`:by_client_request_id`
  path convention).
- `TokenizationIssuer` / `TokenizationNetwork` enums carry catch-alls — the
  published lists grew between spec versions.
- AP-only enablement doc-commented; paper supported.

## Sprint 18 — Fixed income *(done)*

- `src/data/fixed_income.rs`: `FixedIncomeDataClient` — latest prices/quotes
  by ISIN (`/v1beta1/fixed_income/latest/*`, f64 numerics, 1000/100 ISIN caps
  validated client-side).
- `src/broker.rs`: `FixedIncomeAssetsClient` — `us_treasuries` /
  `us_corporates` lists on the broker host with the new
  `Credentials::BasicAuth` / `Credentials::from_broker_env()`
  (`APCA_BROKER_API_KEY` / `APCA_BROKER_API_SECRET`). Bond orders remain out
  of scope (broker-only).

## Wave-3 closeout

- `cargo build` / `test` / `clippy --all-targets -- -D warnings` / `fmt` all
  clean; `cargo doc --no-deps` warning-free.
- README feature list and `docs/diagrams/architecture.mmd` updated.
- Follow-ups after landing: `tests/live_smoke.rs` sweeps extended to the new
  read-only endpoints (run against a paper account — all gated surfaces
  returned the expected 403/404s), `tests/live_orders.rs` added (opt-in paper
  order submit/cancel round-trip), and `examples/news.rs` added (symbol-arg
  news fetch used to verify `NewsClient` against live data).
