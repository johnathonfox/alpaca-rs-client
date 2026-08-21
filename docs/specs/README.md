# Pinned Alpaca OpenAPI specs

Snapshots of Alpaca's published OpenAPI specs, fetched 2026-08-20 from the
spec backend that serves docs.alpaca.markets:

- `trading-api.json` — Trading API **2.0.1** (OpenAPI 3.0.0, 45 paths)
- `market-data-api.json` — Market Data API **1.1** (OpenAPI 3.0.0, 47 paths)

## Why they are pinned

Per [ADR 0002](../adr/0002-beyond-parity-coverage.md) and
[ADR 0003](../adr/0003-deferred-scope-coverage.md), the crate's scope is
governed by diffing against Alpaca's OpenAPI specs rather than by parity with
alpaca-py. These snapshots are the baseline those diffs run against.

## How to audit for new surface

1. Fetch the current specs from the same source and drop them next to these
   (e.g. in `target/tmp/`).
2. `diff` or `git diff --no-index` the old and new files; new/changed paths
   and schemas are candidates for implementation.
3. Record the outcome in a new or updated ADR, then replace the pinned files
   here once the audit's changes land.
