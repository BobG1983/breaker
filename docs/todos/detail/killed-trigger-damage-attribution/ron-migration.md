# RON Migration: Death Trigger Unification

All RON files that reference `CellDestroyed`, `Death`, or `Died` triggers and need updating.

## Migration Rules

| Before | After |
|--------|-------|
| `trigger: CellDestroyed` | `trigger: Death(Cell)` |
| `trigger: Death` (no discriminant) | `trigger: Death(Any)` |
| `trigger: Died` | `trigger: Died` (unchanged — but now has killer in context) |

## Chip RON Files (10 files, 15 occurrences)

### `CellDestroyed` → `Death(Cell)`

| File | Line(s) | Count |
|------|---------|-------|
| `chips/standard/cascade.chip.ron` | 8, 18, 28 | 3 (one per tier) |
| `chips/standard/chain_reaction.chip.ron` | 8, 9 | 2 |
| `chips/standard/splinter.chip.ron` | 8, 19, 30 | 3 (one per tier) |
| `chips/standard/feedback_loop.chip.ron` | 10 | 1 |
| `chips/evolutions/entropy_engine.evolution.ron` | 6 | 1 |
| `chips/evolutions/voltchain.evolution.ron` | 6 | 1 |
| `chips/evolutions/gravity_well.evolution.ron` | 6 | 1 |
| `chips/evolutions/supernova.evolution.ron` | 8 | 1 |
| `chips/evolutions/split_decision.evolution.ron` | 6 | 1 |
| `chips/evolutions/chain_reaction.evolution.ron` | 6 | 1 |

### `Died` — unchanged (2 files)

| File | Line | Notes |
|------|------|-------|
| `chips/standard/death_lightning.chip.ron` | 10 | `Died` stays — mark-and-reward pattern |
| `chips/standard/powder_keg.chip.ron` | 10 | `Died` stays — mark-and-reward pattern |

## Example RON (1 file)

| File | Line | Change |
|------|------|--------|
| `examples/breaker.example.ron` | 61 | Update trigger list in comments: add `Killed(Cell)`, `Killed(Any)`, `Death(Cell)`, `Death(Any)`, remove `CellDestroyed` |

## Scenario RON Files (6 files, 7 occurrences)

### `CellDestroyed` → `Death(Cell)`

| File | Line(s) | Count |
|------|---------|-------|
| `scenarios/stress/entropy_engine_stress.scenario.ron` | 19 | 1 |
| `scenarios/stress/cascade_shockwave_stress.scenario.ron` | 19 (+ comment line 4) | 1 |
| `scenarios/stress/gravity_well_stress.scenario.ron` | 34 (+ comment line 1) | 1 |
| `scenarios/stress/gravity_well_chaos.scenario.ron` | 22 (+ comment line 1) | 1 |
| `scenarios/stress/supernova_chain_stress.scenario.ron` | 22 (+ comment line 2) | 1 |

### `Death` → `Death(Any)`

| File | Line | Count |
|------|------|-------|
| `scenarios/chaos/cell_death_speed_burst.scenario.ron` | 27 (+ comment line 5) | 1 |

## Rust Code Migration (not RON, but trigger references)

The `Trigger::CellDestroyed` enum variant and all Rust test files that construct it also need updating. These are covered by the main implementation, not this RON migration doc. Rough count: ~25 test files reference `Trigger::CellDestroyed`, ~10 reference `Trigger::Death` (no discriminant).

## Total

- **10 chip RON files** — mechanical find-replace `CellDestroyed` → `Death(Cell)`
- **6 scenario RON files** — 5× `CellDestroyed` → `Death(Cell)`, 1× `Death` → `Death(Any)`
- **1 example RON file** — update comments
- **2 chip RON files unchanged** — `Died` trigger stays as-is
