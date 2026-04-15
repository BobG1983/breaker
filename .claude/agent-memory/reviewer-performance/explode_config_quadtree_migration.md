---
name: ExplodeConfig::fire() quadtree migration — confirmed clean
description: Follow-up 6 migrated ExplodeConfig::fire() from full world scan to quadtree + narrow phase. Allocation structure reviewed. Death bridges placed in EffectV3Systems::Death set.
type: project
---

## Migration reviewed (Follow-up 6)

`breaker-game/src/effect_v3/effects/explode/config.rs`

### What changed
Old: `world.query_filtered::<(Entity, &Position2D), (With<Cell>, Without<Dead>)>()` — transient QueryState, full archetype scan, O(n_cells).
New: `CollisionQuadtree::query_circle_filtered(pos, radius, CELL_LAYER)` broad phase, then narrow phase via `world.get::<Position2D>(e)` + `world.get::<Dead>(e)` per candidate.

### Quadtree cost structure (confirmed by reading tree.rs)
`query_circle_filtered` is itself a two-stage pipeline:
1. `query_aabb_filtered` (AABB broad phase with layer filter) → allocates one `Vec<Entity>` + one `HashSet<Entity>` internally
2. A `collect_matching_items` tree walk that converts AABB candidates to a second Vec + HashSet for circle refinement

So a single `fire()` call produces:
- Two `Vec<Entity>` allocations inside `query_circle_filtered` (broad candidates + final results)
- Two `HashSet<Entity>` allocations inside `query_circle_filtered` (dedup for AABB pass + dedup for circle pass)
- One additional `Vec<Entity>` collect in `config.rs` for `targets`

Total: 5 allocations per `fire()` call. At N chain-reaction detonations per tick, that is 5N allocations.

### Is this better than the old O(n_cells) scan?
Yes. The old approach was O(n_cells) per call with transient QueryState creation overhead.
The new approach is O(quadtree_nodes_in_radius) for the broad phase (much less than all cells for small explosion radii) plus O(candidates) for the narrow phase world.get calls.
The spatial pruning is the correct fix regardless of the allocation overhead.

### Allocation concern
The quadtree itself allocates 4 Vec/HashSet internally per call (not visible to game code but real).
At current scale (1–few volatile cells, rare chain reactions) this is acceptable.
At Phase 3 if chain reactions hit 20+ detonations in a single tick, the 5N allocations become visible. A future optimization would be to give `query_circle_filtered` capacity hints or return an iterator instead of an owned Vec.

### Narrow phase world.get calls
`world.get::<Position2D>(e)` and `world.get::<Dead>(e)` per candidate are O(1) entity lookups — correct pattern, not a query iteration. No concern.

### world.resource::<CollisionQuadtree>() per fire() call
Returns a shared reference to the already-borrowed resource (read-only). In exclusive World context (&mut World), this is a direct pointer dereference — zero allocation, negligible cost. Not a concern.

### Death bridge scheduling (register.rs)
`EffectV3Systems::Death.after(DeathPipelineSystems::HandleKill)` in FixedUpdate.
All four bridge systems share the same set and the same `Commands` + immutable queries — no world-exclusive access. Normal ECS parallelism applies within the set (Bevy will serialize if any two share a conflicting resource, but all four bridges share identical param signatures so they run sequentially within the set by default). This is unchanged behavior — the set rename from Bridge to Death is a correctness fix with zero performance delta.

## Status
Pre-existing IMPORTANT follow-up from Wave 1 review: RESOLVED. Migration is correct.
Residual minor note: `query_circle_filtered` allocates 4 internal Vec/HashSet per call; acceptable now, possible Phase 3 concern under heavy chain reactions.
