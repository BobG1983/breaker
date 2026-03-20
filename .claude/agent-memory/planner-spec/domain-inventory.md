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
- `handle_cell_hit` — reads BoltHitCell, calls take_damage(BASE_BOLT_DAMAGE * (1.0 + boost)), sends CellDestroyed.

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

### Sub-domain: bolt/behaviors/ (overclock evaluation engine)

#### Events (`bolt/behaviors/events.rs`)
- `OverclockEffectFired { pub effect: TriggerChain, pub bolt: Entity }` — Event, fired when chain resolves to leaf. (After Phase 0: was tuple struct `OverclockEffectFired(pub TriggerChain)`)

#### Resources (`bolt/behaviors/active.rs`)
- `ActiveOverclocks(pub Vec<TriggerChain>)` — runtime active overclock chains

#### Components (`bolt/behaviors/armed.rs`)
- `ArmedTriggers(pub Vec<TriggerChain>)` — per-bolt partially resolved chains

#### Systems (`bolt/behaviors/bridges.rs`)
- `bridge_overclock_bump` — reads BumpPerformed (Perfect only), evaluates chains, fires/arms
- `bridge_overclock_impact` — reads BoltHitCell, evaluates active chains + armed triggers
- `bridge_overclock_cell_destroyed` — reads CellDestroyed, evaluates active + armed_all, uses Entity::PLACEHOLDER for bolt
- `bridge_overclock_bolt_lost` — reads BoltLost, evaluates active + armed_all, uses Entity::PLACEHOLDER for bolt
- Private helpers: `evaluate_active_chains`, `evaluate_armed_all`, `evaluate_armed`, `resolve_armed`, `arm_bolt`

#### Pure functions (`bolt/behaviors/evaluate.rs`)
- `evaluate(trigger: OverclockTriggerKind, chain: &TriggerChain) -> EvalResult` — NoMatch/Arm/Fire

#### Plugin (`bolt/behaviors/plugin.rs`)
- `BoltBehaviorsPlugin` — registers ActiveOverclocks, bridge systems in FixedUpdate with ordering constraints

#### Observers (`bolt/behaviors/effects/shockwave.rs`) [Phase 1 — new]
- `handle_shockwave` — observes OverclockEffectFired, pattern matches Shockwave, area damage to cells

### DamageVisualQuery update
- `DamageVisualQuery` now includes `Has<Locked>` (5th element)

## breaker domain (`src/breaker/`)

### Query Aliases (`breaker/queries.rs`)
- `MovementQuery` — (&mut Transform, &mut BreakerVelocity, &BreakerState, &BreakerMaxSpeed, &BreakerAcceleration, &BreakerDeceleration, &DecelEasing, &BreakerWidth)
- `BumpTimingQuery` — (&mut BumpState, &BumpPerfectWindow, &BumpEarlyWindow, &BumpLateWindow, &BumpPerfectCooldown, &BumpWeakCooldown, Option<&BumpPerfectMultiplier>, Option<&BumpWeakMultiplier>)
- `BumpGradingQuery` — (&mut BumpState, &BumpPerfectWindow, &BumpLateWindow, &BumpPerfectCooldown, &BumpWeakCooldown, Option<&BumpPerfectMultiplier>, Option<&BumpWeakMultiplier>)

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

**Why:** Built from reading all domain source files during Phase 4b.2 spec writing (2026-03-19). Updated with CellHealth migration details and hot-reload callsites (2026-03-19).
**How to apply:** Use this to avoid re-reading files when writing future specs in these domains.
