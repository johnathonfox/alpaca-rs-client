# ADR 0003: Deferred-scope coverage — crypto wallets/funding, tokenization, fixed income, crypto perps

- Status: accepted
- Date: 2026-08-21
- Supersedes: the deferral list of [ADR 0002](0002-beyond-parity-coverage.md)

## Context / Problem

ADR 0002 deferred four documented Alpaca surfaces as "niche or pre-GA":
crypto wallets/funding, tokenization, fixed income, and crypto perpetuals.
The user has now asked for all four to be implemented. A fresh audit against
the live OpenAPI specs (Trading 2.0.1, Market Data 1.1, docs v1.4) shows the
surfaces have since reached GA or beta with published schemas:

- **Crypto wallets & funding** (trading host, GA but enablement-gated):
  `GET /v2/wallets`, `GET|POST /v2/wallets/transfers`,
  `GET /v2/wallets/transfers/{id}`, `GET|POST /v2/wallets/whitelists`,
  `DELETE /v2/wallets/whitelists/{id}`, `GET /v2/wallets/fees/estimate`.
  `POST /v2/wallets/transfers` is deprecated (sunset 2026-10-09) — withdrawals
  are moving to the web app.
- **Tokenization** (trading host, GA for enabled Authorized Participants,
  paper supported): `POST /v2/tokenization/mint` (optional `Idempotency-Key`
  header), `GET /v2/tokenization/requests`, `GET
  /v2/tokenization/requests/{id}`, `GET
  /v2/tokenization/requests:by_client_request_id`.
- **Fixed income**: split personality. *Market data* is public and beta on the
  data host: `GET /v1beta1/fixed_income/latest/prices` and
  `.../latest/quotes` (ISIN-keyed, f64 numerics). *Asset lists*
  (`GET /v1/assets/fixed_income/us_treasuries`, `.../us_corporates`) and bond
  *orders* are Broker-API only (Basic Auth, `broker-api[.sandbox].alpaca.markets`).
- **Crypto perpetuals** (beta): trading-side `GET|POST
  /v2/perpetuals/wallets[...]` (same shapes as spot crypto funding) plus
  `GET|POST /v2/perpetuals/leverage` (set via query params) and
  `GET /v2/perpetuals/account_vitals`; market-data latest bars/trades/quotes/
  orderbooks/pricing under `/v1beta1/crypto-perps/{loc}/latest/*` (`loc` is
  always `global` today). Perp orders go through the standard `POST
  /v2/orders` with `asset_class: crypto_perp`.

## Options Considered

1. **Trading/data hosts only, skip broker API** — smallest surface, but
   leaves the fixed-income asset lists (the only way to discover tradable
   bonds) unimplemented, making "fixed income" support partial.
2. **Everything, including a full Broker API client** — the broker API is a
   separate product (accounts, KYC, journals…); a full client would dwarf the
   rest of the crate and serve a different audience.
3. **Curated: all trading/data-host endpoints + a minimal broker module for
   the fixed-income asset lists only** — covers everything an algo trader can
   act on; broker module is explicitly scoped to bond discovery.

## Decision

Option 3.

- `src/trading/wallets.rs`: crypto funding wallets, transfers, whitelists,
  fee estimates. `GET /v2/wallets` returns a single object when `asset` is
  given and an array otherwise — deserialize leniently into `Vec`. The
  deprecated withdrawal (`POST /v2/wallets/transfers`) is implemented but
  doc-marked deprecated with its sunset date.
- `src/trading/perpetuals.rs`: the `/v2/perpetuals/*` wallet/transfer/
  whitelist/fee endpoints reusing the wallet models, plus leverage get/set
  (query-param POST) and account vitals. Beta-gated; documented as such.
- `src/trading/tokenization.rs`: mint + request lookups. Mint accepts an
  optional idempotency key (new `RestClient` support) and `client_request_id`
  (documented in the guide, missing from the schema — include it).
- `src/data/fixed_income.rs`: `FixedIncomeDataClient` — latest prices/quotes
  (f64 numerics, ISIN-keyed maps, per data-side convention).
- `src/data/crypto_perp.rs`: `CryptoPerpDataClient` — latest bars/trades/
  quotes/orderbooks (reusing the existing crypto models where shapes match)
  plus the perp-specific pricing model (`fr`, `ft`, `oi`, `ip`, `mp`).
  `loc` is hardcoded to `global` (the only published value) with a doc note.
- `src/broker.rs`: `FixedIncomeAssetsClient` — `us_treasuries` /
  `us_corporates` lists on the broker host with new
  `Credentials::BasicAuth` (env: `APCA_BROKER_API_KEY`,
  `APCA_BROKER_API_SECRET`). No other broker endpoints. Bond orders stay out
  of scope (broker-only; no public trading-API path).
- `AssetClass` gains `CryptoPerp` (perp orders flow through `/v2/orders` and
  `/v2/assets?asset_class=crypto_perp`).
- Issuer/network/chain enums get `#[serde(other)]`-tolerant variants or
  catch-alls — the published lists have already grown between spec versions.

Rulings carried over from ADR 0002 conventions: string `Decimal` for
trading-host numbers, `f64` on the data host, no pagination on any of these
endpoints (none support it), wiremock tests only, doc-marked availability
gates (crypto wallets need Alpaca enablement; tokenization needs AP status;
perps are beta and non-US).

## Consequences

### Positive

- The crate now covers every documented Alpaca endpoint reachable with
  trading/data credentials, plus bond discovery via broker credentials.
- Shared wallet models keep the spot-crypto and perps funding surfaces DRY.

### Negative

- Several endpoints are enablement-gated or beta; most users will only ever
  see 403s there. Doc comments carry that burden.
- `Credentials` grows a third variant (Basic Auth) for a single two-endpoint
  client — accepted as the price of completing fixed income.
- Some schema quirks are coded defensively (optional fields, lenient
  single-vs-array decoding) because spec and live behavior disagree; these
  are marked in doc comments where known.
