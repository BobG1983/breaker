---
name: ChainBolt tether corrupts bolt speed — BoltSpeedAccurate fires
description: enforce_distance_constraints averages axial velocity between tethered entities, leaving bolt speed != base_speed.clamp(min,max); apply_velocity_formula is never called after constraint enforcement
type: project
---

## Confirmed game bug — tether_chain_bolt_stress

**Invariant:** `BoltSpeedAccurate` (x6, frames 3842–3846)

**Root cause:** `rantzsoft_physics2d/src/systems/enforce_distance_constraints.rs` modifies `Velocity2D` on both tethered entities by averaging their axial velocity components (lines 48–55). After redistribution, the chain bolt's speed is no longer equal to `(base_speed * mult).clamp(min_speed, max_speed)`. No `apply_velocity_formula` call follows the constraint enforcement, so the bolt remains at the incorrect speed until the next wall/cell/breaker collision.

**Why:** `apply_velocity_formula` is called in `bolt_wall_collision`, `bolt_cell_collision`, `bolt_breaker_collision`, `clamp_bolt_to_playfield`, `launch_bolt`, `reset_bolt`, etc. — but NOT after `enforce_distance_constraints`. The constraint solver is in `rantzsoft_physics2d` and has no knowledge of the bolt speed formula, which lives in `breaker-game/src/bolt/queries.rs`.

**Scenario:** `tether_chain_bolt_stress` — breaker=Aegis layout=Scatter, seed=8080, ChainBolt(tether_distance=120.0) on every PerfectBumped→Impacted(Cell) chain. Violations appear at frames 3842–3846 when tether goes taut and velocity is redistributed.

**Fix location:** Either:
1. `breaker-game/src/bolt/systems/` — add a post-constraint speed-clamp system that runs after `enforce_distance_constraints` in the same `FixedUpdate` tick, calling `apply_velocity_formula` on all bolts tagged `ActiveFilter`.
2. Or `rantzsoft_physics2d/src/systems/enforce_distance_constraints.rs` — expose a hook/callback for speed normalization (but this requires game knowledge which violates the crate boundary rule).

Option 1 is the correct approach given `rantzsoft_*` must have zero game knowledge.
