---
name: Phase 1B/1C lint patterns
description: Clippy state after Phase 1C; 28 doc_markdown errors in cleanup_cell.rs (new); 92 warnings in game crate; scenario runner clean
type: project
---

## Phase 1C Lint State

### 28 Errors — all in one file

All 28 errors are `clippy::doc_markdown` ("item in documentation is missing backticks") in:
`breaker-game/src/cells/systems/cleanup_cell.rs`

Lines: 88, 90, 91, 92, 93, 94, 129, 131, 132, 133, 134, 135, 170, 172, 173, 197, 199, 200, 201, 202, 256, 258, 259, 260 (and column variants)

These are in the `/// Behavior N:` doc comments on test functions. The items that need backticks are type/message/function names: `cleanup_cell`, `CellDestroyedAt`, `RequestCellDestroyed`, `was_required_to_clear`.

### 92 Warnings — inherited from Phase 1B effect stubs (unchanged)

**Pattern A — `missing_const_for_fn` (~50 occurrences)**
Effect stub `fire`, `reverse`, `register` functions in `effect/effects/` and trigger `bridge_*` functions in `effect/triggers/`. Fix: add `const`.

**Pattern B — `needless_pass_by_ref_mut` (9 occurrences)**
Placeholder functions take `&mut World` but never mutate. Files: `chain_lightning.rs`, `explode.rs`, `piercing_beam.rs`, `pulse.rs`, `random_effect.rs`, `shockwave.rs`. Fix: `&mut World` → `&World`.

**Pattern C — `struct_field_names` (6 occurrences)**
`effect/core/types.rs` — use `Self` instead of `EffectNode` inside `impl EffectNode`.

**Pattern D — dead code (unused structs/functions/types/fields)**
Same cluster as Phase 1B: `BoltSizeBoost`, `BumpForceBoost`, `ChainHit`, `InitBreakerQuery`, functions `init_breaker`, `apply_breaker_config_overrides`, etc. Will clear when Wave 8 wires bridges.

**Pattern E — `unused_imports` (4 locations)**
- `bolt_wall_collision.rs:9` — `bevy::prelude::*`
- `breaker/systems/mod.rs:25` — `apply_breaker_config_overrides`, `init_breaker`
- `chips/systems/dispatch_chip_effects/system.rs:11` — `BoundEffects`, `EffectCommandsExt`, `RootEffect`, `Target`

**Pattern F — `float_arithmetic` / `suboptimal_flops` (3 occurrences)**
- `effect/effects/gravity_well.rs:100,101` — `x += a * b` → `x = a.mul_add(b, x)`
- `effect/effects/shockwave.rs:53` — mul_add fix

**Pattern G — `redundant_clone` (1 occurrence)**
- `effect/triggers/timer.rs:143` — remove `.clone()`

**Pattern H — `unused_variable` (1 occurrence)**
- `effect/effects/random_effect.rs:9` — rename `world` to `_world`

### Scenario runner (dsclippy)
All scenario runner warnings originate from `breaker-game` dependency. The runner binary itself is clean.

### rantzsoft_* crates
`spatial2d`, `physics2d` pass with zero warnings or errors.
