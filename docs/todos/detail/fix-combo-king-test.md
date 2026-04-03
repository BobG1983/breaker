# Fix detect_combo_king stale migration test

## Summary

`state/run/node/highlights/systems/detect_combo_king.rs` has two test quality issues found during Wave 1 full verification:

### 1. Stale migration test (lines 145-198)

The test block labelled "C7 Wave 2a" creates a duplicate `TestCellDestroyedAt` struct (separate from the existing `TestCellDestroyed` at line 73) and builds its own minimal app instead of using `test_app()`. This means it omits the `HighlightTriggered` message registration and could silently miss regressions. The test is labelled as a migration check (`CellDestroyed -> CellDestroyedAt migration`) but is now permanent behavior.

**Fix**: Merge into `cell_destroyed_increments_cells_since_last_breaker_hit` (same behavior, same assertion) or rewrite against `test_app()`.

### 2. Incomplete dedup assertion (lines 331-364)

The `dedup_only_one_combo_king_in_run_stats` test covers `HighlightTriggered` still being emitted on the second match but never asserts the count of `HighlightTriggered` messages. The code at lines 38-54 shows the emit always fires but the push is deduped. Add an assertion that `CapturedHighlightTriggered` contains exactly one `ComboKing` entry to make expected behavior explicit.

## Status
`[ready]`
