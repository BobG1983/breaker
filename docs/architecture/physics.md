# Physics — FixedUpdate + CCD + Spatial Pipeline

## Timestep

All physics runs in `FixedUpdate` for deterministic behavior. This is required for seeded run reproducibility — the same seed must produce identical physics across hardware. Visual interpolation smooths rendering between fixed ticks via the `AfterFixedMainLoop` propagation pipeline in `rantzsoft_spatial2d`.

## Coordinate System — Position2D is Canonical

`Position2D` (from `rantzsoft_spatial2d`) is the canonical position for all game entities. `Transform` is **derived** by `derive_transform` in `AfterFixedMainLoop` — game systems must never write `Transform` directly. The propagation pipeline:

1. `FixedFirst`: `save_previous` snapshots Position2D/Rotation2D/Scale2D/Velocity2D into Previous* components
2. `FixedUpdate`: game systems read and write `Position2D` (via `Velocity2D` + `apply_velocity` marker, or directly)
3. `AfterFixedMainLoop`: `compute_globals` → `derive_transform` → propagation systems write `Transform` for rendering

This ensures all game logic uses `Position2D` and all rendering uses the derived `Transform`.

## Quadtree Spatial Index

`rantzsoft_physics2d` provides a `CollisionQuadtree` resource (via `RantzPhysics2dPlugin`) containing an incremental quadtree over all entities with `Aabb2D` + `GlobalPosition2D` + `CollisionLayers`. The quadtree is maintained by `maintain_quadtree` in `FixedUpdate` tagged `PhysicsSystems::MaintainQuadtree`.

Collision systems order `.after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree)` to ensure the index is current before queries.

## Collision Layers

`CollisionLayers` (from `rantzsoft_physics2d`) is a bitmask pair (`membership`, `mask`) controlling which entities interact in spatial queries. Filtering rule: `self.mask & other.membership != 0`. All game entities with `Aabb2D` carry `CollisionLayers`. The game defines its own layer bit constants.

## Collision — Swept CCD

Continuous collision detection via ray-vs-expanded-AABB intersection. The bolt's path is traced as a ray each frame; cell and wall AABBs are Minkowski-expanded by the bolt radius so a point-ray test is equivalent to a circle-vs-rectangle test.

- `ray_vs_aabb` in `shared/math.rs` — pure math helper used by `bolt_cell_collision` and `bolt_breaker_collision`
- `MAX_BOUNCES` cap prevents infinite loops in degenerate geometries (e.g., bolt trapped between two cells)
- `CCD_EPSILON` separation gap placed after each collision to prevent floating-point re-contact
- On each hit, the bolt is placed just before the impact point, velocity is reflected, and tracing continues with remaining movement distance
- Collision systems now live in the **bolt domain** (moved from the old `physics/` domain which was dissolved)

## Bolt Reflection

- Direction entirely overwritten on breaker contact — no incoming angle carryover
- Reflection angle determined by: hit position on breaker, breaker tilt state, bump grade
- No perfectly vertical or horizontal reflections — always enforce minimum angle

## Distance Constraints (Chain Bolts)

`DistanceConstraint` (from `rantzsoft_physics2d`) is a component that defines a tethered pair of entities with a maximum separation distance. `enforce_distance_constraints` in the bolt domain (runs `.after(clamp_bolt_to_playfield)`) enforces these constraints each tick. Chain bolts use `DistanceConstraint` to stay tethered to their anchor bolt.

## Spreading Shockwave

The `Shockwave` effect spawns an entity-based expanding wavefront (not an instant area query). The `ShockwaveRadius` component grows each tick via `tick_shockwave`, and `shockwave_collision` queries the `CollisionQuadtree` for cells within the current radius each tick, dealing damage to any newly-entered cells. `animate_shockwave` in `Update` drives the visual neon ring VFX.
