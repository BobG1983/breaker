# Breaker Builder

`Breaker::builder()` — typestate builder for breaker entity construction.

## Why

The breaker was previously constructed across a 4-system pipeline: `spawn_breaker` → `init_breaker_params` → `init_breaker` → `dispatch_breaker_effects`. This spread ~40 components across 4 systems running in sequence during `OnEnter(GameState::Playing)`, with guard conditions to prevent double-initialization on node re-entry. Tests had 6 different ad-hoc spawn patterns, each assembling a different subset of components.

The builder collapses all of this into a single `build()` call that produces every component needed for a valid breaker entity.

## Dimensions: `BreakerBuilder<D, Mv, Da, Sp, Bm, V, R>`

| Dim | Unconfigured | Configured | Transition |
|-----|-------------|-----------|------------|
| **D** (Dimensions) | `NoDimensions` | `HasDimensions` | `.dimensions(width, height, y_position)` |
| **Mv** (Movement) | `NoMovement` | `HasMovement` | `.movement(MovementSettings)` |
| **Da** (Dashing) | `NoDashing` | `HasDashing` | `.dashing(DashSettings)` |
| **Sp** (Spread) | `NoSpread` | `HasSpread` | `.spread(degrees)` |
| **Bm** (Bump) | `NoBump` | `HasBump` | `.bump(BumpSettings)` |
| **V** (Visual) | `Unvisual` | `Rendered` / `Headless` | `.rendered(&mut meshes, &mut materials)` / `.headless()` |
| **R** (Role) | `NoRole` | `Primary` / `Extra` | `.primary()` / `.extra()` |

### Mutually Exclusive

- **Visual**: `Rendered` (includes `Mesh2d` + `MeshMaterial2d`) vs `Headless` (omits them). Most tests use `headless()`.
- **Role**: `Primary` (`PrimaryBreaker` + `CleanupOnExit<RunState>`) vs `Extra` (`ExtraBreaker` + `CleanupOnExit<NodeState>`). Production breakers use `.primary()`.

Four terminal `impl` blocks exist (Rendered+Primary, Rendered+Extra, Headless+Primary, Headless+Extra), each providing `build()` and `spawn()`.

### Definition Shortcut

`.definition(&BreakerDefinition)` transitions **D + Mv + Da + Sp + Bm** simultaneously. This is the production path — one call fills all gameplay dimensions from the definition struct. Also stores lives, effects, and color in optional data.

### Override Methods

After `.definition()` (or after individually satisfying a dimension), values can be overridden:
- `.with_width(f32)`, `.with_height(f32)`, `.with_y_position(f32)` — require `HasDimensions`
- `.with_max_speed(f32)` — requires `HasMovement`
- `.with_reflection_spread(f32)` — requires `HasSpread`, value in degrees

## Settings Structs

```rust
pub struct MovementSettings {
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub decel_ease: EaseFunction,
    pub decel_ease_strength: f32,
}

pub struct DashSettings {
    pub dash: DashParams,
    pub brake: BrakeParams,
    pub settle: SettleParams,
}

pub struct DashParams {
    pub speed_multiplier: f32,
    pub duration: f32,
    pub tilt_angle: f32,      // degrees — converted to radians in build()
    pub tilt_ease: EaseFunction,
}

pub struct BrakeParams {
    pub tilt_angle: f32,      // degrees — converted to radians in build()
    pub tilt_duration: f32,
    pub tilt_ease: EaseFunction,
    pub decel_multiplier: f32,
}

pub struct SettleParams {
    pub duration: f32,
    pub tilt_ease: EaseFunction,
}

pub struct BumpSettings {
    pub perfect_window: f32,
    pub early_window: f32,
    pub late_window: f32,
    pub perfect_cooldown: f32,
    pub weak_cooldown: f32,
    pub feedback: BumpFeedbackSettings,
}

pub struct BumpFeedbackSettings {
    pub duration: f32,
    pub peak: f32,
    pub peak_fraction: f32,
    pub rise_ease: EaseFunction,
    pub fall_ease: EaseFunction,
}
```

## Optional Methods (any typestate)

- `.with_lives(Option<u32>)` — `None` = infinite lives, `Some(n)` = n lives. Also set by `.definition()` from `life_pool`.
- `.with_effects(Vec<RootEffect>)` — breaker-specific effects (normally from `BreakerDefinition`)
- `.with_color([f32; 3])` — HDR color override (normally from `BreakerDefinition.color_rgb`)

## build() Output

Returns `impl Bundle` with every component for a valid breaker entity:

| Category | Components |
|----------|-----------|
| Core | `Breaker`, `BreakerInitialized`, `GameDrawLayer::Breaker` |
| State | `Velocity2D::default()`, `DashState::default()`, `BreakerTilt::default()`, `BumpState::default()`, `DashStateTimer::default()` |
| Spatial | `Position2D`, `PreviousPosition` |
| Scale | `Scale2D`, `PreviousScale` |
| Physics | `Aabb2D`, `CollisionLayers(BREAKER_LAYER, BOLT_LAYER)` |
| Dimensions | `BaseWidth`, `BaseHeight`, `BreakerBaseY`, `MinWidth`, `MaxWidth`, `MinHeight`, `MaxHeight` |
| Movement | `MaxSpeed`, `BreakerAcceleration`, `BreakerDeceleration`, `DecelEasing` |
| Dashing | `DashSpeedMultiplier`, `DashDuration`, `DashTilt`, `DashTiltEase`, `BrakeTilt`, `BrakeDecel`, `SettleDuration`, `SettleTiltEase` |
| Spread | `BreakerReflectionSpread` (radians) |
| Bump | `BumpPerfectWindow`, `BumpEarlyWindow`, `BumpLateWindow`, `BumpPerfectCooldown`, `BumpWeakCooldown`, `BumpFeedback` |
| Lifecycle | `LivesCount(Option<u32>)` (always present, `None` = infinite) |
| Role (Primary) | `PrimaryBreaker`, `CleanupOnExit<RunState>` |
| Role (Extra) | `ExtraBreaker`, `CleanupOnExit<NodeState>` |
| Rendered only | `Mesh2d`, `MeshMaterial2d` |

## spawn() Behavior

1. `commands.spawn(self.build())`
2. If effects are present, queues `commands.dispatch_initial_effects(effects, None)`
3. No entity parameter needed — the command resolves targets from world by convention (Breaker → `(With<Breaker>, With<PrimaryBreaker>)`, Bolt → `With<PrimaryBolt>`)

## Systems Replaced

The builder's `spawn()` replaces 4 systems:
- `spawn_breaker` — replaced by `Breaker::builder().definition(&def).rendered(...).primary().spawn(commands)`
- `init_breaker_params` — all 22+ stat components are now in `build()`
- `init_breaker` — `BreakerInitialized` + `LivesCount` are now in `build()`
- `dispatch_breaker_effects` — `spawn()` queues `dispatch_initial_effects` directly

These are consolidated into `spawn_or_reuse_breaker` in `breaker/systems/`.

## Systems Retained

- `reset_breaker` — resets dynamic state (position, velocity, tilt, bump) on each node entry
- `apply_node_scale_to_breaker` — applies node-specific entity scale from `ActiveNodeLayout`
- `sync_breaker_scale` — updates `Scale2D` each Update from `effective_size()` (size boosts + node scaling)

## Renames

- `BumpVisualParams` → `BumpFeedback` — config for bump recoil animation
- `BumpVisual` → `BumpFeedbackState` — runtime animation state
- `BreakerState` → `DashState` — dash/brake/settle state machine
- `BreakerVelocity` → `Velocity2D` — shared spatial velocity component
- `BreakerWidth`/`BreakerHeight` → `BaseWidth`/`BaseHeight` — shared size components in `shared/size.rs`
- `EntityScale` → `NodeScalingFactor` — layout-derived scale resource
- `BreakerConfig` — eliminated entirely; all fields moved into `BreakerDefinition`
- `BreakerStatOverrides` — eliminated; `BreakerDefinition` is the authoritative source

## Key Files

- `breaker-game/src/breaker/builder/core.rs` — implementation
- `breaker-game/src/breaker/queries.rs` — QueryData structs (`BreakerCollisionData`, `BreakerSizeData`, `BreakerMovementData`, `BreakerDashData`, `BreakerBumpTimingData`, `BreakerBumpGradingData`, `BreakerResetData`, `SyncBreakerScaleData`)
- `breaker-game/src/breaker/definition.rs` — `BreakerDefinition` (36 fields, all with `#[serde(default)]` except `name`)
- `breaker-game/src/shared/size.rs` — `effective_size()`, `effective_radius()` pure functions
- `breaker-game/assets/breakers/breaker.example.ron` — annotated reference showing all fields with defaults
