---
name: Effect trigger bridge systems performance analysis
description: Analysis of all trigger bridge systems, desugar_until, tick_time_expires, evaluate_staged_effects allocation patterns — reviewed on develop branch
type: project
---

## Entity Scale for BoundEffects/StagedEffects

Entities that carry BoundEffects + StagedEffects:
- Cells: ~50 (dispatched at OnEnter(Playing) via dispatch_cell_effects)
- Breaker: 1 (dispatched via dispatch_breaker_effects)
- Bolt: 1-8 (inserted on chip dispatch; spawned bolts inherit with spawn_bolts effect)
- Walls: 0 currently (dispatch_wall_effects is a no-op)
Total: ~52-59 entities

## desugar_until: run_if Guard FIXED (feature/full-verification-fixes)

desugar_until now has run_if(in_state(PlayingState::Active)) in triggers/mod.rs line 85.
Confirmed fixed.

## tick_time_expires: run_if Guard FIXED (feature/full-verification-fixes)

tick_time_expires now has run_if(in_state(PlayingState::Active)) in triggers/timer.rs line 62.
Confirmed fixed.

## evaluate_staged_effects: Vec::new() per entity per bridge call

evaluate_staged_effects allocates let mut additions = Vec::new() on every call.
With ~20 bridge systems all calling it: 20 × 60 = 1200 Vec::new() calls per tick.
All are zero-heap until a trigger matches and additions.push() fires.
In practice most ticks have 0-1 matching triggers. ZERO heap allocations per tick.
The cost is the function call overhead and stack allocation of the header — negligible.

## impact/system.rs: Global Entity Scan on Collision Messages

bridge_impact_bolt_cell (and 5 similar systems) iterate over ALL ~60 entities with
BoundEffects on each collision message arrival. Each BoltImpactCell message causes
120 entity iterations (60 for Impact(Cell) + 60 for Impact(Bolt)).

At current scale (1 bolt hitting cells): 1-8 messages/tick × 120 iterations = up to
960 entity evaluations per tick. All evaluations are O(N_bound_effects) — typically
1-3 entries per entity.

These systems have run_if(in_state(PlayingState::Active)) guards. Clean.

At current scale this is acceptable. Will become Moderate at 200+ entities with
BoundEffects or 10+ simultaneous bolts.

## impacted/system.rs: Targeted by Entity — Efficient

bridge_impacted_* systems use query.get_mut(msg.entity) — targeted lookup, O(1).
No full entity scan. Correct and efficient pattern.

## Scheduling Guard Summary (as of feature/full-verification-fixes)

Systems WITH run_if(in_state(PlayingState::Active)):
- All bridge systems in EffectSystems::Bridge set (impact, bumped, bump, etc.)
- tick_gravity_well + apply_gravity_pull
- despawn_second_wind_on_contact
- All EffectSystems::Recalculate systems
- desugar_until (fixed in feature/full-verification-fixes)
- tick_time_expires (fixed in feature/full-verification-fixes)
- process_piercing_beam (fixed in feature/full-verification-fixes)

Systems WITHOUT run_if guards (FixedUpdate, always-run):
- bridge_no_bump (placeholder, const fn no-op — zero cost, but inconsistent with peers)

Note: bridge_no_bump is a Wave 8 placeholder — no query iteration, no allocations. Minor.

## Intentional Patterns

- evaluate_staged_effects Vec::new() per call: zero-cost in idle path. Acceptable.
- bridge_impact_* global scan: bounded by entity count. Acceptable at current scale.
- desugar_until 4 × Vec::new() per entity: stack-only, zero heap. Acceptable.
- All phase bridge systems correctly check message queues before entity iteration.
