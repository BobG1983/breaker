# Breaker Builder Pattern — Detailed Implementation Spec

## Overview

Implement a typestate builder for `Breaker` entities, eliminating the multi-system init pipeline. Simultaneously: add a Visual dimension to the Bolt builder, migrate breaker queries to `QueryData` structs, rename `BumpVisualParams` → `BumpFeedback` / `BumpVisual` → `BumpFeedbackState`, and update the "always use builder" convention to cover all entity types.

---

## Part 1: Breaker Builder Implementation

### 1.1 New Files

**`breaker-game/src/breaker/builder/mod.rs`**
- Module wiring: `pub(crate) mod builder; pub use builder::*; #[cfg(test)] mod tests;`

**`breaker-game/src/breaker/builder/builder.rs`**
- Entry point: `impl Breaker { pub fn builder() -> BreakerBuilder<NoDimensions, NoMovement, NoDashing, NoSpread, NoBump, Unvisual> }`
- Typestate markers (12 structs): `NoDimensions`, `HasDimensions`, `NoMovement`, `HasMovement`, `NoDashing`, `HasDashing`, `NoSpread`, `HasSpread`, `NoBump`, `HasBump`, `Unvisual`, `Rendered`, `Headless`
- Settings structs: `MovementSettings`, `DashSettings`, `DashParams`, `BrakeParams`, `SettleParams`, `BumpSettings`
- Optional data: `OptionalBreakerData { lives: Option<u32>, effects: Option<Vec<RootEffect>> }`
- Builder struct: `BreakerBuilder<D, Mv, Da, Sp, Bm, V>`
- Uses `Spatial::builder()` internally for spatial components (Position2D, PreviousPosition, BaseSpeed, MaxSpeed)

### 1.2 Typestate Dimensions

| Dim | Marker | Data stored | Transition method |
|-----|--------|------------|-------------------|
| D | `NoDimensions` → `HasDimensions` | `width: f32, height: f32, y_position: f32` | `.dimensions(w, h, y)` |
| Mv | `NoMovement` → `HasMovement` | `MovementSettings` | `.movement(MovementSettings)` |
| Da | `NoDashing` → `HasDashing` | `DashSettings` | `.dashing(DashSettings)` |
| Sp | `NoSpread` → `HasSpread` | `spread_degrees: f32` | `.spread(degrees)` |
| Bm | `NoBump` → `HasBump` | `BumpSettings` | `.bump(BumpSettings)` |
| V | `Unvisual` → `Rendered`/`Headless` | mesh + material handles / nothing | `.rendered(&mut meshes, &mut mats)` / `.headless()` |

### 1.3 Config Shortcut

```rust
impl<V> BreakerBuilder<NoDimensions, NoMovement, NoDashing, NoSpread, NoBump, V> {
    pub fn config(self, cfg: &BreakerConfig)
        -> BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, V>
```

Internally constructs `HasDimensions`, `HasMovement` (from `MovementSettings`), `HasDashing` (from `DashSettings`), `HasSpread`, `HasBump` (from `BumpSettings`) by reading all `BreakerConfig` fields. After `.config()`, individual `.with_*()` overrides are available.

**Why config is optional**: You can satisfy each dimension individually without a `BreakerConfig` resource. This is useful for tests that need precise control over one parameter group without constructing a full config. You MUST provide all dimensions — config is a convenience, not a requirement.

### 1.4 Override Methods

Available after the relevant dimension is satisfied. Each stores into the dimension's data, overwriting the config-derived value:

**After HasDimensions**: `.with_width(f32)`, `.with_height(f32)`, `.with_y_position(f32)`
**After HasMovement**: `.with_max_speed(f32)`, `.with_acceleration(f32)`, `.with_deceleration(f32)`
**After HasDashing**: `.with_dash_speed_multiplier(f32)`, `.with_dash_duration(f32)`, `.with_reflection_spread(f32)` (this one goes into Spread dim), `.with_settle_duration(f32)`, etc.
**After HasBump**: `.with_bump_perfect_window(f32)`, `.with_bump_early_window(f32)`, etc.

### 1.5 Optional Methods (any typestate)

```rust
impl<D, Mv, Da, Sp, Bm, V> BreakerBuilder<D, Mv, Da, Sp, Bm, V> {
    pub fn with_lives(mut self, n: u32) -> Self { ... }
    pub fn with_effects(mut self, effects: Vec<RootEffect>) -> Self { ... }
}
```

### 1.6 LivesCount Change

**Current**: `LivesCount(u32)` — absent means infinite lives. Systems use `With<LivesCount>` / `Option<&LivesCount>`.

**New**: `LivesCount(Option<u32>)` — always present on breaker entities. `None` = infinite, `Some(n)` = n lives. Systems check `lives.0.map_or(false, |n| n == 0)` instead of `With<LivesCount>`.

**Why**: `build()` returns a fixed `impl Bundle`. Conditional component inclusion in a bundle return type is not possible. Making `LivesCount` always-present with an `Option` interior avoids this.

**Files affected**:
- `breaker-game/src/breaker/components/` — `LivesCount` definition
- `breaker-game/src/run/` — systems that read `LivesCount` (life loss, game over check)
- `breaker-game/src/breaker/systems/init_breaker/` — currently inserts `LivesCount`, will be removed
- Tests that check `With<LivesCount>` or `Option<&LivesCount>`

### 1.7 build() Output

Two terminal impls (Rendered and Headless). Both return `impl Bundle` containing:

**Core** (hardcoded):
- `Breaker` (triggers `#[require]` for `Spatial2D`, `InterpolateTransform2D`)
- `BreakerInitialized`
- `CleanupOnRunEnd`
- `Velocity2D::default()` (replacing `BreakerVelocity`)
- `DashState::default()` (renamed from `BreakerState`)
- `BreakerTilt::default()`
- `BumpState::default()`
- `BreakerStateTimer::default()`
- `GameDrawLayer::Breaker`
- `LivesCount(self.optional.lives)` — always present

**Spatial** (from `Spatial::builder()` using dimensions + movement):
- `Spatial`, `BaseSpeed(max_speed)`, `MaxSpeed(max_speed)`, `Position2D(0.0, y_position)`, `PreviousPosition(0.0, y_position)`

**Scale** (from dimensions):
- `Scale2D { x: width, y: height }`, `PreviousScale { x: width, y: height }`

**Physics** (computed from dimensions):
- `Aabb2D::new(Vec2::ZERO, Vec2::new(width/2, height/2))`
- `CollisionLayers::new(BREAKER_LAYER, BOLT_LAYER)`

**Dimension stats**:
- `BaseWidth(width)`, `BaseHeight(height)`, `MinWidth(min_w)`, `MaxWidth(max_w)`, `MinHeight(min_h)`, `MaxHeight(max_h)`, `BreakerBaseY(y_position)`
- Min/max default to 0.5× and 5× base when RON specifies `None`

**Movement stats**:
- `BreakerAcceleration(accel)`, `BreakerDeceleration(decel)`, `DecelEasing { ease, strength }`

**Dashing stats**:
- `DashSpeedMultiplier(...)`, `DashDuration(...)`, `DashTilt(tilt.to_radians())`, `DashTiltEase(...)`
- `BrakeTilt { angle: brake.tilt.to_radians(), duration, ease }`, `BrakeDecel(brake.decel)`
- `SettleDuration(...)`, `SettleTiltEase(...)`

**Spread**:
- `BreakerReflectionSpread(spread.to_radians())`

**Bump stats**:
- `BumpPerfectWindow(...)`, `BumpEarlyWindow(...)`, `BumpLateWindow(...)`, `BumpPerfectCooldown(...)`, `BumpWeakCooldown(...)`, `BumpFeedback { ... }`

**Rendered only** (if `.rendered()`):
- `Mesh2d(meshes.add(Rectangle::new(1.0, 1.0)))`, `MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::WHITE)))`

### 1.8 spawn() Behavior

```rust
pub fn spawn(self, commands: &mut Commands) -> Entity {
    let entity = commands.spawn(self.build()).id();
    if let Some(effects) = self.optional.effects {
        commands.dispatch_initial_effects(effects, None);
    }
    entity
}
```

Takes `&mut Commands`, not `&mut World`. Effect dispatch is queued via `dispatch_initial_effects` — a new command extension in `effect/commands/ext.rs` that requires no entity parameter. It resolves targets from world by convention.

**Why Commands, not World**: Exclusive world access (`&mut World`) prevents the system from being scheduled in parallel with anything. Since `spawn()` only needs to spawn an entity and queue commands, `Commands` is sufficient.

### 1.9 `dispatch_initial_effects` Command

**Location**: `breaker-game/src/effect/commands/ext.rs`

A new command extension that takes `Vec<RootEffect>` + `source_chip` and dispatches them by convention. No entity parameter — the command resolves all targets from world.

**Conventions** (the routing rules the command enforces):

| Target | Convention | Action |
|--------|-----------|--------|
| `Breaker` | Primary breaker(s) | Query `With<Breaker>`, dispatch children directly |
| `Bolt` | Primary bolt(s) | Query `With<PrimaryBolt>`, dispatch children directly |
| `Cell` | No specific cell at init time | Noop |
| `Wall` | No specific wall at init time | Noop |
| `AllBolts` | Deferred broadcast | Wrap in `Once(When(NodeStart, On(AllBolts, ...)))`, push to first breaker |
| `AllCells` | Deferred broadcast | Wrap in `Once(When(NodeStart, On(AllCells, ...)))`, push to first breaker |
| `AllWalls` | Deferred broadcast | Wrap in `Once(When(NodeStart, On(AllWalls, ...)))`, push to first breaker |

Deferred effects push to the **first** breaker entity (not all breakers) — the primary breaker is the conventional holder for deferred effects.

**Trait extension**:
```rust
/// Queue dispatching initial effects by convention — resolves targets from world.
fn dispatch_initial_effects(&mut self, effects: Vec<RootEffect>, source_chip: Option<String>);
```

**Command struct**:
```rust
pub(crate) struct DispatchInitialEffects {
    effects: Vec<RootEffect>,
    source_chip: Option<String>,
}
```

**Implementation** (`Command for DispatchInitialEffects`):
```rust
fn apply(self, world: &mut World) {
    for root in self.effects {
        let RootEffect::On { target, then } = root;

        match target {
            Target::Breaker => {
                let entities = resolve_target_from_world(Target::Breaker, world);
                for entity in entities {
                    dispatch_children_to(entity, &then, &self.source_chip, world);
                }
            }
            Target::Bolt => {
                let entities = resolve_primary_bolts(world);
                for entity in entities {
                    dispatch_children_to(entity, &then, &self.source_chip, world);
                }
            }
            Target::Cell | Target::Wall => {
                // Noop — can't dispatch initial effects to a specific cell/wall
            }
            Target::AllBolts | Target::AllCells | Target::AllWalls => {
                // Deferred: wrap and push to the first breaker
                let wrapped = EffectNode::Once(vec![
                    EffectNode::When {
                        trigger: Trigger::NodeStart,
                        then: vec![EffectNode::On {
                            target,
                            permanent: true,
                            then,
                        }],
                    },
                ]);
                if let Some(breaker) = first_breaker(world) {
                    push_bound_to(breaker, &self.source_chip, wrapped, world);
                }
            }
        }
    }
}
```

**Helper functions** (all in `effect/commands/ext.rs`):

```rust
/// Resolve a Target to entities using direct world queries.
/// Already exists in ext.rs — used by ResolveOnCommand.
fn resolve_target_from_world(target: Target, world: &mut World) -> Vec<Entity> {
    match target {
        Target::Breaker => {
            let mut query = world.query_filtered::<Entity, With<Breaker>>();
            query.iter(world).collect()
        }
        Target::Bolt | Target::AllBolts => {
            let mut query = world.query_filtered::<Entity, With<Bolt>>();
            query.iter(world).collect()
        }
        Target::Cell | Target::AllCells => {
            let mut query = world.query_filtered::<Entity, With<Cell>>();
            query.iter(world).collect()
        }
        Target::Wall | Target::AllWalls => {
            let mut query = world.query_filtered::<Entity, With<Wall>>();
            query.iter(world).collect()
        }
    }
}

/// Resolve primary bolts only (With<PrimaryBolt>).
fn resolve_primary_bolts(world: &mut World) -> Vec<Entity> {
    let mut query = world.query_filtered::<Entity, With<PrimaryBolt>>();
    query.iter(world).collect()
}

/// Returns the first breaker entity, or None if no breaker exists.
fn first_breaker(world: &mut World) -> Option<Entity> {
    let mut query = world.query_filtered::<Entity, With<Breaker>>();
    query.iter(world).next()
}

/// Dispatch children to a single entity: fire Do immediately, push
/// When/Once/On to BoundEffects. Ensures BoundEffects + StagedEffects exist.
fn dispatch_children_to(
    entity: Entity,
    children: &[EffectNode],
    source_chip: &str,
    world: &mut World,
) {
    // Ensure effect components exist on the target
    if let Ok(mut entity_ref) = world.get_entity_mut(entity) {
        ensure_effect_components(&mut entity_ref);
    } else {
        return;
    }

    for child in children {
        match child {
            EffectNode::Do(effect) => {
                effect.clone().fire(entity, source_chip, world);
            }
            EffectNode::On {
                target: inner_target,
                then: inner_children,
                ..
            } => {
                // Recursively resolve nested On nodes
                let inner_entities = resolve_target_from_world(*inner_target, world);
                for inner_entity in inner_entities {
                    dispatch_children_to(
                        inner_entity,
                        inner_children,
                        source_chip,
                        world,
                    );
                }
            }
            other => {
                push_bound_to(entity, source_chip, other.clone(), world);
            }
        }
    }
}

/// Push a single effect node to an entity's BoundEffects.
/// Ensures BoundEffects + StagedEffects exist on the entity.
fn push_bound_to(
    entity: Entity,
    source_chip: &str,
    node: EffectNode,
    world: &mut World,
) {
    if let Ok(mut entity_ref) = world.get_entity_mut(entity) {
        ensure_effect_components(&mut entity_ref);
        if let Some(mut bound) = entity_ref.get_mut::<BoundEffects>() {
            bound.0.push((source_chip.to_owned(), node));
        }
    }
}
```

**Key design decisions**:

1. **No entity parameter**: The command resolves all targets from world by convention. `Breaker` → `With<Breaker>`, `Bolt` → `With<PrimaryBolt>`, `AllX` → defer to first breaker.

2. **First breaker for deferrals**: `AllBolts`/`AllCells`/`AllWalls` effects are deferred via `Once(When(NodeStart, On(...)))` and pushed to the first breaker entity. Only one breaker holds deferred effects — avoids duplicate dispatch when multiple breakers exist.

3. **Recursive nested On resolution**: `dispatch_children_to` handles `On { target, then }` children by recursively resolving the inner target. This matches `dispatch_chip_effects`' current `dispatch_children` behavior.

4. **Replaces all three dispatch systems**: After migration:
   - `dispatch_breaker_effects` → deleted, `spawn()` calls `dispatch_initial_effects`
   - `dispatch_chip_effects` → chip selection handler calls `commands.dispatch_initial_effects(effects, chip_name)`
   - `dispatch_cell_effects` → future cell builder calls `dispatch_initial_effects`

5. **Source chip passthrough**: Breaker/bolt builders pass `None` (definition effects, not from a chip). Chip dispatch passes `Some(chip_name)`. The `source_chip` propagates through `dispatch_children_to` and `push_bound_to` into every `BoundEffects` entry. `BoundEffects` entries store `Option<String>` — `None` means intrinsic to the entity definition.

### 1.9 Systems Eliminated

| System | Reason | Absorbed by |
|--------|--------|-------------|
| `init_breaker_params` | All 22 stat components now in `build()` | Builder |
| `init_breaker` | `BreakerInitialized` + `LivesCount` now in builder | Builder |
| `spawn_breaker` | Replaced by builder call in production code | Builder |
| `dispatch_breaker_effects` | Effect dispatch moved to `spawn()` | Builder's `spawn()` |

### 1.10 Systems Retained

| System | Why it stays |
|--------|-------------|
| `reset_breaker` | Resets dynamic state per-node (position to center, velocity to 0, tilt/bump cleared). Node re-entry concern, not construction. |
| `apply_entity_scale_to_breaker` | Applies `EntityScale` from `ActiveNodeLayout`. Node-specific, not construction. |

### 1.11 Production Spawn Site Update

**`breaker-game/src/breaker/plugin.rs`** — the `OnEnter(GameState::Playing)` schedule currently runs `spawn_breaker` → `init_breaker_params` → `init_breaker` → `dispatch_breaker_effects` → `reset_breaker`.

New schedule:
1. `spawn_or_reuse_breaker` — if no breaker exists, `Breaker::builder().config(&cfg).rendered(&mut meshes, &mut mats).spawn(world)`. If breaker exists, no-op + emit `BreakerSpawned`.
2. `apply_entity_scale_to_breaker` — same as today
3. `reset_breaker` — same as today

The new `spawn_or_reuse_breaker` is a regular system (uses `Commands` for the builder's `spawn()`). It replaces 4 systems with 1.

---

## Part 2: Renames

### 2.1 BumpVisualParams → BumpFeedback

**Why**: "Visual" is misleading — these parameters control the bump recoil feel, not rendering. They're stamped in headless mode too. "Feedback" accurately describes the bump response animation.

**Files to change** (all in `breaker-game/src/breaker/`):
- `components/bump.rs` — struct definition, field on `BumpSettings`
- `systems/init_breaker_params.rs` — insertion (will be deleted, but if incremental: rename first)
- `systems/trigger_bump_visual.rs` — reads `BumpFeedback` to create `BumpFeedbackState`
- `queries.rs` — referenced in query type aliases
- All test files that reference `BumpVisualParams`

### 2.2 BumpVisual → BumpFeedbackState

**Why**: Pairs with `BumpFeedback`. "State" clarifies it's the runtime animation state (timer, duration, peak_offset), not the configuration.

**Files to change**:
- `components/bump.rs` — struct definition
- `filters.rs` — `BumpTriggerFilter` uses `Without<BumpVisual>` → `Without<BumpFeedbackState>`
- `systems/trigger_bump_visual.rs` — inserts `BumpFeedbackState`
- `systems/tick_bump_visual.rs` — ticks `BumpFeedbackState`
- Test files referencing `BumpVisual`

---

## Part 3: Unified Size System

### 3.1 Problem

Size calculations are scattered across systems with inconsistent logic. `bolt_breaker_collision` correctly applies `ActiveSizeBoosts * EntityScale`, but `breaker_cell_collision` and `breaker_wall_collision` apply only `EntityScale` — **missing `ActiveSizeBoosts`** (bug). There's no single source of truth for "effective entity size" the way `apply_velocity_formula` is the single source of truth for speed.

### 3.2 New Components

Move to `shared/` — cross-domain, used by breaker, bolt, cell, wall:

| Component | Replaces | Description |
|-----------|----------|-------------|
| `BaseWidth(f32)` | `BreakerWidth` | Configured base width |
| `BaseHeight(f32)` | `BreakerHeight` | Configured base height |
| `MinWidth(f32)` | (new) | Lower clamp for effective width |
| `MaxWidth(f32)` | (new) | Upper clamp for effective width |
| `MinHeight(f32)` | (new) | Lower clamp for effective height |
| `MaxHeight(f32)` | (new) | Upper clamp for effective height |
| `BaseRadius(f32)` | `BoltRadius` | Configured base radius |
| `MinRadius(f32)` | (new) | Lower clamp for effective radius |
| `MaxRadius(f32)` | (new) | Upper clamp for effective radius |
| `NodeScalingFactor(f32)` | `EntityScale` | Per-node scale from layout |

Min/max default to 0.5× and 5× base when RON specifies `None`. The builder resolves `None` → default at build time, so components are always present (never `Option` on the entity).

### 3.3 Add `Scale2D` + `PreviousScale` to `SpatialData`

**rantzsoft_spatial2d change**: Add `Scale2D` and `PreviousScale` to the `SpatialData` QueryData struct. Scale is a spatial property — the interpolation system already reads it. After this change, `SpatialData` contains position, velocity, global_position, speed/angle constraints, and scale.

### 3.4 Helper Functions

Location: `shared/size.rs` (new file, cross-domain)

**Rectangular entities** (breaker, cell, wall):
```rust
pub(crate) fn effective_size(
    base_w: &BaseWidth, base_h: &BaseHeight,
    min_w: &MinWidth, max_w: &MaxWidth,
    min_h: &MinHeight, max_h: &MaxHeight,
    size_boosts: Option<&ActiveSizeBoosts>,
    node_scale: Option<&NodeScalingFactor>,
) -> Vec2 {
    let mult = size_boosts.map_or(1.0, ActiveSizeBoosts::multiplier);
    let scale = node_scale.map_or(1.0, |s| s.0);
    Vec2::new(
        (base_w.0 * mult * scale).clamp(min_w.0, max_w.0),
        (base_h.0 * mult * scale).clamp(min_h.0, max_h.0),
    )
}
```

**Circular entities** (bolt):
```rust
pub(crate) fn effective_radius(
    base_r: &BaseRadius, min_r: &MinRadius, max_r: &MaxRadius,
    size_boosts: Option<&ActiveSizeBoosts>,
    node_scale: Option<&NodeScalingFactor>,
) -> f32 {
    let mult = size_boosts.map_or(1.0, ActiveSizeBoosts::multiplier);
    let scale = node_scale.map_or(1.0, |s| s.0);
    (base_r.0 * mult * scale).clamp(min_r.0, max_r.0)
}
```

These are the single source of truth for gameplay dimensions — called everywhere an entity's effective size is needed. Mirrors `apply_velocity_formula` for speed.

### 3.5 QueryData for Size Constraints

```rust
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct ExtentConstraints {
    pub base_width: &'static BaseWidth,
    pub base_height: &'static BaseHeight,
    pub min_width: &'static MinWidth,
    pub max_width: &'static MaxWidth,
    pub min_height: &'static MinHeight,
    pub max_height: &'static MaxHeight,
    pub size_boosts: Option<&'static ActiveSizeBoosts>,
    pub node_scale: Option<&'static NodeScalingFactor>,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct RadiusConstraints {
    pub base_radius: &'static BaseRadius,
    pub min_radius: &'static MinRadius,
    pub max_radius: &'static MaxRadius,
    pub size_boosts: Option<&'static ActiveSizeBoosts>,
    pub node_scale: Option<&'static NodeScalingFactor>,
}
```

Used by nesting into domain-specific QueryData structs.

### 3.6 Scale Sync Systems

`width_boost_visual` → `sync_breaker_scale`: sets `Scale2D = effective_size(...)` each frame so the visual matches gameplay dimensions.

```rust
pub(crate) fn sync_breaker_scale(
    mut query: Query<(&ExtentConstraints, &mut Scale2D), With<Breaker>>
) {
    for (extents, mut scale) in &mut query {
        let size = effective_size(...extents fields...);
        scale.x = size.x;
        scale.y = size.y;
    }
}
```

Add `sync_bolt_scale` for bolt entities — sets `Scale2D` from `effective_radius`.

### 3.7 Bug Fixes

- `breaker_cell_collision`: currently missing `ActiveSizeBoosts`. After migration, uses `ExtentConstraints` → `effective_size()` → correct.
- `breaker_wall_collision`: same bug, same fix.
- All collision systems use `effective_size()` / `effective_radius()` — physics and visuals always agree by construction.

### 3.8 RON Changes

Breaker RON files add optional min/max fields:
```ron
(
    width: 120.0,
    height: 20.0,
    min_width: None,  // defaults to 60.0 (0.5 × base)
    max_width: None,  // defaults to 600.0 (5 × base)
    // ... same for height
)
```

Bolt RON files add radius min/max:
```ron
(
    radius: 8.0,
    min_radius: None,  // defaults to 4.0
    max_radius: None,  // defaults to 40.0
)
```

---

## Part 4: QueryData Migration

### 4.1 Current State

`breaker-game/src/breaker/queries.rs` uses **type aliases** (not `#[derive(QueryData)]` structs). The bolt domain uses proper `QueryData` structs with named fields — these are superior because they provide field-name access instead of tuple indexing. `#[query_data(mutable)]` auto-generates a `ReadOnly` variant — systems that only read use `Query<FooDataReadOnly>` for maximum scheduling parallelism.

### 4.2 Renames Applied in QueryData

| Old | New | Reason |
|-----|-----|--------|
| `BreakerState` | `DashState` | More descriptive — it's the dash state machine (Idle/Dashing/Braking/Settling) |
| `BreakerVelocity` | `Velocity2D` | Unify with spatial crate. y=0 is harmless, opens future design space |
| `BreakerWidth` | `BaseWidth` | Generic cross-domain size component |
| `BreakerHeight` | `BaseHeight` | Generic cross-domain size component |
| `BoltRadius` | `BaseRadius` | Generic cross-domain size component |
| `EntityScale` | `NodeScalingFactor` | Clearer intent — per-node scale from layout |

### 4.3 QueryData Structs

**`BreakerCollisionData`** — used by `bolt_breaker_collision` (read-only variant):
```rust
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerCollisionData {
    pub entity: Entity,
    pub spatial: SpatialData,
    pub extents: ExtentConstraints,
    pub tilt: &'static BreakerTilt,
    pub reflection_spread: &'static BreakerReflectionSpread,
}
```
Tilt is used in reflection — adds `tilt_angle` to the base reflection angle. Confirmed in `bolt_breaker_collision/system.rs:60`.

**`BreakerSizeData`** — used by `breaker_cell_collision`, `breaker_wall_collision` (read-only):
```rust
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerSizeData {
    pub entity: Entity,
    pub spatial: SpatialData,
    pub extents: ExtentConstraints,
}
```

**`BreakerMovementData`** — used by `move_breaker`:
```rust
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerMovementData {
    pub spatial: SpatialData,
    pub state: &'static DashState,
    pub acceleration: &'static BreakerAcceleration,
    pub deceleration: &'static BreakerDeceleration,
    pub decel_easing: &'static DecelEasing,
    pub extents: ExtentConstraints,
    pub speed_boosts: Option<&'static ActiveSpeedBoosts>,
}
```
`MaxSpeed` comes from `SpatialData`. Half-width for playfield clamping from `effective_size()` via `ExtentConstraints`.

**`DashConfig`** — nested read-only config for the dash state machine:
```rust
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct DashConfig {
    pub speed: &'static DashSpeedMultiplier,
    pub duration: &'static DashDuration,
    pub tilt: &'static DashTilt,
    pub tilt_ease: &'static DashTiltEase,
    pub brake_tilt: &'static BrakeTilt,
    pub brake_decel: &'static BrakeDecel,
    pub settle_duration: &'static SettleDuration,
    pub settle_tilt_ease: &'static SettleTiltEase,
}
```

**`BreakerDashData`** — used by `update_breaker_state`:
```rust
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerDashData {
    pub spatial: SpatialData,
    pub state: &'static mut DashState,
    pub tilt: &'static mut BreakerTilt,
    pub timer: &'static mut BreakerStateTimer,
    pub deceleration: &'static BreakerDeceleration,
    pub decel_easing: &'static DecelEasing,
    pub config: DashConfig,
    pub flash_step: Option<&'static FlashStepActive>,
    pub extents: ExtentConstraints,
    pub speed_boosts: Option<&'static ActiveSpeedBoosts>,
}
```
Access: `data.config.speed`, `data.config.brake_tilt`, etc. Eliminates the nested tuple hack.

**`BreakerBumpData`** — unified timing + grading, used by `update_bump` and `grade_bump`:
```rust
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerBumpData {
    pub bump: &'static mut BumpState,
    pub perfect_window: &'static BumpPerfectWindow,
    pub early_window: &'static BumpEarlyWindow,
    pub late_window: &'static BumpLateWindow,
    pub perfect_cooldown: &'static BumpPerfectCooldown,
    pub weak_cooldown: &'static BumpWeakCooldown,
    pub anchor_planted: Option<&'static AnchorPlanted>,
    pub anchor_active: Option<&'static AnchorActive>,
}
```
Both `update_bump` and `grade_bump` use this. Grading ignores `early_window`.

**`BreakerResetData`** — used by `reset_breaker`:
```rust
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerResetData {
    pub spatial: SpatialData,
    pub state: &'static mut DashState,
    pub tilt: &'static mut BreakerTilt,
    pub timer: &'static mut BreakerStateTimer,
    pub bump: &'static mut BumpState,
    pub base_y: &'static BreakerBaseY,
    pub prev_position: Option<&'static mut PreviousPosition>,
}
```

**Bump feedback queries** — `trigger_bump_visual` and `animate_bump_visual` stay as inline queries (small, single-consumer each).

### 4.4 Type Aliases Eliminated

| Old Alias | Replaced By |
|-----------|-------------|
| `CollisionQueryBreaker` | `BreakerCollisionData` (read-only variant) |
| `MovementQuery` | `BreakerMovementData` |
| `DashQuery` (nested tuple) | `BreakerDashData` + nested `DashConfig` |
| `BumpTimingQuery` | `BreakerBumpData` |
| `BumpGradingQuery` | `BreakerBumpData` (same struct, grading ignores `early_window`) |
| `ResetQuery` | `BreakerResetData` |
| `WidthBoostVisualQuery` | inline `(&ExtentConstraints, &mut Scale2D)` in `sync_breaker_scale` |
| `BumpTelemetryQuery` | inline query or `BreakerBumpData` read-only variant |

### 4.5 Files Affected

- `breaker/queries.rs` — rewrite: delete all type aliases, add `#[derive(QueryData)]` structs
- `breaker/systems/move_breaker/system.rs` — `MovementQuery` → `BreakerMovementData`
- `breaker/systems/dash/system.rs` — `DashQuery` → `BreakerDashData` (eliminates nested tuple hack)
- `breaker/systems/spawn_breaker/system.rs` — `ResetQuery` → `BreakerResetData` (in `reset_breaker`)
- `breaker/systems/bump/system.rs` — `BumpTimingQuery`/`BumpGradingQuery` → `BreakerBumpData`
- `breaker/systems/width_boost_visual.rs` — replaced by `sync_breaker_scale`
- `breaker/systems/breaker_cell_collision.rs` — inline tuple → `BreakerSizeData` + `effective_size()` (bug fix)
- `breaker/systems/breaker_wall_collision.rs` — same
- `bolt/systems/bolt_breaker_collision/system.rs` — `CollisionQueryBreaker` → `BreakerCollisionData`
- `breaker/filters.rs` — `BumpTriggerFilter` uses `BumpFeedbackState`
- `debug/bump_telemetry.rs` — `BumpTelemetryQuery` → inline or read-only variant
- All RON files referencing `width`, `height`, `radius` field names

---

## Part 5: Bolt Builder — Visual Dimension

### 4.1 Add V Dimension

Add `V` (Visual) to `BoltBuilder<P, S, A, M, R, V>`:

**New markers**: `Unvisual`, `Rendered`, `Headless` (shared with breaker — define in a common `builder_types` module or duplicate per domain).

**Transitions**:
- `.rendered(&mut Assets<Mesh>, &mut Assets<ColorMaterial>)` → stores handles (unit circle mesh, white material)
- `.headless()` → no rendering components

### 4.2 Terminal Impls

Currently 4 terminal impls (Primary×Serving, Primary×Velocity, Extra×Serving, Extra×Velocity). Each splits into Rendered and Headless variants = 8 total. Use a shared `build_core()` + `build_visual()` pattern to avoid duplication.

### 4.3 spawn() Update

Current `spawn()` inserts mesh/material after spawning. New `spawn()` just does `world.spawn(self.build())` + optional insertions + `transfer_effects`. Mesh/material are in the bundle from `build()`.

### 4.4 with_effects on Bolt

Bolt already has `.with_effects(Vec<(String, EffectNode)>)` for attaching effect nodes. This stays as-is. `spawn()` will call `transfer_effects` after spawning, which dispatches these to the entity's `BoundEffects`.

### 4.5 Config Overrides

Add override methods available after `.config()`:
- `.with_base_speed(f32)`, `.with_min_speed(f32)`, `.with_max_speed(f32)`
- `.with_min_angle_h(f32)`, `.with_min_angle_v(f32)`

### 4.6 Files Affected

- `breaker-game/src/bolt/builder/builder.rs` — add V dimension, rendered/headless, config overrides
- `breaker-game/src/bolt/builder/tests/` — update all tests to use `.headless()` (most) or `.rendered()` (rendering tests)
- `breaker-game/src/bolt/systems/spawn_bolt/system.rs` — stop inserting Mesh2d/MeshMaterial2d after spawn; use `.rendered()` on the builder
- Every effect `fire()` that calls `Bolt::builder()` — add `.headless()` (effects don't add rendering)
- `breaker-scenario-runner/` — scenario runner bolt spawning uses `.headless()`

---

## Part 6: Test Migration

### 5.1 Breaker Tests

All 6 current test helper patterns become:

```rust
// Minimal headless (most tests)
Breaker::builder().config(&test_config()).headless().build()

// Override one stat
Breaker::builder().config(&cfg).with_max_speed(600.0).headless().build()

// With specific dimensions only (no config)
Breaker::builder()
    .dimensions(120.0, 20.0, -250.0)
    .movement(MovementSettings { ... })
    .dashing(DashSettings { ... })
    .spread(75.0)
    .bump(BumpSettings { ... })
    .headless()
    .build()
```

**Files with breaker test helpers to migrate**:
- `breaker/systems/dash/tests/helpers.rs` — `spawn_test_breaker`, `breaker_param_bundle`
- `breaker/systems/move_breaker/tests.rs` — `spawn_breaker_at`, `spawn_breaker`
- `breaker/systems/bump/tests/helpers.rs` — inline `bump_param_bundle`
- `chips/tests/helpers.rs` — `spawn_breaker` for dispatch tests
- `bolt/systems/bolt_lost/tests/shield_tests/helpers.rs` — `spawn_shielded_breaker`
- `breaker/systems/dispatch_breaker_effects/tests/` — minimal `(Breaker, BoundEffects::default())`

### 5.2 Bolt Tests

All bolt tests add `.headless()` before `.build()`. Only `spawn_bolt` system tests use `.rendered()`.

---

## Part 7: Architecture Doc Updates

### 6.1 New Files

- `docs/architecture/builders/pattern.md` — typestate builder pattern explanation (for junior engineers)
- `docs/architecture/builders/bolt.md` — bolt builder specifics
- `docs/architecture/builders/breaker.md` — breaker builder specifics

### 6.2 Updated Files

- `docs/architecture/type_state_builder_pattern.md` — redirect to `builders/` folder
- `docs/architecture/plugins.md` — remove references to `init_breaker_params`, `init_breaker`, `spawn_breaker`
- `docs/architecture/layout.md` — update breaker system list
- `docs/architecture/effects/structure.md` — note `dispatch_breaker_effects` moved to builder

---

## Part 8: Memory Updates

### 7.1 Global Memory

Update `feedback_always_use_bolt_builder.md` → generalize to "always use entity builders when available — `Bolt::builder()`, `Breaker::builder()`. Never manually assemble component tuples for entities that have builders."

---

## Implementation Order

### Wave 1: Renames (no behavior change)
1. `BumpVisualParams` → `BumpFeedback`
2. `BumpVisual` → `BumpFeedbackState`
3. `BreakerState` → `DashState`
4. `BreakerVelocity` → `Velocity2D` (breaker systems migrated to use spatial velocity)
5. `BreakerWidth` → `BaseWidth`, `BreakerHeight` → `BaseHeight`, `BoltRadius` → `BaseRadius`
6. `EntityScale` → `NodeScalingFactor`
7. Update all references + RON files

### Wave 2: Size system (new, cross-domain)
1. Add `Scale2D` + `PreviousScale` to `SpatialData` in rantzsoft_spatial2d
2. Create `BaseWidth`, `BaseHeight`, `MinWidth`, `MaxWidth`, `MinHeight`, `MaxHeight`, `BaseRadius`, `MinRadius`, `MaxRadius`, `NodeScalingFactor` in `shared/`
3. Create `effective_size()` and `effective_radius()` helpers in `shared/size.rs`
4. Create `ExtentConstraints` and `RadiusConstraints` QueryData structs in `shared/`
5. Replace `width_boost_visual` → `sync_breaker_scale`, add `sync_bolt_scale`
6. Fix `breaker_cell_collision` and `breaker_wall_collision` to use `effective_size()` (bug fix: missing `ActiveSizeBoosts`)
7. Update all collision systems to use the helpers
8. Update RON files with optional min/max fields

### Wave 3: LivesCount refactor (no behavior change)
1. Change `LivesCount(u32)` → `LivesCount(Option<u32>)`
2. Update all systems that read LivesCount
3. Update tests

### Wave 4: QueryData migration
1. Convert query type aliases to `#[derive(QueryData)]` structs in `breaker/queries.rs`
2. Create nested `DashConfig` QueryData
3. Nest `SpatialData` and `ExtentConstraints` into breaker QueryData structs
4. Update all breaker systems to use new QueryData field names
5. Verify bolt systems use `ReadOnly` variant where appropriate
6. Update all tests

### Wave 5: `dispatch_initial_effects` command
1. Implement `DispatchInitialEffects` command in `effect/commands/ext.rs`
2. Implement helper functions: `dispatch_children_to`, `push_bound_to`, `first_breaker`, `resolve_primary_bolts`
3. Create settings structs in `breaker/builder/`

### Wave 6: Breaker builder implementation
1. Implement `BreakerBuilder<D, Mv, Da, Sp, Bm, V>` with all typestate transitions
2. Implement `build()` for all terminal states
3. Implement `spawn()` with `dispatch_initial_effects`
4. Write builder tests

### Wave 7: Breaker migration
1. Replace `spawn_breaker` + `init_breaker_params` + `init_breaker` + `dispatch_breaker_effects` with builder
2. Update plugin.rs schedule
3. Migrate all test helpers to use builder
4. Delete eliminated systems

### Wave 8: Bolt builder Visual dimension + size system
1. Add V dimension to `BoltBuilder`
2. Add `.rendered()` / `.headless()`
3. Add config override methods
4. Migrate `BoltRadius` → `BaseRadius` + `RadiusConstraints` in bolt QueryData
5. Add `sync_bolt_scale` system
6. Update `spawn_bolt` system to use `.rendered()`
7. Update all effect `fire()` functions to use `.headless()`
8. Update all bolt tests to use `.headless()`

### Wave 9: Architecture docs + memory
1. Write `docs/architecture/builders/` files (already drafted)
2. Update existing architecture docs
3. Update memory

---

## Wave Dependencies

| Wave | Depends On | Can Parallel With | Reason |
|------|-----------|-------------------|--------|
| **1** (Renames) | — | — | Foundation — touches most files, must go first |
| **2** (Size system) | 1 | — | Uses renamed components from Wave 1 |
| **3** (LivesCount) | 1 | **2** | Only touches `LivesCount` + `run/` — no overlap with size system files |
| **4** (QueryData) | 1, 2 | — | Nests `SpatialData` + `ExtentConstraints` from Wave 2 into breaker queries |
| **5** (dispatch_initial_effects) | 1 | **3, 4** | Only touches `effect/commands/ext.rs` — no overlap with queries or LivesCount |
| **6** (Breaker builder) | 4, 5 | — | Needs QueryData structs (Wave 4) and dispatch command (Wave 5) |
| **7** (Breaker migration) | 6 | — | Replaces systems with builder calls — must follow builder impl |
| **8** (Bolt builder V dim + size) | 2, 7 | — | Uses size system (Wave 2), should follow breaker migration to avoid merge conflicts in shared builder patterns |
| **9** (Docs + memory) | 7, 8 | — | Documents final state — must follow all code changes |

**Parallel groups:**
```
Wave 1 (renames)
    ↓
Wave 2 (size) ──────── Wave 3 (LivesCount) ── Wave 5 (dispatch command)
    ↓                        ↓                       ↓
Wave 4 (QueryData) ─────────┘───────────────────────┘
    ↓
Wave 6 (breaker builder)
    ↓
Wave 7 (breaker migration)
    ↓
Wave 8 (bolt builder V + size)
    ↓
Wave 9 (docs + memory)
```
