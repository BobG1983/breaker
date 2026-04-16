---
name: Magnetic Wave 5 apply_magnetic_fields patterns
description: apply_magnetic_fields system — O(bolts×magnets) nested loop, sqrt usage, PhantomPhase optional — all confirmed acceptable at current scale
type: project
---

File: `breaker-game/src/cells/behaviors/magnetic/systems/apply_magnetic_fields.rs`

## Pattern inventory

**MagneticQuery archetype**: `MagneticCell + Without<Dead>` filter — clean, no fragmentation. `MagneticCell` is a permanent marker, never added/removed at runtime. `MagneticField` is also permanent. Zero archetype churn.

**O(bolts × magnets) nested loop**: Outer loop over bolts (1–few), inner loop over magnet cells. Benign at <20 bolts and <50 magnetic cells. No allocation inside either loop.

**`bolt_pos.0.distance()` before `inverse_square_attraction`**: Uses sqrt (via `Vec2::distance`). This is the cheap reject/cull guard. At the entity counts present, the cost is negligible. A `length_squared()` comparison could avoid the sqrt on the cull path but is not warranted now.

**`inverse_square_attraction` internals**: Calls `delta.normalize_or_zero()` (sqrt) + `delta.length_squared()` (no sqrt). The normalize_or_zero call is unavoidable — direction is needed. Two Vec2 operations total per magnet. No allocation.

**`total_force.length()` cap check**: One sqrt per bolt per frame to check against `2 * base_speed`. At 1–few bolts this is negligible. Could be `length_squared() > max_accel * max_accel` to eliminate the sqrt, but the absolute cost is immaterial at this scale.

**`Option<&PhantomPhase>` in magnet query**: This is the correct pattern — `MagneticCell` and `PhantomCell` are separate behaviors that can stack; using `Option<>` instead of a `Without<PhantomCell>` filter is correct because a cell that is both magnetic AND phantom in Solid/Telegraph phase should still attract. The option check (`is_some_and(|p| *p == PhantomPhase::Ghost)`) is a branch-per-inner-iteration, not an allocation. Acceptable.

**Schedule**: `FixedUpdate`, gated by `run_if(in_state(NodeState::Playing))`. Correct placement for physics-affecting logic.

**No allocations in hot path**: `Vec2::ZERO` is a stack constant. `total_force` accumulation is pure stack arithmetic. No `Vec`, no `String`, no `collect()`.

## Phase 3 watch

If magnetic cells become numerous (>20 on a single grid) AND bolts multiply (ExtraBolt upgrades pushing to 10+), the nested loop is O(n*m) = O(200). Still trivial. No action needed unless both axes grow simultaneously to 50+.

**Why:** Reviewed as part of Wave 5 magnetic cell modifier performance audit.
