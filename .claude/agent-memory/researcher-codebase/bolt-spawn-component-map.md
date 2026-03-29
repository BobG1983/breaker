---
name: bolt-spawn-component-map
description: Complete bolt entity component inventory, collision layer setup, SpawnAdditionalBolt/SpawnChainBolt message status, phantom bolt pattern, and DistanceConstraint wiring
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

## SpawnAdditionalBolt Message — Status: PLACEHOLDER

`SpawnAdditionalBolt` is defined in `src/bolt/messages.rs`:
```rust
pub struct SpawnAdditionalBolt {
    pub source_chip: Option<String>,
    pub lifespan: Option<f32>,
    pub inherit: bool,
}
```

The docstring says "Consumed by `handle_spawn_bolt` in the effect domain" but that handler
**does not exist**. The effect handler in `src/effect/effects/spawn_bolts.rs::fire()` is a
placeholder — it just logs, does not write `SpawnAdditionalBolt` or spawn actual bolt entities.

`SpawnChainBolt` message is defined (same file) but there is no `handle_chain_bolt` handler
either.

## SpawnChainBolt Message — Status: PLACEHOLDER

```rust
pub struct SpawnChainBolt {
    pub anchor: Entity,
    pub tether_distance: f32,
    pub source_chip: Option<String>,
}
```

Docstring says consumed by `handle_chain_bolt` — handler does not exist.
The `chain_bolt.rs::fire()` effect is a placeholder that spawns entities with
`ChainBoltMarker` + `Transform` only (no bolt components, no DistanceConstraint).

## Phantom Bolt — Status: PLACEHOLDER

`src/effect/effects/spawn_phantom.rs` exists but is NOT the ECS-bolt approach.
It spawns `(PhantomBoltMarker, PhantomTimer, PhantomOwner, Transform)` — no
`Bolt` marker, no physics components. Reads `Transform` (NOT `Position2D`).
Uses `PhantomTimer(f32)` float instead of `BoltLifespan(Timer)`.

This is the OLD placeholder approach. The PLANNED approach (for SpawnPhantom spec work)
would be a real bolt entity with:
- All bolt spawn components (Bolt, Position2D, Velocity2D, Aabb2D, CollisionLayers, ...)
- `ExtraBolt` marker (so it despawns on loss rather than respawning)
- `BoltLifespan(Timer::from_seconds(duration, TimerMode::Once))`
- `PiercingRemaining(u32::MAX)` — or equivalent infinite piercing
- `CleanupOnNodeExit` (NOT `CleanupOnRunEnd` — phantom dies with the node)

## Extra Bolt Pattern (from ExtraBolt marker)

`ExtraBolt` component marks bolt-like entities that:
- Are despawned on loss (via `RequestBoltDestroyed`) NOT respawned
- Have `CleanupOnNodeExit` or handled by bolt_lost
- Used by `LostQuery` to differentiate behavior

## BoltLifespan — Already Implemented

`BoltLifespan(Timer)` is a real component in `src/bolt/components.rs`.
`tick_bolt_lifespan` system in `FixedUpdate` ticks it and writes `RequestBoltDestroyed`
when `just_finished()`. This is a complete, tested, working lifespan system.

## DistanceConstraint Wiring

Component in `rantzsoft_physics2d::constraint::DistanceConstraint`:
```rust
pub struct DistanceConstraint {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub max_distance: f32,
}
```

The constraint is placed on a **third entity** (not on either constrained entity).
`enforce_distance_constraints` queries `Query<&DistanceConstraint>` separately from
`Query<(&mut Position2D, &mut Velocity2D)>` via `get_many_mut`.

System runs in `FixedUpdate` in set `PhysicsSystems::EnforceDistanceConstraints`
(registered by `RantzPhysics2dPlugin`, after `PhysicsSystems::MaintainQuadtree`).

`bolt_lost` runs `.after(PhysicsSystems::EnforceDistanceConstraints)` — the constraint
solver runs first, then loss detection.

Solver behavior:
- If distance <= max_distance: no-op (slack)
- If distance > max_distance (taut): symmetric position correction (each moves half the overshoot)
- Velocity redistribution along constraint axis unless both entities are actively converging
- Perpendicular velocity preserved unchanged
- Missing entity: gracefully skipped (`get_many_mut` returns Err)

For ChainBolt: spawn a DistanceConstraint entity referencing chain_bolt_entity and anchor_entity.
Both constrained entities need `Position2D` and `Velocity2D`.
