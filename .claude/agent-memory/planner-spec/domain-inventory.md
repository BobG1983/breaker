---
name: domain-inventory
description: Types, systems, and query aliases per domain — physics, cells, bolt, breaker, chips
type: project
---

## physics domain (`src/physics/`)

### Query Aliases (`physics/queries.rs`)
- `CollisionQueryBolt` — (Entity, &mut Transform, &mut BoltVelocity, &BoltBaseSpeed, &BoltRadius)
- `CollisionQueryBreaker` — (&Transform, &BreakerTilt, &BreakerWidth, &BreakerHeight, &MaxReflectionAngle, &MinAngleFromHorizontal)
- `CollisionQueryCell` — (Entity, &Transform, &CellWidth, &CellHeight, Option<&CellHealth>)

### Filters (`physics/filters.rs`)
- `CollisionFilterBreaker` — (With<Breaker>, Without<Bolt>)
- `CollisionFilterCell` — (With<Cell>, Without<Bolt>, Without<Wall>)
- `CollisionFilterWall` — (With<Wall>, Without<Bolt>, Without<Cell>)

### Systems
- `bolt_cell_collision` — CCD sweep bolt vs cells+walls; sends BoltHitCell. Also imports BASE_BOLT_DAMAGE for pierce lookahead effective_damage calculation.
- `bolt_breaker_collision` — CCD sweep bolt vs breaker; sends BoltHitBreaker

### Messages (`physics/messages.rs`)
- `BoltHitCell { cell: Entity, bolt: Entity }` — sent to cells domain
- `BoltHitBreaker { bolt: Entity }` — sent to breaker domain

## cells domain (`src/cells/`)

### Query Aliases (`cells/queries.rs`)
- `DamageVisualQuery` — (&mut CellHealth, &MeshMaterial2d<ColorMaterial>, &CellDamageVisuals, Has<RequiredToClear>)

### Components (`cells/components.rs`)
- `Cell` — marker
- `RequiredToClear` — marker (pub)
- `CellHealth { current: f32, max: f32 }` — has `new(f32)`, `is_destroyed() -> bool`, `take_damage(f32) -> bool`, `fraction() -> f32`
- `CellWidth(f32)`, `CellHeight(f32)` — with `half_width()`, `half_height()`
- `CellDamageVisuals { hdr_base, green_min, blue_range, blue_base }`
- `CellTypeAlias(char)`

### Resources (`cells/resources.rs`)
- `CellTypeDefinition { hp: f32, ... }` — hp is f32 (migration complete)
- `CellTypeRegistry { types: HashMap<char, CellTypeDefinition> }`
- `CellDefaults` / `CellConfig` — grid layout only (width, height, padding), no hp field

### Constants (`shared/mod.rs`)
- `BASE_BOLT_DAMAGE: f32 = 10.0`

### Systems
- `handle_cell_hit` — reads DamageCell (NOT BoltHitCell), calls take_damage(msg.damage), sends CellDestroyed. DamageCell.damage already includes DamageBoost calculation from the sender (bolt_cell_collision or shockwave).

### Cell type RON files (`assets/cells/`)
- `standard.cell.ron` — `hp: 10.0`
- `tough.cell.ron` — `hp: 30.0`

## bolt domain (`src/bolt/`)

### Components (`bolt/components.rs`)
- `Bolt` — marker (pub)
- `BoltServing` — marker (pub), signals bolt on breaker awaiting launch
- `BoltVelocity { value: Vec2 }` — has `speed()`, `direction()`, `enforce_min_angle()`
- `BoltBaseSpeed(f32)`, `BoltMinSpeed(f32)`, `BoltMaxSpeed(f32)`, `BoltRadius(f32)`
- `ExtraBolt` — marker for Prism spawned extra bolts

### Filters (`bolt/filters.rs`)
- `ActiveFilter` — excludes BoltServing (only active bolts)

### Systems
- `prepare_bolt_velocity` — clamps speed to [min, max], enforces min angle. Runs FixedUpdate.

NOTE: The overclock evaluation engine (`ActiveChains`, `EffectFired`, `TriggerKind`, bridge systems, `handle_shockwave`) was unified into the top-level `behaviors/` domain in the refactor/unify-behaviors branch. See behaviors domain below. `bolt/behaviors/` sub-domain no longer exists.

### DamageVisualQuery update
- `DamageVisualQuery` now includes `Has<Locked>` (5th element)

## breaker domain (`src/breaker/`)

### Query Aliases (`breaker/queries.rs`)
- `MovementQuery` — (&mut Transform, &mut BreakerVelocity, &BreakerState, &BreakerMaxSpeed, &BreakerAcceleration, &BreakerDeceleration, &DecelEasing, &BreakerWidth)
- `BumpTimingQuery` — (&mut BumpState, &BumpPerfectWindow, &BumpEarlyWindow, &BumpLateWindow, &BumpPerfectCooldown, &BumpWeakCooldown)
- `BumpGradingQuery` — (&mut BumpState, &BumpPerfectWindow, &BumpLateWindow, &BumpPerfectCooldown, &BumpWeakCooldown)
- NOTE (2026-03-21): BumpPerfectMultiplier and BumpWeakMultiplier DELETED from both queries. Multiplier logic moved to TriggerChain::SpeedBoost leaves in archetype RON.

### Components (core.rs)
- `Breaker` — marker (pub)
- `BreakerWidth(f32)` — `half_width()`
- `BreakerHeight(f32)` — `half_height()`
- `MaxReflectionAngle(f32)`, `MinAngleFromHorizontal(f32)`

### Components (movement.rs)
- `BreakerVelocity { x: f32 }`
- `BreakerTilt { angle, ease_start, ease_target }`
- `BreakerMaxSpeed(f32)`, `BreakerAcceleration(f32)`, `BreakerDeceleration(f32)`
- `DecelEasing { ease: EaseFunction, strength: f32 }`

### Systems
- `move_breaker` — input-driven movement, speed clamping, playfield clamping
- `bump.rs` — `update_bump`, `grade_bump`, `perfect_bump_dash_cancel`
- `tilt_visual.rs` — `animate_tilt_visual` — runs Update, reference for visual-only systems

## chips domain (`src/chips/`)

### Components (`chips/components.rs`)
- `Piercing(u32)` — bolt effect: pierce through N cells
- `PiercingRemaining(u32)` — runtime pierces left; reset by observer when Piercing applied
- `DamageBoost(f32)` — bolt: damage = BASE_BOLT_DAMAGE * (1.0 + self.0)
- `BoltSpeedBoost(f32)` — flat speed added to min/max speed bounds
- `BoltSizeBoost(f32)` — flat radius increase
- `ChainHit(u32)` — chain to N additional cells (future)
- `WidthBoost(f32)` — flat breaker width increase
- `BreakerSpeedBoost(f32)` — flat max speed increase
- `BumpForceBoost(f32)` — flat multiplier increase for bump
- `TiltControlBoost(f32)` — flat angle increase for tilt control

## behaviors domain (`src/behaviors/`) — UNIFIED as of refactor/unify-behaviors

NOTE: This domain was restructured. The old `behaviors/consequences/` directory is GONE. The old `bolt/behaviors/` sub-domain is GONE. Both were merged here.

### Resources (`behaviors/active.rs`)
- `ActiveChains(pub Vec<TriggerChain>)` — runtime active overclock chains (was `ActiveOverclocks` in old `bolt/behaviors/`)

### Components (`behaviors/armed.rs`)
- `ArmedTriggers(pub Vec<TriggerChain>)` — per-bolt partially resolved chains (was in `bolt/behaviors/armed.rs`)

### Events (`behaviors/events.rs`)
- `EffectFired { pub effect: TriggerChain, pub bolt: Option<Entity> }` — fired when chain resolves to leaf (was `OverclockEffectFired` in old `bolt/behaviors/events.rs`)

### Pure functions (`behaviors/evaluate.rs`)
- `evaluate(trigger: TriggerKind, chain: &TriggerChain) -> EvalResult` — NoMatch/Arm/Fire (was `OverclockTriggerKind` in old `bolt/behaviors/evaluate.rs`)
- `TriggerKind` enum (was `OverclockTriggerKind`) — PerfectBump, EarlyBump, LateBump, BumpWhiff, BumpSuccess, CellImpact, BreakerImpact, WallImpact, CellDestroyed, BoltLost

### Systems (`behaviors/bridges.rs`)
- All bridge systems now live here (was `bolt/behaviors/bridges.rs`)
- `bridge_bump`, `bridge_cell_impact`, `bridge_breaker_impact`, `bridge_wall_impact`, `bridge_cell_destroyed`, `bridge_bolt_lost`, `bridge_bump_whiff`

### Effects observers (`behaviors/effects/`)
- `handle_shockwave` in `behaviors/effects/shockwave.rs` (was `bolt/behaviors/effects/shockwave.rs`)
- `handle_life_lost` in `behaviors/effects/life_lost.rs`
- `handle_time_penalty` in `behaviors/effects/time_penalty.rs`
- `handle_spawn_bolt` in `behaviors/effects/spawn_bolt.rs`
- `handle_speed_boost` in `behaviors/effects/speed_boost.rs` — handles TriggerChain::SpeedBoost { target, multiplier }; targets specific bolt from EffectFired.bolt; applies multiplier to bolt velocity
All observe `EffectFired` (not `ConsequenceFired`).

## rantzsoft_spatial2d (`rantzsoft_spatial2d/src/`)

### Components (`components.rs`)
- `Position2D(Vec2)` — Deref/DerefMut, distance(), distance_squared(), arithmetic
- `Rotation2D(Rot2)` — from_degrees, from_radians, as_radians, as_degrees, to_quat
- `Scale2D { x: f32, y: f32 }` — new(), uniform(), to_vec3()
- `PreviousPosition(Vec2)`, `PreviousRotation(Rot2)`, `PreviousScale { x, y }`
- `InterpolateTransform2D` — marker
- `VisualOffset(Vec3)` — pixel offset added to Transform
- `Spatial2D` — #[require] marker for all spatial components

### Propagation (`propagation.rs`)
- `PositionPropagation` — enum { Relative, Absolute }
- `RotationPropagation` — enum { Relative, Absolute }
- `ScalePropagation` — enum { Relative, Absolute }

### Systems
- `save_previous` — FixedFirst — snapshots current to Previous* for InterpolateTransform2D entities
- `propagate_position<D: DrawLayer>` — AfterFixedMainLoop — writes Transform.translation from Position2D + DrawLayer z + VisualOffset, interpolation, parent/child with counteract hack
- `propagate_rotation` — AfterFixedMainLoop — writes Transform.rotation
- `propagate_scale` — AfterFixedMainLoop — writes Transform.scale

### Plugin
- `RantzSpatial2dPlugin<D: DrawLayer>` — generic over DrawLayer

## rantzsoft_physics2d (`rantzsoft_physics2d/src/`)

### Components
- `DistanceConstraint { entity_a, entity_b, max_distance }` — in `constraint.rs`

### Resources
- `CollisionQuadtree` — wraps quadtree spatial index

### Systems
- `maintain_quadtree` — FixedUpdate, PhysicsSystems::MaintainQuadtree — reads Position2D for AABB center

### Plugin
- `RantzPhysics2dPlugin` — registers CollisionQuadtree + maintain_quadtree
- `PhysicsSystems` system set — currently only `MaintainQuadtree` variant

**Why:** Built from reading all domain source files during Phase 4b.2 spec writing (2026-03-19). Updated with CellHealth migration details and hot-reload callsites (2026-03-19). Behaviors domain restructured in refactor/unify-behaviors (2026-03-21). rantzsoft crates inventoried for Phase 11a (2026-03-23).
**How to apply:** Use this to avoid re-reading files when writing future specs in these domains.
