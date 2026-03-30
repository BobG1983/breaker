---
name: Phase 5 complex effects performance analysis
description: Analysis of chain_lightning, piercing_beam, tether_beam, spawn_bolts, entropy_engine, spawn_phantom, chain_bolt — entity scale, query patterns, quadtree usage, archetype impact
type: project
---

## Phase 5 Entity Scale

- ChainLightningChain + ChainLightningArc: sequential arc model (reworked). 0-1 chain + 0-1 arc at a time.
- PiercingBeamRequest: deferred-request pattern. 0-1 at a time.
- TetherBeam: 0-1 beam entity + 2 extra bolt entities while active. Long-lived (lasts until bolt despawn).
- SpawnBolts: N extra bolts spawned on fire(). 1-N Bolt entities added (with optional BoltLifespan).
- EntropyEngine: EntropyEngineState component on primary bolt. 0-1 entities.
- SpawnPhantom: PhantomBoltMarker + PhantomOwner bolt entities. 0-max_active at a time.
- ChainBolt: 1 extra bolt + 1 DistanceConstraint entity while active.

All effects remain episodic. Total active bolts with all effects: primary + chain + tether A + tether B + phantoms (up to max_active) = at most ~8 bolt entities simultaneously in extreme edge cases.

## chain_lightning REWORK — tick_chain_lightning (sequential arc model)

chain_lightning was reworked from the instant batch model (ChainLightningRequest +
process_chain_lightning) to a sequential arc model (ChainLightningChain + tick_chain_lightning).

Key facts post-rework:
- register() now has run_if(in_state(PlayingState::Active)) guard (effect.rs line 334). FIXED.
- fire() reads Position2D, falls back to Vec2::ZERO. No Transform fallback. FIXED.
- ChainLightningChain entity: 0-1 active at a time. Short-lived (despawns after last jump).
- ChainLightningArc entity: 0-1 active at a time. Spawned/despawned once per jump.

## chain_lightning REWORK — hit_set: HashSet<Entity> allocation timing

ChainLightningChain.hit_set is allocated once at fire() time (via HashSet::new(), then
hit_set.insert(target)). After that, the HashSet is mutated in-place (.insert()) on each
arc arrival in tick_chain_lightning. Not a per-tick allocation for the HashSet itself.
Clean. Same acceptable pattern as PulseDamaged in phase4.

## chain_lightning REWORK — query_circle_filtered in tick_chain_lightning Idle state

Idle branch calls query_circle_filtered once per idle chain per tick (effect.rs line 205).
Cost: 2 Vec + 2 HashSet + 2 tree walks (per baseline memory).

After quadtree returns candidates, a SECOND filter collect produces a third Vec (line 213):
  let valid: Vec<Entity> = candidates.into_iter().filter(|e| !chain.hit_set.contains(e)).collect();

So total per Idle tick per chain: 3 Vec + 2 HashSet.

With 0-1 chains active and the run_if guard present: 0-3 Vec + 0-2 HashSet per FixedUpdate.
Acceptable at current scale. This is the most allocation-dense path but it's bounded.

## chain_lightning REWORK — arc entity churn

Each jump: spawn ChainLightningArc (entity + Transform + CleanupOnNodeExit) → travel ticks →
despawn. With arcs=5 and arc_speed=200 units/s, a full chain lasts several seconds total.
At any given time: 0-1 arc entity alive.

Spawn/despawn causes archetype invalidation but only at arc arrival frequency (once per several
hundred milliseconds), not per frame. Not a concern.

## chain_lightning REWORK — ChainLightningWorld archetype access

Four queries in ChainLightningWorld:
1. quadtree: Res<CollisionQuadtree> — resource, no entity access
2. cell_positions: Query<&GlobalPosition2D, With<Cell>> — immutable, narrow (Cell archetype only)
3. rng: ResMut<GameRng> — resource, not entity
4. arc_transforms: Query<&mut Transform, With<ChainLightningArc>> — mutable Transform but
   narrowed to ChainLightningArc archetype only. Does not conflict with cell or bolt Transform.

The &mut Transform in arc_transforms prevents any concurrent system from accessing Transform
on any entity — Bevy's conflict detection is type-level, not archetype-level before 0.15+.
In practice there are no other systems that need Transform access in the same FixedUpdate tick
on arc entities. No scheduling conflict at current entity composition.

## chain_lightning REWORK — fire() exclusive world access pattern

fire() takes &mut World (chip dispatch pattern, same as all other effects).
resource_mut::<Messages<DamageCell>>().write() is called once per first target hit.
No performance concern — fire() is called at chip dispatch time, not per frame.

## chain_lightning REWORK — entity scale (confirmed)

Post-rework scale: 0-1 ChainLightningChain + 0-1 ChainLightningArc active at a time.
Previous model had 0 chain entities (request-based). New model: same effective scale,
different lifetime pattern (chain lives for the full arc sequence rather than 1 tick).

## Intentional Patterns (post-rework)

- hit_set HashSet: allocated at spawn, mutated in place. Correct — not a hot-path alloc.
- query_circle_filtered in Idle: correct choice for arc chaining (circle containment needed).
  The second Vec filter is necessary to exclude already-hit targets.
- run_if(in_state(PlayingState::Active)): present. Confirmed fixed from pre-rework.
- arc_transforms &mut Transform: narrowed to ChainLightningArc archetype. Acceptable.

## piercing_beam.rs — process_piercing_beam: Missing run_if guard

Same gap as chain_lightning: process_piercing_beam in FixedUpdate without run_if guard (line 153-157).
Same reasoning: requests are 0 while not in PlayingState::Active. Minor.

## piercing_beam.rs — Uses Position2D with Transform fallback (OPEN)

fire() reads Position2D first, then falls back to Transform (line 37-41). The Transform fallback
is wrong — chain_lightning was fixed to use Position2D-only fallback; piercing_beam has not been fixed yet.

## piercing_beam.rs — query_aabb_filtered in process_piercing_beam — CORRECT CHOICE

process_piercing_beam correctly uses query_aabb_filtered (line 124) with an oriented bounding box
as broad-phase, then does its own OBB narrow-phase check. This is the efficient pattern (1 Vec +
1 HashSet vs 2 Vec + 2 HashSet for circle query). Good.

## tether_beam.rs — tick_tether_beam: Per-tick quadtree query — CORRECT PATTERN

tick_tether_beam calls query_aabb_filtered once per active beam per FixedUpdate tick (line 196).
With 0-1 beams active: 0-1 quadtree queries per tick (1 Vec + 1 HashSet).
Has correct run_if(in_state(PlayingState::Active)) guard (line 243).

## tether_beam.rs — HashSet allocated per beam per tick

damaged_this_tick: HashSet<Entity> is allocated fresh every tick per active beam (line 204).
With 0-1 beams: 0-1 HashSet allocations per FixedUpdate tick.
Acceptable at current scale. Would become Moderate at 5+ simultaneous beams.

The HashSet is used to deduplicate candidates from the quadtree — necessary because
query_aabb_filtered may return duplicates. Cannot easily avoid this.

## spawn_bolts.rs — BoundEffects.clone() per spawned bolt — Intentional

If inherit=true, BoundEffects is cloned once per spawned bolt (line 96).
BoundEffects(Vec<(String, EffectNode)>) — clone allocates a new Vec + strings.
fire() is episodic (chip dispatch), not per-frame. Clean.

## spawn_bolts.rs — resource_mut::<GameRng> inside spawn loop — Minor

GameRng is borrowed mutably inside the spawn loop, then released, then re-borrowed next iteration.
This is a World borrow pattern — not a performance issue in the allocator sense, but each
reborrow forces a lookup. For count=3, it's 3 separate borrows. Negligible.

## spawn_phantom.rs — owned.remove(0) is O(N) on the enforcement loop

While max_active is exceeded, owned.remove(0) shifts the Vec on each removal.
In practice max_active is small (2-5). O(N) on a 5-element Vec is unmeasurable.
Not a concern.

## entropy_engine.rs — Vec<f32> weights allocation per fire()

Line 55: `let weights: Vec<f32> = pool.iter().map(|(w, _)| *w).collect();`
This allocates a new Vec on every fire() call. Pool sizes are small (config-time constants, likely
3-10 entries). fire() is episodic (once per cell kill on the primary bolt).
Acceptable. Could be pre-built once at chip bind time, but not worth it at this scale.

## entropy_engine.rs — EntropyEngineState component add causes archetype change

First fire() call inserts EntropyEngineState onto the bolt entity (line 30-33).
This causes an archetype change on the bolt entity. Happens once per node start (at first kill).
Not a per-frame concern.

## Archetype Impact Summary

New episodic archetypes from Phase 5:
- ChainLightningChain + EffectSourceChip + CleanupOnNodeExit: spawned by fire(), lives for
  full arc sequence (seconds), despawned on last jump. Novel archetype. (Previous model used
  ChainLightningRequest — that archetype is gone post-rework.)
- ChainLightningArc + Transform + CleanupOnNodeExit: spawned/despawned once per jump during
  arc travel. Short-lived per jump.
- PiercingBeamRequest: same deferred-request archetype pattern as before.
- TetherBoltMarker on bolt + TetherBeamComponent beam entity: separate archetype, lives until bolt despawn.
- EntropyEngineState on bolt: one archetype change per node, first kill only.
- PhantomBoltMarker + PhantomOwner on bolt entities: variant of bolt archetype (same as ChainBoltMarker). Short-lived.

All archetype changes are episodic (chip dispatch / cell kill / arc arrival), not per-frame. Clean.

## Intentional Patterns

- tick_chain_lightning: sequential arc model — one quadtree query per Idle chain per tick.
  run_if guard present. 0-1 chains active at any time. Correct.
- query_circle_filtered in tick_chain_lightning Idle: correct choice for arc-to-arc chaining
  (circle containment semantics needed). Valid: Vec filter is necessary for hit_set exclusion.
- process_piercing_beam: FixedUpdate deferred-request pattern is correct. run_if gap is
  acceptable at current scale.
- damaged_this_tick HashSet in tether_beam: necessary for dedup, per-beam not per-frame in practice.
- Transform vs Position2D in piercing_beam: consistency gap but not perf issue (O(1) lookup).
