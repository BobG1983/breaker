---
name: Phase 5 complex effects performance analysis
description: Analysis of chain_lightning, piercing_beam, tether_beam, spawn_bolts, entropy_engine, spawn_phantom, chain_bolt — entity scale, query patterns, quadtree usage, archetype impact
type: project
---

## Phase 5 Entity Scale

- ChainLightningRequest: spawned by fire(), consumed by process_chain_lightning same/next tick. 0-1 at a time.
- PiercingBeamRequest: same deferred-request pattern. 0-1 at a time.
- TetherBeam: 0-1 beam entity + 2 extra bolt entities while active. Long-lived (lasts until bolt despawn).
- SpawnBolts: N extra bolts spawned on fire(). 1-N Bolt entities added (with optional BoltLifespan).
- EntropyEngine: EntropyEngineState component on primary bolt. 0-1 entities.
- SpawnPhantom: PhantomBoltMarker + PhantomOwner bolt entities. 0-max_active at a time.
- ChainBolt: 1 extra bolt + 1 DistanceConstraint entity while active.

All effects remain episodic. Total active bolts with all effects: primary + chain + tether A + tether B + phantoms (up to max_active) = at most ~8 bolt entities simultaneously in extreme edge cases.

## chain_lightning.rs — process_chain_lightning: Missing run_if guard

process_chain_lightning is registered in FixedUpdate (register(), line 119-124) with NO run_if(in_state(PlayingState::Active)).

The system runs every FixedUpdate in all game states — Paused, ChipSelect, etc. It queries
ChainLightningRequest entities (should be 0 while paused) and sends DamageCell messages.

However: requests are short-lived (spawned by fire(), despawned on first process), and fire() is
called from chip dispatch which also only happens in PlayingState::Active. So in practice the
query returns 0 entities in irrelevant states. Minor correctness gap but not a real performance
hit.

## chain_lightning.rs — Uses Transform, not Position2D

fire() reads entity position via world.get::<Transform>(entity) (line 32) and
world.get::<Transform>(target) (line 78). All other bolt-spawning effects use Position2D.

This is a domain consistency issue (the codebase standard is Position2D), but not a
Bevy-performance concern. The Transform lookup itself is O(1). Note: if bolt entities don't
have Transform at all (they use Position2D + sync), fire() silently defaults to Vec2::ZERO.

## chain_lightning.rs — query_circle_filtered per arc in fire()

fire() calls query_circle_filtered once per arc (up to `arcs` iterations) in a tight loop.
Per baseline memory, each call costs: 2 Vec + 2 HashSet + 2 tree walks.

With arcs=5 (example config): 5 × (2 Vec + 2 HashSet) = 10 allocations in a single fire() call.
fire() is called from chip dispatch (episodic: once per cell bump), not every frame.
Zero performance concern. Documenting because the per-arc allocation pattern is worth awareness.

## piercing_beam.rs — process_piercing_beam: Missing run_if guard

Same gap as chain_lightning: process_piercing_beam in FixedUpdate without run_if guard (line 153-157).
Same reasoning: requests are 0 while not in PlayingState::Active. Minor.

## piercing_beam.rs — Uses Transform, not Position2D

fire() reads world.get::<Transform>(entity) (line 36). Same consistency gap as chain_lightning.

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
- ChainLightningRequest: spawned/despawned every chip fire. Novel archetype, short-lived.
- PiercingBeamRequest: same.
- TetherBoltMarker on bolt + TetherBeamComponent beam entity: separate archetype, lives until bolt despawn.
- EntropyEngineState on bolt: one archetype change per node, first kill only.
- PhantomBoltMarker + PhantomOwner on bolt entities: variant of bolt archetype (same as ChainBoltMarker). Short-lived.

All archetype changes are episodic (chip dispatch / cell kill), not per-frame. Clean.

## Intentional Patterns

- process_chain_lightning / process_piercing_beam: FixedUpdate deferred-request pattern is correct.
  Requests live 1 tick, consumed synchronously. run_if gap is acceptable at current scale.
- query_circle_filtered in chain_lightning fire(): correct — callers need circle containment for
  arc chaining. AABB + narrow-phase would also work but fire() is episodic.
- damaged_this_tick HashSet in tether_beam: necessary for dedup, per-beam not per-frame in practice.
- Transform vs Position2D in chain_lightning + piercing_beam: consistency gap but not perf issue.
