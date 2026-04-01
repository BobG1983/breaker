# Breaker Builder

`Breaker::builder()` — typestate builder for breaker entity construction.

## Why

The breaker was previously constructed across a 5-system pipeline: `spawn_breaker` → `init_breaker_params` → `apply_entity_scale_to_breaker` → `init_breaker` → `dispatch_breaker_effects`. This spread ~40 components across 4 systems running in sequence during `OnEnter(GameState::Playing)`, with guard conditions to prevent double-initialization on node re-entry. Tests had 6 different ad-hoc spawn patterns, each assembling a different subset of components.

The builder collapses all of this into a single `build()` call that produces every component needed for a valid breaker entity.

## Dimensions: `BreakerBuilder<D, Mv, Da, Sp, Bm, V>`

| Dim | Unconfigured | Configured | Transition |
|-----|-------------|-----------|------------|
| **D** (Dimensions) | `NoDimensions` | `HasDimensions` | `.dimensions(width, height, y_position)` |
| **Mv** (Movement) | `NoMovement` | `HasMovement` | `.movement(MovementSettings)` |
| **Da** (Dashing) | `NoDashing` | `HasDashing` | `.dashing(DashSettings)` |
| **Sp** (Spread) | `NoSpread` | `HasSpread` | `.spread(degrees)` |
| **Bm** (Bump) | `NoBump` | `HasBump` | `.bump(BumpSettings)` |
| **V** (Visual) | `Unvisual` | `Rendered` / `Headless` | `.rendered(&mut meshes, &mut materials)` / `.headless()` |

### Mutually Exclusive

- **Visual**: `Rendered` (includes `Mesh2d` + `MeshMaterial2d`) vs `Headless` (omits them). Most tests use `headless()`.

### Config Shortcut

`.config(&BreakerConfig)` transitions **D + Mv + Da + Sp + Bm** simultaneously. This is the production path — one call fills all gameplay dimensions from the config resource.

### Config Overrides

After `.config()` (or after individually satisfying a dimension), values can be overridden:
- `.with_width(f32)`, `.with_height(f32)`, `.with_y_position(f32)`
- `.with_max_speed(f32)`, `.with_acceleration(f32)`, `.with_deceleration(f32)`
- `.with_dash_speed_multiplier(f32)`, `.with_reflection_spread(f32)`, etc.

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
    pub tilt: f32,
    pub tilt_ease: EaseFunction,
}

pub struct BrakeParams {
    pub tilt: f32,
    pub duration: f32,
    pub ease: EaseFunction,
    pub decel: f32,
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
    pub feedback: BumpFeedback,
}
```

## Optional Methods (any typestate)

- `.with_lives(u32)` — inserts `LivesCount`
- `.with_effects(Vec<RootEffect>)` — breaker-specific effects (normally from `BreakerDefinition`)

## build() Output

Returns `impl Bundle` with every component for a valid breaker entity:

| Category | Components |
|----------|-----------|
| Core | `Breaker`, `BreakerInitialized`, `CleanupOnRunEnd` |
| State | `BreakerVelocity::default()`, `BreakerState::default()`, `BreakerTilt::default()`, `BumpState::default()`, `BreakerStateTimer::default()` |
| Spatial | `Spatial` (via `Spatial::builder()`), `BaseSpeed`, `MaxSpeed`, `Position2D`, `PreviousPosition` |
| Scale | `Scale2D`, `PreviousScale` |
| Physics | `Aabb2D`, `CollisionLayers(BREAKER_LAYER, BOLT_LAYER)` |
| Draw | `GameDrawLayer::Breaker` |
| Dimensions | `BreakerWidth`, `BreakerHeight`, `BreakerBaseY` |
| Movement | `BreakerAcceleration`, `BreakerDeceleration`, `DecelEasing` |
| Dashing | `DashSpeedMultiplier`, `DashDuration`, `DashTilt`, `DashTiltEase`, `BrakeTilt`, `BrakeDecel`, `SettleDuration`, `SettleTiltEase` |
| Spread | `BreakerReflectionSpread` |
| Bump | `BumpPerfectWindow`, `BumpEarlyWindow`, `BumpLateWindow`, `BumpPerfectCooldown`, `BumpWeakCooldown`, `BumpFeedback` |
| Optional | `LivesCount(Option<u32>)` (always present, `None` = infinite) |
| Rendered only | `Mesh2d`, `MeshMaterial2d` |

## spawn() Behavior

1. `commands.spawn(self.build())`
2. If `.with_effects()` was provided, queues `commands.dispatch_initial_effects(effects, source_chip)`
3. No entity parameter needed — the command resolves targets from world by convention (Breaker → `With<Breaker>`, Bolt → `With<PrimaryBolt>`, AllX → defer to first breaker)

## Systems Eliminated

- `init_breaker_params` — all 22 stat components now in `build()`
- `init_breaker` — `BreakerInitialized` + `LivesCount` now in builder
- `spawn_breaker` — replaced by `Breaker::builder().config(&cfg).rendered(...).spawn(world)`

## Systems Retained

- `reset_breaker` — resets dynamic state (position, velocity, tilt, bump) on each node entry
- `apply_entity_scale_to_breaker` — applies node-specific entity scale from layout

## Renames

- `BumpVisualParams` → `BumpFeedback` — config for bump recoil animation
- `BumpVisual` → `BumpFeedbackState` — runtime animation state

## Key Files

- `breaker-game/src/breaker/builder/` — implementation
- `breaker-game/src/breaker/queries.rs` — QueryData structs (migrated from type aliases)
- `breaker-game/src/breaker/resources.rs` — `BreakerConfig`
