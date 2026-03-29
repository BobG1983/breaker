---
name: bolt-spawn-component-map
description: Complete bolt entity component inventory, CollisionLayers setup, and CCD participation requirements
type: reference
---

# Bolt Spawn & Component Map

## Full Component Set on a Normal Bolt (spawned by `spawn_bolt`)

### Inserted directly by `spawn_bolt` in `commands.spawn((...))`:
- `Bolt` — marker (also `#[require]`s `Spatial2D`, `InterpolateTransform2D`, `Velocity2D`)
- `Velocity2D(Vec2)` — zero if serving (node 0), initial_velocity() otherwise
- `GameDrawLayer::Bolt`
- `Position2D(Vec2)` — spawn position (breaker.y + spawn_offset_y, breaker.x)
- `PreviousPosition(Vec2)` — same as Position2D to prevent interpolation teleport
- `Scale2D { x: radius, y: radius }`
- `PreviousScale { x: radius, y: radius }`
- `Aabb2D::new(Vec2::ZERO, Vec2::new(radius, radius))` — local-space AABB
- `CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER)`
- `Mesh2d(circle_handle)`
- `MeshMaterial2d(color_material_handle)`
- `CleanupOnRunEnd` — persists across nodes; cleaned only on run end

### Conditionally inserted by `spawn_bolt`:
- `BoltServing` — only on node_index == 0 (serving bolt, zero velocity, waits for launch)

### Auto-inserted via `Bolt #[require]` (in `components.rs`):
- `Spatial2D` — triggers insertion of its own `#[require]` set (see below)
- `InterpolateTransform2D`
- `Velocity2D` (default Vec2::ZERO, overridden by explicit value)

### Auto-inserted via `Spatial2D #[require]`:
- `Position2D` (default)
- `Rotation2D` (default)
- `Scale2D` (default, overridden by explicit)
- `PreviousPosition` (default, overridden by explicit)
- `PreviousRotation` (default)
- `PreviousScale` (default, overridden by explicit)
- `GlobalPosition2D` (default) — REQUIRED by quadtree `maintain_quadtree`
- `GlobalRotation2D` (default)
- `GlobalScale2D` (default)
- `PositionPropagation` (default)
- `RotationPropagation` (default)
- `ScalePropagation` (default)
- `Transform` (derived by spatial plugin)

### Inserted by `init_bolt_params` (OnEnter(Playing), after spawn_bolt):
- `BoltBaseSpeed(f32)`
- `BoltMinSpeed(f32)`
- `BoltMaxSpeed(f32)`
- `BoltRadius(f32)`
- `BoltSpawnOffsetY(f32)`
- `BoltRespawnOffsetY(f32)`
- `BoltRespawnAngleSpread(f32)`
- `BoltInitialAngle(f32)`

Guard: `Without<BoltBaseSpeed>` — skips already-initialized bolts.

## CollisionLayers Setup

```rust
CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER)
// membership = 0x01 (bolt is in BOLT_LAYER)
// mask       = 0x0E (bolt detects cells 0x02, walls 0x04, breaker 0x08)
```

Layer constants (`src/shared/collision_layers.rs`):
- `BOLT_LAYER    = 1 << 0 = 0x01`
- `CELL_LAYER    = 1 << 1 = 0x02`
- `WALL_LAYER    = 1 << 2 = 0x04`
- `BREAKER_LAYER = 1 << 3 = 0x08`

## For CCD Participation (quadtree)

`maintain_quadtree` reads `(Entity, &Aabb2D, &GlobalPosition2D, &CollisionLayers)`.
A bolt must have ALL of:
- `Aabb2D` — local-space bounds
- `GlobalPosition2D` — world-space position (comes free from `Spatial2D #[require]`)
- `CollisionLayers` — layer membership + mask

`bolt_cell_collision` queries `CollisionQueryBolt` (see `src/bolt/queries.rs`):
- `Entity`
- `&mut Position2D`
- `&mut Velocity2D`
- `&BoltBaseSpeed`
- `&BoltRadius`
- `Option<&mut PiercingRemaining>`
- `Option<&EffectivePiercing>`
- `Option<&EffectiveDamageMultiplier>`
- `Option<&EntityScale>`
- `Option<&SpawnedByEvolution>`

ActiveFilter = `(With<Bolt>, Without<BoltServing>)`.
