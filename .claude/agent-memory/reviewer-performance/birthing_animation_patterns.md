---
name: Birthing animation query patterns
description: tick_birthing, all_animate_in_complete, begin_node_birthing, and Without<Birthing> filter placement — all acceptable at bolt scale
type: project
---

Added in the bolt birthing animation feature (branch feature/bolt-birthing-animation).

## tick_birthing
- Query: `(Entity, &mut Birthing, &mut Scale2D, &mut CollisionLayers)` — no filter beyond Birthing presence itself
- No With<Bolt> needed: Birthing is bolt-domain only at this point; adding it would create a new archetype combination but change nothing real
- Runs in FixedUpdate gated by `.or(in_state(NodeState::AnimateIn), in_state(NodeState::Playing))`
- Removes Birthing via Commands.entity().remove::<Birthing>() on completion — one archetype transition per bolt, spawn-time equivalent
- At 1–few bolts: totally acceptable

## begin_node_birthing
- Runs OnEnter(NodeState::AnimateIn) — once per node entry, not hot path
- Query: `(Entity, &Scale2D, &PreviousScale, &CollisionLayers), (With<Bolt>, Without<Birthing>)`
- Without<Birthing> filter is correct and precise — exactly the right entities
- Inserts Birthing + zeroes Scale2D + PreviousScale + CollisionLayers via commands.entity().insert(tuple)
- At 1 bolt: negligible

## all_animate_in_complete
- Query: `(), With<Birthing>` — unit query, just checks emptiness via query.is_empty()
- Runs in FixedUpdate gated by run_if(in_state(NodeState::AnimateIn)) — state-gated, not every frame
- `query.is_empty()` in Bevy 0.18 is O(archetype count) not O(entity count) — cheap
- No Without<Bolt> needed: Birthing is only ever on bolt entities in practice; the query is intentionally broad so it works as a catch-all gate
- Sends ChangeState<NodeState> — message-triggered, one write per transition
- At any bolt count: acceptable

## Without<Birthing> filter proliferation
- sync_bolt_scale: `(With<Bolt>, Without<Birthing>)` — visual sync skips birthing bolts (Scale2D managed by tick_birthing during animation)
- tick_bolt_lifespan: `(With<Bolt>, Without<Birthing>)` — lifespan timer skips birthing bolts (don't count down lifespan during animation)
- hover_bolt: uses ServingFilter which intentionally does NOT exclude Birthing — birthing+serving bolts must still track breaker position
- apply_gravity_pull: `(Without<GravityWell>, Without<Birthing>)` — gravity doesn't deflect birthing bolts (no collision layers active)
- ActiveFilter in filters.rs: `(With<Bolt>, Without<BoltServing>, Without<Birthing>)` — shared filter

These filters do not cause "query cache invalidation" in the sense that archetype moves happen. Bevy 0.18 query cache is keyed on the component set of the archetype, not per-entity. Adding Without<Birthing> creates a new, narrower archetype match — it does not invalidate existing caches. The two relevant archetypes are bolt+Birthing and bolt-without-Birthing; systems already had to handle this split before the filter was added.

**How to apply:** Do not flag Without<Birthing> additions as causing query cache churn. They are correct narrowing filters that prevent wrong behavior during animation.
