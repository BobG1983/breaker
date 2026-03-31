---
name: Wave 1 stat-boost lazy init and FIFO gravity-well/phantom effects
description: Stat-boost lazy init pattern (speed/damage/size/bump_force), GravityWell FIFO, SpawnPhantom FIFO — all episodic; confirmed clean at current scale; minor dead-code in stat-boost fire()
type: project
---

## Context

Reviewed on feature/scenario-coverage branch. Wave 1 changes add:
1. Lazy init (insert Active* + Effective* on first fire) to speed_boost, damage_boost, size_boost, bump_force
2. Per-owner FIFO cap enforcement to GravityWell (using GravityWellSpawnCounter resource)
3. Same FIFO pattern to SpawnPhantom (using PhantomSpawnCounter resource)

## Stat-Boost Lazy Init (speed_boost.rs, damage_boost.rs, size_boost.rs, bump_force.rs)

Pattern in all four files:
  if world.get::<Active*>(entity).is_none() {
      world.entity_mut(entity).insert((Active*::default(), Effective*::default()));
  }
  if world.get::<Effective*>(entity).is_none() {           // ← dead branch
      world.entity_mut(entity).insert(Effective*::default());
  }
  if let Some(mut active) = world.get_mut::<Active*>(entity) { active.0.push(...); }

Performance verdict: CLEAN. fire() is episodic (chip activation, not per-frame).
The second `is_none()` check is a defensive guard for entities that have Active* but not
Effective* (half-initialized via external insertion). This IS reachable — see reviewer-quality
wave1-lazy-init-fifo-patterns.md which correctly documents this as intentional.
No performance impact (single world.get on one entity during episodic fire).

quick_stop.rs does NOT have the lazy init pattern — it assumes components are pre-inserted.
This is intentional (different registration pattern). Not a bug or gap.

## recalculate_* systems

REMOVED as of the cache-removal refactor (same branch). All 6 recalculate_* systems and all
6 Effective* components are gone. Consumers call .multiplier() inline. See
phase3-stat-effects.md for the updated call-site inventory and net-cost analysis.

## GravityWell FIFO (gravity_well/effect.rs)

fire() pattern:
  1. Read counter from GravityWellSpawnCounter resource (get_resource_or_insert_with)
  2. world.query all GravityWellConfig+GravityWellSpawnOrder, filter by owner, collect Vec
  3. sort Vec ascending (oldest-first), build despawn list
  4. despawn excess wells
  5. spawn new well with GravityWellSpawnOrder(counter_value)
  6. increment counter in resource

Performance verdict: CLEAN. fire() is called per chip-activation event, not per FixedUpdate.
Number of active wells is bounded by max * active-chip-count (practically single digits).
Full world query + sort on every fire() is fine at this scale.

HashMap resource (GravityWellSpawnCounter): lazily initialized, only mutated in fire(), never
accessed in the hot path (apply_gravity_pull only reads Position2D + GravityWellConfig).

apply_gravity_pull: clean hot path. N_wells * N_bolts inner loop with no allocations.
At current scale: 0-3 wells * 1-4 bolts = 0-12 iterations per FixedUpdate tick.

## SpawnPhantom FIFO (spawn_phantom/effect.rs)

Same pattern as GravityWell FIFO. fire() scans all phantom entities, filters by owner, sorts,
despawns excess, then spawns new phantom.

owned.remove(0) in enforcement while-loop: O(N) shift per removal. max_active is small (2-5).
At max_active=5, each fire() does at most a few element shifts — unmeasurable.
If max_active ever exceeds ~10, replace with drain(..excess_count) + separate despawn loop.

## Archetype Impact

New components from Wave 1:
- GravityWellSpawnOrder on GravityWellMarker entity: same archetype as before + one field.
  Novel archetype for gravity wells (GravityWellMarker + GravityWellConfig + GravityWellSpawnOrder
  + Position2D + CleanupOnNodeExit). 0-max_active entities alive at once.
- PhantomSpawnOrder on PhantomBoltMarker entity: similar — adds one component to phantom bolt
  archetype. 0-max_active phantom bolt entities alive at once.
- Active*/Effective* stat components on bolt entity: added lazily on first fire. Causes one
  archetype change per chip activation. With 1-4 bolts and chip activations being episodic,
  not a fragmentation concern.

## Intentional Patterns (confirmed clean)

- Lazy init in stat-boost fire(): correct — avoids requiring components pre-inserted on every
  possible target entity. The dead second is_none() check is a minor code smell, not a bug.
- FIFO cap via sorted Vec in fire(): correct for episodic dispatch. The per-fire world scan
  is fine at current well/phantom counts.
- HashMap resource for spawn counters: correct — counters survive node transitions, are
  per-owner, and are only accessed in fire() (not hot path).
- apply_gravity_pull: no allocations, correct query filters. Clean hot path.
