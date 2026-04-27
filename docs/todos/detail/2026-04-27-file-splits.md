# File Splits — 2026-04-27 (Wave 1 + Wave 2 post-merge sweep)

Branch context: feature work on `rantzsoft_dmg` (Wave 1) added `aggregate_one_shots` peek accessor + `preview_damage` helper, plus six new behavior-test groups (Behaviors 80–85). Wave 2 wired `preview_damage` into `bolt_cell_collision`. This sweep checks whether either wave pushed any source file over the 400-line threshold.

## File Length Review

### Files Over Threshold

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `rantzsoft_dmg/src/components/damage_boost_stack.rs` | 560 | 82 | 478 | 31 | A: test extraction | MEDIUM |

Tests are 85.4% of the file — a textbook Strategy A candidate.

### Priority Guide
- **HIGH**: 1000+ lines, or 800+ test lines
- **MEDIUM**: 501–999 lines (this case)
- **LOW**: 400–500 lines

### Pre-Wave 1 size

Wave 1 added Behaviors 80–85 (six groups, 14 new test functions, ~154 lines of tests at lines 405–559) and the `aggregate_one_shots` method (~9 lines, lines 67–74). Pre-Wave 1 line count: ~397 — right at the threshold but technically under it. **Wave 1 pushed this file over 400 lines.**

Wave 2 did NOT touch this file at all — Wave 2 lives in `breaker-game/`.

---

### Refactor spec hint — `damage_boost_stack.rs`

- Source file: `rantzsoft_dmg/src/components/damage_boost_stack.rs`
- Total lines: 560 (prod: 82, tests: 478)
- Strategy: **A — test extraction**, single `tests.rs` (478 lines is well under the 800-line sub-split threshold)
- Target structure:

```
rantzsoft_dmg/src/components/
  damage_boost_stack/
    mod.rs            // wiring only — module declarations + re-exports
    component.rs      // production: imports, doc comment, the negative-Clone contract,
                      //              `DamageBoostStack` struct + impl block
    tests.rs          // all 31 tests + the `assert_f32_eq` helper
```

- `mod.rs` contents (and ONLY this):

```rust
//! `DamageBoostStack` component — a `Vec`-backed collection of dealer-side
//! damage multipliers, append-semantic (no de-duplication of duplicate
//! sources). Callers pair a persistent multiplier with a `SourceId` so they
//! can later retract only their own contribution; one-shot multipliers are
//! drained on the next aggregate call.

mod component;

#[cfg(test)]
mod tests;

pub use component::DamageBoostStack;
```

- `component.rs` contents:
  - The `use bevy::prelude::*;` import
  - The `use crate::SourceId;` import
  - The Behavior 67a negative-Clone contract comment block (lines 11–16 of the original)
  - The `DamageBoostStack` struct (lines 28–32)
  - The `impl DamageBoostStack { ... }` block (lines 34–81)
  - NO test code

- `tests.rs` contents:
  - `use super::component::DamageBoostStack;`
  - `use crate::SourceId;`
  - The `assert_f32_eq` helper (lines 91–103 of the original)
  - All 31 test functions, unchanged
  - NO `#[cfg(test)]` attribute on the module body — the gate on `mod tests;` in `mod.rs` is sufficient

- Test groups in `tests.rs` (single file — not sub-split):
  - Behavior 56: `add` appends a single entry (2 tests)
  - Behavior 57: same source N times produces N entries (2 tests)
  - Behavior 58: mixed sources/multipliers multiply all entries (2 tests)
  - Behavior 59: `remove_by_source` removes ALL matching entries (2 tests)
  - Behavior 60: `remove_by_source` for absent source is a no-op (2 tests)
  - Behavior 61: `add_one_shot` appends to `one_shots` only (2 tests)
  - Behavior 62: `aggregate_persistent` returns 1.0 when empty (2 tests)
  - Behavior 63: `aggregate_and_consume_one_shots` returns product + clears (3 tests)
  - Behavior 64: `is_empty` is true iff BOTH lanes are empty (4 tests)
  - Behavior 65: `Default::default` produces empty stack (2 tests)
  - Behavior 66: `DamageBoostStack` is a Bevy `Component` (2 tests)
  - Behavior 67: `DamageBoostStack` derives `Debug` (2 tests)
  - Behavior 80: `aggregate_one_shots` returns 1.0 when empty (2 tests) — Wave 1
  - Behavior 81: single one-shot returns the value (2 tests) — Wave 1
  - Behavior 82: multiple one-shots return the product (2 tests) — Wave 1
  - Behavior 83: peek does not consume (2 tests) — Wave 1
  - Behavior 84: `aggregate_one_shots` ignores persistent lane (2 tests) — Wave 1
  - Behavior 85: symmetric counterpart does not pollute (3 tests) — Wave 1

- Imports needed:
  - `component.rs`: `use bevy::prelude::*;` and `use crate::SourceId;` (unchanged from original)
  - `tests.rs`: `use super::component::DamageBoostStack;` and `use crate::SourceId;`

- Re-exports needed:
  - `mod.rs` must re-export `DamageBoostStack` so `components/mod.rs` line 11 `pub use damage_boost_stack::DamageBoostStack;` continues to resolve. (Single re-export: `pub use component::DamageBoostStack;`.)

- Parent-module impact:
  - `rantzsoft_dmg/src/components/mod.rs` declares `mod damage_boost_stack;` and `pub use damage_boost_stack::DamageBoostStack;` — both continue to work unchanged because Rust resolves `damage_boost_stack` to either `damage_boost_stack.rs` OR `damage_boost_stack/mod.rs`.

- External impact:
  - `rantzsoft_dmg/src/lib.rs` re-exports `DamageBoostStack` via `pub use components::{ DamageBoostStack, ... };` — unchanged because the public path through `components` is preserved.
  - No other callers reference items inside `damage_boost_stack` directly — they all reach the type via the crate root.

- Negative-Clone contract preservation:
  - The commented `let _ = DamageBoostStack::default().clone();` line is a documented negative contract (must NOT compile if uncommented). Keep this comment block in `component.rs` directly above the struct definition. Behavior is unchanged because the struct still does not derive `Clone` after the move.

- Delegate: writer-code can execute this refactor directly via `/quickfix` — single-file split with no API change, no behavior change, no logic change.

---

## Notes / observations

- `rantzsoft_dmg/src/preview.rs` (Wave 1, ~256 lines) is **clean** — well under threshold.
- `breaker-game/src/cells/systems/bolt_cell_collision/damage_multiplier.rs` (Wave 2, ~275 lines) is **clean** — well under threshold.
- The companion file `rantzsoft_dmg/src/components/vulnerable_stack.rs` was NOT inspected in this sweep (Wave 1 did not add behaviors there). Future work in todo #2 ("source-filtered damage boosts AND vulnerability") will likely push it over too — flag for the writer of that todo to plan the analogous split as part of that work, not now.
- No mod.rs violations introduced by Wave 1 or Wave 2.
- Pattern: small `Component` + impl files with thorough behavior-spec test coverage routinely cross 400 lines as the test catalog grows. Strategy A (single `tests.rs`) is the cheapest fix and matches the established `rantzsoft_*` module-directory pattern.
