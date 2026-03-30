---
name: Scenario coverage branch checker analysis
description: Performance analysis of the three new invariant checkers added in feature/scenario-coverage (AabbMatchesEntityDimensions, GravityWellCountReasonable, SizeBoostInRange)
type: project
---

## Branch: feature/scenario-coverage

### New Invariant Checkers (all in checkers_c, under playing_gate)

All three new checkers are correct. No performance issues found.

#### check_gravity_well_count_reasonable
- Single `With<GravityWellMarker>` query — one archetype, O(count), zero allocations on the clean path.
- Reads `max_gravity_well_count` from `ScenarioConfig` resource (not a heap alloc).
- Structurally identical to check_pulse_ring_accumulation and check_bolt_count_reasonable — correct pattern.
- Entity scale: 0-10 GravityWellMarker entities in normal play.

#### check_size_boost_in_range
- Query: `(Entity, &ActiveSizeBoosts, &EffectiveSizeMultiplier)` — both required (no Option<>).
- `active.multiplier()` iterates the inner Vec<f32> from the borrowed component — no allocation, no clone.
- Violation path does `format!` per offending entity — violations are abnormal so this does not affect steady-state.
- Entity scale: 1 bolt + 1 breaker with ActiveSizeBoosts when size_boost chip is active.

#### check_aabb_matches_entity_dimensions
- Two queries: `(Entity, &Aabb2D, &BoltRadius)` with `With<ScenarioTagBolt>` and a BreakerAabbQuery with `Option<&EntityScale>`.
- The `Option<EntityScale>` is intentionally correct (scale defaults to 1.0 when absent). At 1 breaker entity, Option<> overhead is purely academic.
- 1 bolt + 1 breaker in every scenario — O(1) in practice.

### Two-Query Pattern in check_chain_arc_count_reasonable (pre-existing, Minor)
- Chains and arcs are counted via two separate queries and summed.
- At 0-1 chain + 0-1 arc (confirmed Phase 5 scale), this is negligible.
- If chain lightning ever allows many concurrent chains, a shared marker component and single query would be worth considering.

### Frame Mutation Helpers (one-shot, not per-tick)
- apply_inject_mismatched_bolt_aabb: runs once at the mutation frame, not every tick.
- apply_spawn_extra_gravity_wells: runs once, spawns N entities with GravityWellMarker + GravityWellConfig + CleanupOnNodeExit.
- apply_inject_wrong_size_multiplier: runs once, spawns one entity with ActiveSizeBoosts + EffectiveSizeMultiplier.
- apply_spawn_extra_chain_arcs: allocates HashSet::new() per chain entity at mutation frame — correct, not per-tick.

### Scheduling Confirmation
- All new checkers placed in checkers_c and run inside playing_gate chain.
- playing_gate = `stats.is_some_and(|s| s.entered_playing)` — no wasted work during Loading.
- checkers_c runs .chain() after checkers_a and checkers_b — serialized within FixedUpdate, acceptable since all share ResMut<ViolationLog>.
