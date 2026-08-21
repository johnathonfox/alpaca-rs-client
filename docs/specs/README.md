# Pinned Alpaca OpenAPI specs

Snapshots of Alpaca's published OpenAPI specs from two sources, which are
**not identical** (verified 2026-08-21: the docs-backend copy has the
perpetuals endpoints, the JS-repo copy has `/v1/locates`; neither is a strict
superset of the other):

- `trading-api.json`, `market-data-api.json` — the spec backend behind
  docs.alpaca.markets (Trading **2.0.1**, Market Data **1.1**), fetched
  2026-08-20. This is the baseline the wave-3 audit (ADR 0003) ran against.
  There is no stable public URL for re-fetching this copy; refresh it
  manually during audits.
- `js-repo-trading-api.json`, `js-repo-market-data-api.json` — the mirror in
  the official [`alpaca-trade-api-js`](https://github.com/alpacahq/alpaca-trade-api-js)
  repo (`tooling/specs/`), fetched 2026-08-21. This source has a stable raw
  URL, so the **spec-drift GitHub workflow** watches it: it re-downloads
  these two files weekly and opens an issue when they drift.

## Why they are pinned

Per [ADR 0002](../adr/0002-beyond-parity-coverage.md) and
[ADR 0003](../adr/0003-deferred-scope-coverage.md), the crate's scope is
governed by diffing against Alpaca's OpenAPI specs rather than by parity with
alpaca-py. These snapshots are the baselines those diffs run against.

## How to audit for new surface

1. Watch for issues opened by the spec-drift workflow, or fetch current
   specs (JS-repo URLs above; the docs-backend spec via the docs site's spec
   backend) and diff against the pinned files.
2. New/changed paths and schemas are candidates for implementation — check
   both sources, since each covers endpoints the other lacks.
3. Record the outcome in a new or updated ADR, then replace the pinned files
   here once the audit's changes land.
