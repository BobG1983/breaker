---
name: bolt-spawn-component-map
description: Complete bolt entity component inventory, CollisionLayers setup, and CCD participation requirements — updated for builder migration (feature/chip-evolution-ecosystem)
type: reference
---

# Bolt Spawn & Component Map

## Full Component Set on a Normal Bolt (spawned by `spawn_bolt`)

**NOTE: As of feature/chip-evolution-ecosystem, `spawn_bolt` is an exclusive system
(`fn(world: &mut World)`) using `Bolt::builder()`. `init_bolt_params` is ELIMINATED —
the builder inserts all config components at spawn time.**

### Inserted by `Bolt::builder().from_config(&config).primary().spawn(world)`:
- `Bolt` — marker (also `#[require]`s `Spatial2D`, `InterpolateTransform2D`, `Velocity2D`)
- `PrimaryBolt` — marker (exclusive to the single primary bolt)
- `CleanupOnRunEnd` — persists across nodes; cleaned only on run end
- `Velocity2D(Vec2)` — zero if serving (node 0), `config.initial_velocity()` otherwise
- `Position2D(Vec2)` — spawn position (breaker.y + spawn_offset_y, breaker.x)
- From `.config(&config)`: `BoltRadius`, `BoltSpawnOffsetY`, `BoltRespawnOffsetY`,
  `BoltRespawnAngleSpread`, `BoltInitialAngle` — bolt-specific gameplay state components
- From `.config(&config)`: `BaseSpeed`, `MinSpeed`, `MaxSpeed`, `MinAngleH`, `MinAngleV`
  from `rantzsoft_spatial2d` — the spatial speed clamp parameters

### Conditionally inserted by builder:
- `BoltServing` — only on node_index == 0 (`.serving()` method)
- `BoundEffects` — when `.with_effects()` called
- `BoltLifespan(Timer)` — when `.with_lifespan(f32)` called
- `SpawnedByEvolution(String)` — when `.spawned_by(&str)` called

### Post-spawn inserts by `spawn_bolt`:
- `Mesh2d(handle)`, `MeshMaterial2d(handle)` — render components added after builder.spawn()
- `EntityScale` — added by `apply_entity_scale_to_bolt` after `spawn_bolt`

### Auto-inserted via `Bolt #[require]` (in `components.rs`):
- `Spatial2D` — triggers insertion of its own `#[require]` set (see below)
- `InterpolateTransform2D`
- `Velocity2D` (default Vec2::ZERO, overridden by explicit value)

### Auto-inserted via `Spatial2D #[require]`:
- `Position2D`, `Rotation2D`, `Scale2D`, `PreviousPosition`, `PreviousRotation`, `PreviousScale`
- `GlobalPosition2D` — REQUIRED by quadtree `maintain_quadtree`
- `GlobalRotation2D`, `GlobalScale2D`
- `PositionPropagation`, `RotationPropagation`, `ScalePropagation`
- `Transform` (derived by spatial plugin)

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

## Bolt collision query type: `BoltCollisionData` (see `src/bolt/queries.rs`)

As of cache-removal refactor, uses `#[derive(QueryData)]` named structs (NOT `CollisionQueryBolt`):
- `entity: Entity`
- `spatial: SpatialData` — from rantzsoft_spatial2d (Position2D + Velocity2D + speed fields)
- `collision: BoltCollisionParams`:
  - `radius: &BoltRadius`
  - `piercing_remaining: Option<&mut PiercingRemaining>`
  - `active_piercings: Option<&ActivePiercings>` — NOT `EffectivePiercing`
  - `active_damage_boosts: Option<&ActiveDamageBoosts>` — NOT `EffectiveDamageMultiplier`
  - `active_speed_boosts: Option<&ActiveSpeedBoosts>`
  - `entity_scale: Option<&EntityScale>`
  - `spawned_by_evolution: Option<&SpawnedByEvolution>`
  - `last_impact: Option<&mut LastImpact>`

**`EffectivePiercing` and `EffectiveDamageMultiplier` do NOT exist** — both were removed
in the Effective* cache-removal refactor. Use `ActivePiercings::total()` and
`ActiveDamageBoosts::multiplier()` instead.
