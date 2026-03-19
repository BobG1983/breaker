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
- `CellHealth { current: u32, max: u32 }` — has `new(u32)`, `is_destroyed() -> bool`, `take_hit() -> bool`, `take_damage(u32) -> bool`, `fraction() -> f32`
  - After migration: `current: f32, max: f32`, `new(f32)`, `is_destroyed` uses `<= 0.0`, `take_damage(f32)` uses subtraction, `fraction()` uses `(current/max).clamp(0.0, 1.0)`, `take_hit` removed
- `CellWidth(f32)`, `CellHeight(f32)` — with `half_width()`, `half_height()`
- `CellDamageVisuals { hdr_base, green_min, blue_range, blue_base }`
- `CellTypeAlias(char)`

### Resources (`cells/resources.rs`)
- `CellTypeDefinition { hp: u32, ... }` — `hp` field to become `f32` after migration
- `CellTypeRegistry { types: HashMap<char, CellTypeDefinition> }`
- `CellDefaults` / `CellConfig` — grid layout only (width, height, padding), no hp field

### Constants (`shared/mod.rs`)
- `BASE_BOLT_DAMAGE: u32 = 10` — to become `f32 = 10.0` after migration

### Systems
- `handle_cell_hit` — reads BoltHitCell, calls take_damage(BASE_BOLT_DAMAGE * (1+boost)), sends CellDestroyed. Has `#[expect]` cast attributes to remove.

### Cell type RON files (`assets/cells/`)
- `standard.cell.ron` — `hp: 10` → `hp: 10.0`
- `tough.cell.ron` — `hp: 30` → `hp: 30.0`

### Hot-reload systems (`src/debug/hot_reload/systems/`)
- `propagate_cell_type_changes.rs` — sets `health.max = def.hp` and `health.current = health.current.min(def.hp)` — both sides become f32 after migration
- `propagate_node_layout_changes.rs` — uses `CellHealth::new(1)` and `CellHealth::new(3)` inline in test fixtures — become `f32`

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
