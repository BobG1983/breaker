---
name: Phase 1 performance baseline
description: Entity scale, quadtree allocation patterns, and confirmed efficient/inefficient patterns from Phase 1 collision system review
type: project
---

## Entity Scale (Phase 1)

- Bolts: 1 normally, up to ~4 with chain-bolt effect
- Breaker: always 1
- Walls: 4 (fixed playfield boundary)
- Cells: ~50 per node (static, fixed grid)
- Orbit cells (shield mechanic): a few per shield cell, counted within the ~50 total

At this scale, most "concerns" are academic. Flag issues that will bite at Phase 3 scale (200+ entities, multiple bolt types).

## Quadtree API Allocation Behavior (confirmed by reading quadtree.rs)

Every quadtree query allocates. Specific costs:

- `query_aabb_filtered(region, layers)` — 1 Vec + 1 HashSet per call
- `query_circle_filtered(center, r, layers)` — calls `query_aabb_filtered` internally (1 Vec + 1 HashSet), then builds a SECOND HashSet from the results, then does a second full tree walk via `collect_matching_items`. Total: 2 Vec + 2 HashSet + 2 tree walks.
- `cast_circle(origin, dir, max_dist, r, layers)` — calls `query_aabb_filtered` (1 Vec + 1 HashSet), then second HashSet + second tree walk, then allocates Vec<(f32, SweepHit)> for raw hits, then sorts. Returns Vec<SweepHit>.

**Conclusion**: `query_aabb_filtered` is always cheaper than `query_circle_filtered` when the caller already does a precise containment test afterward. Use AABB queries + explicit narrow-phase, not circle queries, for overlap detection.

## Confirmed Efficient Patterns (feature/collision-cleanup verified)

- `bolt_wall_collision.rs` — NOW uses `query_aabb_filtered` (fixed from prior `query_circle_filtered`). CollisionLayers hoisted above bolt loop.
- `cell_wall_collision.rs` — `CollisionLayers::new(CELL_LAYER, WALL_LAYER)` correctly hoisted ABOVE the per-cell loop (line 34). Was inside the loop in a prior version.
- `breaker_cell_collision.rs` and `breaker_wall_collision.rs` — `CollisionLayers` constructed once before the single `breaker_query.single()` result is used. Clean.
- `cleanup_cell` — purely message-driven, no query scan. Only runs when `RequestCellDestroyed` messages are pending. Zero overhead otherwise.
- `cleanup_destroyed_bolts` — same message-driven pattern. Clean.
- `tick_cell_regen` — `With<Cell>` filter is correct; `CellRegen` is the data component. No unnecessary mutable access.
- `rotate_shield_cells` — `With<OrbitCell>` filter narrows correctly. Angle mutation is the only write needed.
- `sync_orbit_cell_positions` — uses `query.get_mut(child)` inside a children loop; correct pattern for parent-child position sync. No unnecessary full-table scans.
- All FixedUpdate collision systems gated with `run_if(in_state(PlayingState::Active))`.

## Placeholder Systems (currently cheap, watch when they activate)

- `breaker_cell_collision` — still returns 0 results today (cells static, breaker doesn't reach cells). One quadtree query + 2 allocations per frame. Add `run_if` guard when moving-cell mechanics arrive.
- `breaker_wall_collision` — same situation. Breaker clamps to playfield bounds in `move_breaker` so overlaps never occur in normal play.
- `cell_wall_collision` — 50 quadtree queries per frame (one per cell), each allocating 1 Vec + 1 HashSet. Static cells never overlap walls. Full cost only when cells can move. At 50 cells this is ~100 allocs/frame; fine for now.

## Archetype Notes

- Cell archetypes are intentionally varied: base `Cell`, `Cell + Locked + LockAdjacents`, `Cell + CellRegen`, `Cell + ShieldParent`, `Cell + OrbitCell + OrbitAngle + OrbitConfig`. This is correct domain modeling. `LockAdjacents(Vec<Entity>)` allocates a Vec per locked cell — acceptable at current scale, note if lock count grows large.
- `CollisionQueryBolt` has three `Option<>` fields (`PiercingRemaining`, `Piercing`, `DamageBoost`). These create multiple archetypes for bolts. With only 1-4 bolts active at a time, this is not a fragmentation concern. Do not change.
