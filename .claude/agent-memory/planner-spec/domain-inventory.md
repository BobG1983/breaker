---
name: domain-inventory
description: Types, systems, and query aliases per domain — physics, cells, bolt, breaker, chips
type: project
---

## ~~physics domain~~ DELETED 2026-03-24 (spatial/physics extraction)

Collision systems and messages moved to bolt domain. `physics/` game domain no longer exists.

### Moved to bolt domain (`src/bolt/`)
- `bolt_cell_collision` — CCD sweep bolt vs cells+walls; sends BoltHitCell, DamageCell, BoltHitWall
- `bolt_breaker_collision` — CCD sweep bolt vs breaker; sends BoltHitBreaker
- Query aliases (`bolt/queries.rs`): CollisionQueryBolt uses Position2D (not Transform)
- Filters (`bolt/filters.rs`): CollisionFilterBreaker, CollisionFilterCell, CollisionFilterWall

### Messages (`bolt/messages.rs`) — moved from physics/messages.rs
- `BoltHitCell { cell: Entity, bolt: Entity }` — sent to cells/behaviors domain
- `BoltHitBreaker { bolt: Entity }` — sent to breaker domain
- `BoltHitWall { bolt: Entity }` — sent to effect domain (wall: Entity field planned for C7 Wave 2a)
- `BoltLost` — sent to bolt/effect domain
- NOTE (C7 Wave 2a): `RequestBoltDestroyed { bolt: Entity }` and `BoltDestroyedAt { position: Vec2 }` planned for two-phase destruction

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
- NOTE (C7 Wave 2a): `CellDestroyed` replaced by two-phase `RequestCellDestroyed { cell }` + `CellDestroyedAt { position, was_required_to_clear }`. handle_cell_hit will write RequestCellDestroyed instead of despawning; cleanup system despawns later.

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

## effect domain (`src/effect/`) — RENAMED from behaviors (B12 refactor complete)

NOTE: The `behaviors/` domain was refactored into `effect/`. All effect types, typed events, bridges, evaluate, active chains, armed triggers, and per-effect handlers now live here. EffectPlugin in `effect/plugin.rs`.

### Resources
- `ActiveEffects(pub Vec<(Option<String>, TriggerChain)>)` in `effect/active.rs` — runtime active trigger chains
- `BreakerRegistry` in `effect/registry.rs` — name->BreakerDefinition lookup

### Components
- `ArmedEffects(pub Vec<(Option<String>, TriggerChain)>)` in `effect/armed.rs` — per-bolt partially resolved chains
- `EffectTarget` in `effect/definition.rs` — marker for entities that can have effects
- NOTE (C7 Wave 2a): `EffectChains(Vec<EffectNode>)` planned in `effect/components.rs` — entity-local effect chains
- NOTE (C7 Wave 2a): `UntilTimers`, `UntilTriggers` planned in `effect/effects/until.rs`
- NOTE (C7 Wave 2b): `AttractionState { active_types: HashSet<AttractionType> }` planned in `effect/effects/attraction.rs`
- NOTE (C7 Wave 2b): `SecondWindWall` marker planned in `effect/effects/second_wind.rs`

### Typed Events (`effect/typed_events.rs`)
- Triggered: `ShockwaveFired`, `LoseLifeFired`, `TimePenaltyFired`, `SpawnBoltsFired`, `SpeedBoostFired`, `ChainBoltFired`, `MultiBoltFired`, `ShieldFired`, `ChainLightningFired`, `SpawnPhantomFired`, `PiercingBeamFired`, `GravityWellFired`, `SecondWindFired`, `RandomEffectFired`, `EntropyEngineFired`
- DELETED (C7 Wave 1): `TimedSpeedBurstFired`, `OneShotDamageBoostFired`, `TimePressureBoostApplied` — replaced by Until/When(OnTimerThreshold) EffectNode trees
- Passive: `PiercingApplied`, `DamageBoostApplied`, `SpeedBoostApplied`, `ChainHitApplied`, `SizeBoostApplied`, `AttractionApplied`, `BumpForceApplied`, `TiltControlApplied`, `RampingDamageApplied`
- Dispatch: `fire_typed_event`, `fire_passive_event` (NOTE: `trigger_chain_to_effect`, `chain_to_passive_effect`, `chain_to_triggered_effect` DELETED in C7 Wave 1)

### Definition types (`effect/definition.rs`)
- `Effect` enum — 23 variants (Shockwave through EntropyEngine; TimedSpeedBurst/TimePressureBoost/OneShotDamageBoost DELETED in C7 Wave 1, SpawnBolt replaced by SpawnBolts)
- `EffectNode` enum — `When { trigger, then }` | `Do(Effect)` | `Until { until, then }` | `Once(Vec<EffectNode>)` (migrated in C7 Wave 1)
- `Trigger` enum — 12 variants (OnPerfectBump through OnSelected, plus TimeExpires(f32), OnDeath, OnTimerThreshold(f32); does NOT derive Eq; KEEPS Copy)
- `Target` — Bolt, Breaker, AllBolts
- `ImpactTarget` — Cell, Breaker, Wall
- `BreakerDefinition`, `BreakerStatOverrides`

### Pure functions (`effect/evaluate.rs`)
- `evaluate(trigger: TriggerKind, chain: &TriggerChain) -> Vec<EvalResult>` — NoMatch/Arm/Fire
- `evaluate_node(trigger: TriggerKind, node: &EffectNode) -> Vec<NodeEvalResult>`
- `TriggerKind` enum — PerfectBump, BumpSuccess, EarlyBump, LateBump, BumpWhiff, CellImpact, BreakerImpact, WallImpact, CellDestroyed, BoltLost. Wave 2a adds: Death

### Bridge systems (`effect/bridges.rs`)
- `bridge_bump`, `bridge_cell_impact`, `bridge_breaker_impact`, `bridge_wall_impact`, `bridge_cell_destroyed`, `bridge_bolt_lost`, `bridge_bump_whiff`
- Helper: `fire_leaf(leaf, bolt, source_chip, commands)` — converts TriggerChain leaf -> Effect -> typed event

### Per-effect handlers (`effect/effects/`)
- Triggered: shockwave, life_lost, time_penalty, spawn_bolt, speed_boost, chain_bolt, multi_bolt, shield, chain_lightning, spawn_phantom, piercing_beam, gravity_well, second_wind, random_effect, entropy_engine
- DELETED (C7 Wave 1): timed_speed_burst, time_pressure_boost, one_shot_damage_boost — replaced by Until/When(OnTimerThreshold) EffectNode trees
- Passive: piercing, damage_boost, bolt_speed_boost, chain_hit, bolt_size_boost, width_boost, breaker_speed_boost, bump_force_boost, tilt_control_boost, attraction, ramping_damage
- Helpers in `effect/effects/mod.rs`: `stack_u32`, `stack_f32`

### Test convenience constructors (`chips/definition.rs` — #[cfg(test)] impl TriggerChain)
- `test_shockwave(range: f32)`, `test_multi_bolt(count: u32)`, `test_shield(duration: f32)`, `test_lose_life()`, `test_time_penalty(seconds: f32)`, `test_spawn_bolt()`, `test_speed_boost(multiplier: f32)`, `test_chain_bolt(tether_distance: f32)`

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

### Systems (registered by plugin)
- `save_previous` — FixedFirst (SpatialSystems::SavePrevious) — snapshots current to Previous* for InterpolateTransform2D entities
- `apply_velocity` — FixedUpdate (SpatialSystems::ApplyVelocity) — advances Position2D for entities with ApplyVelocity marker
- `compute_globals` — AfterFixedMainLoop (SpatialSystems::ComputeGlobals) — computes Global* from hierarchy
- `derive_transform<D: DrawLayer>` — AfterFixedMainLoop (SpatialSystems::DeriveTransform) — writes Transform from Global* + interpolation

`propagate_position`, `propagate_rotation`, `propagate_scale` are NOT registered by the plugin (pub(crate) only).

### Plugin and SystemSet
- `RantzSpatial2dPlugin<D: DrawLayer>` — generic over DrawLayer
- `SpatialSystems` — system set enum exported from plugin module; 4 variants for cross-system ordering

## rantzsoft_physics2d (`rantzsoft_physics2d/src/`)

### Components
- `DistanceConstraint { entity_a, entity_b, max_distance }` — in `constraint.rs`

### Resources
- `CollisionQuadtree` — wraps quadtree spatial index

### Systems
- `maintain_quadtree` — FixedUpdate, PhysicsSystems::MaintainQuadtree — reads Position2D for AABB center

### Plugin and SystemSet
- `RantzPhysics2dPlugin` — registers CollisionQuadtree + maintain_quadtree + enforce_distance_constraints
- `PhysicsSystems` system set — `MaintainQuadtree` and `EnforceDistanceConstraints` variants
- Prelude available: `use rantzsoft_physics2d::prelude::*` re-exports all public types including `SweepHit` and `cast_circle` (via Quadtree method)

**Why:** Built from reading all domain source files during Phase 4b.2 spec writing (2026-03-19). Updated with CellHealth migration details and hot-reload callsites (2026-03-19). Behaviors domain restructured in refactor/unify-behaviors (2026-03-21). rantzsoft crates inventoried for Phase 11a (2026-03-23).
**How to apply:** Use this to avoid re-reading files when writing future specs in these domains.
