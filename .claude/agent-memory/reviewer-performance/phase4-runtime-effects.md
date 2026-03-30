---
name: Phase 4 runtime effects performance analysis
description: Analysis of shockwave, pulse, explode, attraction, second_wind, bolt_lost (shield path) — entity scale, query patterns, quadtree usage, archetype impact
type: project
---

## Phase 4 Entity Scale

- Shockwaves: 0 normally, 1-2 briefly per cell hit when effect fires. Very short-lived (despawn when radius reaches max).
- Pulse rings: 0 normally, 1 per bolt per 0.5s interval while PulseEmitter is active. Max handful at a time.
- ExplodeRequests: 0 normally, spawned and despawned within same tick. Truly transient.
- AttractionTypes: 1 bolt only normally. ActiveAttractions component on that bolt.
- SecondWindWall: 0-1 total. Spawned once, despawned on first bolt contact.

All runtime effects are episodic. Systems are gated with run_if(in_state(PlayingState::Active)). Entity counts are tiny.

## Quadtree Usage Pattern: query_circle_filtered in Hot Paths

shockwave.rs (apply_shockwave_damage) and pulse.rs (apply_pulse_damage) both call
query_circle_filtered once per active shockwave/ring per FixedUpdate tick.

Per baseline memory, query_circle_filtered costs: 1 Vec + 1 HashSet (from query_aabb_filtered
broad phase), then a second Vec + HashSet for the circle refinement, plus 2 full tree walks.
Total: 2 Vec + 2 HashSet + 2 tree walks per call.

At current entity scale (0-2 shockwaves, 0-few rings active at once), this is not a problem.
Would become Moderate concern at 10+ simultaneous active rings/shockwaves.

query_aabb_filtered is used correctly in apply_attraction — correct choice because caller does
its own distance comparison afterward. 1 Vec + 1 HashSet per active attraction type per bolt.

## Archetype Analysis

New components added:
- ShockwaveDamaged(HashSet<Entity>) — on shockwave entity. Novel archetype. Shockwave entity
  has: ShockwaveSource, ShockwaveRadius, ShockwaveMaxRadius, ShockwaveSpeed, ShockwaveDamaged,
  Transform. One archetype. Entities are short-lived (seconds), so the archetype is created and
  cleared frequently — but only when shockwave fires, not per-frame.

- PulseEmitter — added to bolt entity at chip fire, removed by reverse(). Causes bolt archetype
  to change. With 1-4 bolts and episodic add/remove, not a fragmentation concern.

- PulseRing entities — PulseRing + PulseSource + PulseRadius + PulseMaxRadius + PulseSpeed +
  PulseDamaged + Transform + CleanupOnNodeExit. Novel archetype. Spawned every 0.5s per emitter,
  despawned when radius exceeds max. Short-lived but episodic.

- ActiveAttractions — added to bolt. Same episodic add/remove pattern as PulseEmitter.

- SecondWindWall — permanently on 0-1 entities. Clean.

## HashSet Allocation: Per-Ring, Not Per-Frame

PulseDamaged(HashSet::new()) is allocated ONCE at ring spawn (tick_pulse_emitter, line 113).
After that, the HashSet is mutated in-place by .insert() in apply_pulse_damage — no per-frame
allocation for the HashSet itself.

Same for ShockwaveDamaged::default() — allocated once at shockwave spawn in fire().

The quadtree query DOES allocate per frame (Vec + HashSet returned from query_circle_filtered).
These are the allocations that matter in the hot path.

## apply_attraction: Quadtree per Active Entry per Bolt per Tick

For each active bolt, for each active attraction entry (1-3 possible types), one
query_aabb_filtered call. With ATTRACTION_SEARCH_RADIUS = 500.0 (covers entire playfield),
this is a full-playfield AABB query. At 1 bolt with 1-3 active attraction types: 1-3 quadtree
queries per tick. Each returns ~50 cell candidates that are then distance-checked in a loop.

This is acceptable at current scale. Would become notable if multiple attracted bolts (chain-bolt
+ attraction) were active simultaneously, giving ~12 quadtree queries per tick. Still fine at
Phase 3, but worth noting.

## manage_attraction_types: query.get_mut inside message loops — CORRECT

attracted.get_mut(msg.bolt) is called inside read() loops at lines 135, 142, 149. This is the
correct pattern for message-driven lookups: reads message to get an entity ID, then fetches
that entity's component by ID. Not a scan. Cost is O(1) per message. No issue.

## bolt_lost shield path: No new performance concerns

The shield reflection branch (lines 100-109) in bolt_lost only runs when has_shield is true AND
a bolt is below the playfield. commands.entity().insert() with 3 components. This is an
exceptional path — fires at most once per bolt per bounce, not every frame. Clean.

Local<Vec<LostBoltEntry>> reuse was already noted as intentional in prior analysis.

## Position2D Migration (completed in feature/full-verification-fixes)

All fire() functions now use entity_position() helper in effects/helpers.rs:
  world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0)
No Transform fallback anywhere. All processing systems use Position2D in queries. Fully clean.

- shockwave/effect.rs: fire() + apply_shockwave_damage both use Position2D. Clean.
- pulse/effect.rs: tick_pulse_emitter + apply_pulse_damage both use Position2D. Clean.
- explode/effect.rs: fire() + process_explode_requests both use Position2D. Clean.
- gravity_well.rs: fire() + apply_gravity_pull both use Position2D. Clean.
- piercing_beam/effect.rs: fire() uses entity_position(). process_piercing_beam uses GlobalPosition2D
  for cell positions (narrow-phase OBB check). Clean.

entity_position() is a single world.get() call — O(1), no allocation. Pure lookup pattern.

Note: gravity_well.rs fire() still allocates Vec<Entity> for cap enforcement on every fire() call,
but fire() is episodic (chip activation), not FixedUpdate. No concern.

## TransferCommand insert-if-absent (commands.rs)

New insert-if-absent logic added before other_children processing (lines 131-136 in commands.rs).
Two get::<Component>().is_none() checks + conditional insert(). Commands run in command flush,
not per-frame. Called at most once per chip transfer event. Not a hot path. Clean.

## Intentional Patterns

- query_circle_filtered in shockwave/pulse: correct — callers NEED circle containment, not
  just AABB overlap. AABB + caller narrow-phase would also work but at current entity scale
  (0-2 shockwaves) the extra allocations don't matter. Flag if ring counts grow >10 simultaneous.
- PulseDamaged HashSet per ring: allocated at spawn, not per frame. Not a hot-path allocation.
- ActiveAttractions(Vec<AttractionEntry>): small Vec (1-3 entries), heap-allocated per chip
  fire. Not per frame.
