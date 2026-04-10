# Wave 8: RON Asset Migration

## Goal
Replace all RON asset files with the new effect syntax before migrating domain systems.

## Steps

1. Replace all `.chip.ron`, `.breaker.ron`, `.evolution.ron` files in `assets/` with the migrated versions
2. Add a test that loads every RON asset file and verifies deserialization succeeds

## Verification
`cargo dtest` — RON deserialization smoke test passes.

## Docs to read
- `effect-refactor/migration/finalized-assets/` — all migrated RON files
- `effect-refactor/ron-deserializing/` — deserialization strategy
