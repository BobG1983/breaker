---
name: bolt-cell collision architecture
description: How bolt-cell CCD works: Minkowski expansion, which components drive radius, two independent scale systems (entity_scale vs compute_grid_scale)
type: project
---

## Bolt-cell CCD pipeline (Bevy 0.18.1)

`bolt_cell_collision` (FixedUpdate, after MaintainQuadtree) calls
`CollisionQuadtree::cast_circle(position, direction, remaining, r, CELL_LAYER)`.

**Bolt radius `r`:** `BoltRadius * NodeScalingFactor` (falls back to `BoltRadius` if no `NodeScalingFactor`).

**Cell collision shape:** `Aabb2D.half_extents` baked in at spawn by `spawn_cells_from_layout`.
`CellWidth` / `CellHeight` are NOT read by the collision system.

**Narrowphase:** each candidate cell AABB is expanded by `r` (`expand_by(r)`) then a point-ray slab test is run — this is Minkowski expansion, equivalent to circle-vs-AABB.

**Broadphase:** swept AABB (`origin ± r` to `end ± r`), layer-filtered.

## Two independent scale systems

- `NodeLayout.entity_scale` → `NodeScalingFactor` on **bolt and breaker only**.
  Applied by `apply_node_scale_to_bolt` on `OnEnter(NodeState::Loading)`.
  Cells never get `NodeScalingFactor`.

- `compute_grid_scale` → uniform grid scale baked into **cell `Aabb2D.half_extents`** at spawn.
  Ensures cells fit in the playfield cell zone. Bolt is unaffected.

These two values are not coordinated. A zoomed-in layout has both operating independently.

## Key collision files
- `bolt/systems/bolt_cell_collision/system.rs` — main loop, reads `BoltRadius * NodeScalingFactor`
- `rantzsoft_physics2d/src/quadtree/tree.rs` — `cast_circle` (broadphase + narrowphase)
- `rantzsoft_physics2d/src/ccd/system.rs` — `ray_vs_aabb`, `CCD_EPSILON = 0.01`
- `rantzsoft_physics2d/src/systems/maintain_quadtree.rs` — registers `Aabb2D` entities into quadtree using `GlobalPosition2D` as center
- `state/run/node/systems/spawn_cells_from_layout/system.rs` — spawns cells, bakes grid scale into `Aabb2D`

**Why:** Researched 2026-04-04 for bolt-collision-on-zoomed-layouts bug investigation.
**How to apply:** When analyzing or modifying bolt-cell collision: the cell shape is `Aabb2D.half_extents`, the bolt radius is `BoltRadius * NodeScalingFactor`, and these are the only inputs to narrowphase geometry. `CellWidth`/`CellHeight` are visual/debug only.
