---
name: breaker-spawn-inventory
description: Complete inventory of every Breaker entity spawn site, all component layers, config vs hardcoded values, test helper patterns
type: reference
---

# Breaker Spawn Inventory

Bevy version: 0.18. Traced on branch `feature/chip-evolution-ecosystem`.

---

## Breaker Component: `#[require]` Chain

Defined in `breaker-game/src/breaker/components/core.rs:8`:

```rust
#[derive(Component, Debug, Default)]
#[require(Spatial2D, InterpolateTransform2D)]
pub struct Breaker;
```

Every spawn of `Breaker` automatically inserts:
- `Spatial2D` (rantzsoft_spatial2d marker)
- `InterpolateTransform2D` (rantzsoft_spatial2d interpolation marker)

`CleanupOnRunEnd` and `CleanupOnNodeExit` are explicitly NOT in `#[require]` — this is verified by tests at core.rs:93 and core.rs:106.

---

## Production Spawn Site

### Site 1: `spawn_breaker`
**File:** `breaker-game/src/breaker/systems/spawn_breaker/system.rs:37`
**Schedule:** `OnEnter(GameState::Playing)`, runs first in chain
**Guard:** `if !existing.is_empty() { send BreakerSpawned; return }` — no double-spawn

Components inserted in a single `commands.spawn((…))`:

| Component | Source |
|---|---|
| `Breaker` | marker (triggers `#[require]` for `Spatial2D`, `InterpolateTransform2D`) |
| `BreakerVelocity::default()` | hardcoded default |
| `BreakerState::default()` | hardcoded default (Idle) |
| `BreakerTilt::default()` | hardcoded default |
| `BumpState::default()` | hardcoded default |
| `BreakerStateTimer::default()` | hardcoded default |
| `GameDrawLayer::Breaker` | hardcoded enum variant |
| `Position2D(Vec2::new(0.0, config.y_position))` | `BreakerConfig::y_position` (default: -250.0) |
| `PreviousPosition(Vec2::new(0.0, config.y_position))` | `BreakerConfig::y_position` — snapped to prevent teleport |
| `Scale2D { x: config.width, y: config.height }` | `BreakerConfig::width` / `height` |
| `PreviousScale { x: config.width, y: config.height }` | `BreakerConfig::width` / `height` |
| `Aabb2D::new(Vec2::ZERO, Vec2::new(config.width/2, config.height/2))` | `BreakerConfig::width` / `height` |
| `CollisionLayers::new(BREAKER_LAYER, BOLT_LAYER)` | shared constants |
| `Mesh2d(meshes.add(Rectangle::new(1.0, 1.0)))` | unit rectangle, scaled by Scale2D |
| `MeshMaterial2d(materials.add(ColorMaterial::from_color(config.color())))` | `BreakerConfig::color_rgb` |
| `CleanupOnRunEnd` | hardcoded — breaker is destroyed on run end |

After spawn, emits `BreakerSpawned` message.

---

## Post-Spawn Initialization (same `OnEnter(GameState::Playing)`)

These systems run after `spawn_breaker` and add more components via `commands.entity(entity).insert(…)`.

### Layer 2: `init_breaker_params`
**File:** `breaker-game/src/breaker/systems/init_breaker_params.rs:21`
**Guard:** `Without<MaxSpeed>` — skips already-initialized breakers (persisted across nodes)

Inserts all config-derived stat components:

| Component | Source |
|---|---|
| `BreakerWidth(config.width)` | `BreakerConfig::width` (default: 120.0) |
| `BreakerHeight(config.height)` | `BreakerConfig::height` (default: 20.0) |
| `BreakerBaseY(config.y_position)` | `BreakerConfig::y_position` (default: -250.0) |
| `MaxSpeed(config.max_speed)` | `BreakerConfig::max_speed` (default: 500.0) |
| `BreakerAcceleration(config.acceleration)` | `BreakerConfig::acceleration` (default: 3000.0) |
| `BreakerDeceleration(config.deceleration)` | `BreakerConfig::deceleration` (default: 2500.0) |
| `DecelEasing { ease, strength }` | `BreakerConfig::decel_ease` / `decel_ease_strength` |
| `DashSpeedMultiplier(config.dash_speed_multiplier)` | `BreakerConfig::dash_speed_multiplier` (default: 4.0) |
| `DashDuration(config.dash_duration)` | `BreakerConfig::dash_duration` (default: 0.15) |
| `DashTilt(config.dash_tilt_angle.to_radians())` | `BreakerConfig::dash_tilt_angle` (default: 15.0°) |
| `DashTiltEase(config.dash_tilt_ease)` | `BreakerConfig::dash_tilt_ease` |
| `BrakeTilt { angle, duration, ease }` | `BreakerConfig::brake_tilt_angle/duration/ease` |
| `BrakeDecel(config.brake_decel_multiplier)` | `BreakerConfig::brake_decel_multiplier` (default: 2.0) |
| `BreakerReflectionSpread(config.reflection_spread.to_radians())` | `BreakerConfig::reflection_spread` (default: 75.0°) |
| `SettleDuration(config.settle_duration)` | `BreakerConfig::settle_duration` (default: 0.25) |
| `SettleTiltEase(config.settle_tilt_ease)` | `BreakerConfig::settle_tilt_ease` |
| `BumpPerfectWindow(config.perfect_window)` | `BreakerConfig::perfect_window` (default: 0.15) |
| `BumpEarlyWindow(config.early_window)` | `BreakerConfig::early_window` (default: 0.15) |
| `BumpLateWindow(config.late_window)` | `BreakerConfig::late_window` (default: 0.15) |
| `BumpPerfectCooldown(config.perfect_bump_cooldown)` | `BreakerConfig::perfect_bump_cooldown` (default: 0.0) |
| `BumpWeakCooldown(config.weak_bump_cooldown)` | `BreakerConfig::weak_bump_cooldown` (default: 0.15) |
| `BumpVisualParams { duration, peak, peak_fraction, rise_ease, fall_ease }` | multiple `BreakerConfig` fields |

### Layer 3: `apply_entity_scale_to_breaker` (conditional)
**File:** `breaker-game/src/breaker/systems/apply_entity_scale_to_breaker.rs:15`
**Guard:** early-returns if no `ActiveNodeLayout` resource
Inserts `EntityScale(layout.0.entity_scale)` — only present when a node layout specifies it.

### Layer 4: `init_breaker`
**File:** `breaker-game/src/breaker/systems/init_breaker/system.rs:73`
**Guard:** `Without<BreakerInitialized>` — runs once per breaker lifetime
**Dependencies:** reads `SelectedBreaker` + `BreakerRegistry` to look up `BreakerDefinition`

Inserts:
- `BreakerInitialized` — always inserted (marker to prevent re-init)
- `LivesCount(def.life_pool)` — only if `BreakerDefinition::life_pool` is `Some(n)`

### Layer 5: `dispatch_breaker_effects`
**File:** `breaker-game/src/breaker/systems/dispatch_breaker_effects/system.rs`
**Schedule:** `OnEnter(GameState::Playing)`, after `init_breaker`
Dispatches `BoundEffects` to target entities based on the selected breaker's `effects` vec.

---

## Hot-Reload Post-Mutation: `propagate_breaker_config` (dev feature only)
**File:** `breaker-game/src/debug/hot_reload/systems/propagate_breaker_config.rs:21`
**Schedule:** `Update`, conditioned on `resource_changed::<BreakerConfig>`
Re-inserts ALL `init_breaker_params` components unconditionally (no `Without<MaxSpeed>` guard).
Used only during hot-reload development, not in production gameplay.

---

## `BreakerConfig` and `BreakerDefinition`

### `BreakerConfig` (resource)
**File:** `breaker-game/src/breaker/resources.rs`
- `GameConfig`-derived resource, loaded from `config/defaults.breaker.ron` with per-breaker `.breaker.ron` ext overrides
- Fields map 1:1 to components inserted by `init_breaker_params` / `propagate_breaker_config`
- Default values are the code `Default` impl (also tested against the RON file)
- `stat_overrides` from `BreakerDefinition` can override: `width`, `height`, `max_speed`, `acceleration`, `deceleration`

### `BreakerDefinition` (asset)
**File:** `breaker-game/src/breaker/definition.rs`
- Loaded per-breaker from RON files in the breaker registry
- `stat_overrides: BreakerStatOverrides` — only 5 fields: `width`, `height`, `max_speed`, `acceleration`, `deceleration`
- `life_pool: Option<u32>` — determines whether `LivesCount` is inserted
- `effects: Vec<RootEffect>` — dispatched to entities via `dispatch_breaker_effects`

### `apply_stat_overrides` (dev/test only — `#[cfg(any(test, feature = "dev"))]`)
**File:** `breaker-game/src/breaker/systems/init_breaker/system.rs:24`
Applies `BreakerStatOverrides` onto a mutable `BreakerConfig`. Called by `apply_breaker_config_overrides` (also test-only) which resets config from RON defaults first, then applies overrides.

---

## Complete Component Set on a Fully-Initialized Breaker

After all 5 layers run, a breaker has:

**Via `#[require]`:**
- `Spatial2D`
- `InterpolateTransform2D`

**Via `spawn_breaker` (Layer 1):**
- `Breaker` (marker)
- `BreakerVelocity`
- `BreakerState`
- `BreakerTilt`
- `BumpState`
- `BreakerStateTimer`
- `GameDrawLayer::Breaker`
- `Position2D`
- `PreviousPosition`
- `Scale2D`
- `PreviousScale`
- `Aabb2D`
- `CollisionLayers`
- `Mesh2d`
- `MeshMaterial2d`
- `CleanupOnRunEnd`

**Via `init_breaker_params` (Layer 2):**
- `BreakerWidth`
- `BreakerHeight`
- `BreakerBaseY`
- `MaxSpeed`
- `BreakerAcceleration`
- `BreakerDeceleration`
- `DecelEasing`
- `DashSpeedMultiplier`
- `DashDuration`
- `DashTilt`
- `DashTiltEase`
- `BrakeTilt`
- `BrakeDecel`
- `BreakerReflectionSpread`
- `SettleDuration`
- `SettleTiltEase`
- `BumpPerfectWindow`
- `BumpEarlyWindow`
- `BumpLateWindow`
- `BumpPerfectCooldown`
- `BumpWeakCooldown`
- `BumpVisualParams`

**Via `apply_entity_scale_to_breaker` (Layer 3, conditional):**
- `EntityScale` — only if `ActiveNodeLayout` resource exists

**Via `init_breaker` (Layer 4, once only):**
- `BreakerInitialized` (always)
- `LivesCount` (only if breaker def has `life_pool: Some(n)`)

**Via `dispatch_breaker_effects` (Layer 5):**
- `BoundEffects` written to breaker and other target entities

**Via `BreakerPlugin` + other domains (post-spawn, runtime):**
- `BumpVisual` — inserted by `trigger_bump_visual` when bump occurs
- `ScenarioTagBreaker` — inserted by scenario runner's `tag_game_entities` on `OnEnter(Playing)`

---

## Test Spawn Sites and Helper Patterns

### Pattern 1: Full stat bundle via `breaker_param_bundle` (dash tests)
**File:** `breaker-game/src/breaker/systems/dash/tests/helpers.rs:18`

`breaker_param_bundle(config)` returns a tuple of:
`(MaxSpeed, BreakerDeceleration, DecelEasing, DashSpeedMultiplier, DashDuration, DashTilt, DashTiltEase, BrakeTilt, BrakeDecel, SettleDuration, SettleTiltEase)`

`spawn_test_breaker(app)` spawns:
```
(Breaker, BreakerState::Idle, BreakerVelocity { x: 0.0 }, BreakerTilt::default(),
 BreakerStateTimer { remaining: 0.0 }, breaker_param_bundle(&config))
```
Used in all dash system tests. Missing: `BreakerWidth`, `Position2D`, `BumpState`.

### Pattern 2: Movement bundle via `spawn_breaker_at` (move_breaker tests)
**File:** `breaker-game/src/breaker/systems/move_breaker/tests.rs:59`

```
(Breaker, state, BreakerVelocity { x: 0.0 }, MaxSpeed(config.max_speed),
 BreakerAcceleration(config.acceleration), BreakerDeceleration(config.deceleration),
 DecelEasing { … }, BreakerWidth(config.width), Position2D(position))
```
Minimal set needed to test movement: no BumpState, no tilt, no dash components.

### Pattern 3: Bump timing bundle via `bump_param_bundle` (bump tests)
**File:** `breaker-game/src/breaker/systems/bump/tests/helpers.rs:50`

`bump_param_bundle(config)` returns:
`(BumpPerfectWindow, BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpWeakCooldown, SettleDuration)`

Used in all bump system tests as:
```
(Breaker, BumpState::default(), bump_param_bundle(&config))
```
Minimal — no movement, position, or visual components.

### Pattern 4: Effect dispatch target (effect/dispatch tests)
**File:** `breaker-game/src/breaker/systems/dispatch_breaker_effects/tests/basic_dispatch.rs:31` (and many others)

```
(Breaker, BoundEffects::default())
```
Absolute minimum — just the marker and the effect storage.

### Pattern 5: Full chip dispatch target (chips tests)
**File:** `breaker-game/src/chips/systems/dispatch_chip_effects/tests/helpers.rs:109`
Function: `spawn_breaker(app)`:

```
(Breaker, BoundEffects::default(), StagedEffects::default(),
 ActiveBumpForces::default(), ActiveSizeBoosts::default(),
 ActiveDamageBoosts::default(), ActiveSpeedBoosts::default())
```
Used for chip dispatch E2E tests; includes all `Active*` effect accumulator components.

`spawn_breaker_bare(app)` (same file, line 132) — same without `BoundEffects` / `StagedEffects`:
```
(Breaker, ActiveBumpForces::default(), ActiveSizeBoosts::default(),
 ActiveDamageBoosts::default(), ActiveSpeedBoosts::default())
```

### Pattern 6: Shield test helper
**File:** `breaker-game/src/bolt/systems/bolt_lost/tests/shield_tests/helpers.rs:11`
Function: `spawn_shielded_breaker(app, pos, charges)`:

```
(Breaker, Position2D(pos), Spatial2D, GameDrawLayer::Breaker)
```
Then inserts `ShieldActive { charges }` separately. Note: manually adds `Spatial2D` even though `#[require]` would auto-insert it — harmless.

### Pattern 7: Hot-reload test helper
**File:** `breaker-game/src/debug/hot_reload/systems/propagate_breaker_config.rs:96`
Function: `spawn_breaker_with_config(world, config)`:

Full stat bundle built manually (same components as `init_breaker_params`). Used only in hot-reload propagation tests.

### Pattern 8: Bump feedback text test (local helper)
**File:** `breaker-game/src/breaker/systems/bump_feedback.rs:121`

```
(Breaker, Transform::from_xyz(0.0, -450.0, 0.0))
```
Minimal — just the marker and a legacy `Transform` to provide position for text spawning. Note: this test uses `Transform` directly (pre-spatial2d migration style), not `Position2D`.

### Pattern 9: Core `#[require]` tests (bare Breaker)
**File:** `breaker-game/src/breaker/components/core.rs:71, 84, 97, 110`

```
app.world_mut().spawn(Breaker)
```
Used only to verify `#[require]` inserts `Spatial2D`, `InterpolateTransform2D`, and does NOT insert cleanup components.

### Pattern 10: apply_entity_scale test (bare + optional EntityScale)
**File:** `breaker-game/src/breaker/systems/apply_entity_scale_to_breaker.rs:62, 87, 109`

```
app.world_mut().spawn(Breaker)
app.world_mut().spawn((Breaker, EntityScale(0.7)))
```

### Pattern 11: Scenario runner (ScenarioTagBreaker only — checker self-tests)
**File:** `breaker-scenario-runner/src/invariants/checkers/breaker_in_bounds.rs:63, 77`

```
(ScenarioTagBreaker, Position2D(Vec2::new(…)))
```
Does NOT use `Breaker` component — these tests exercise the invariant checker directly using only the scenario marker + position. The real `Breaker` component is added first by `spawn_breaker`, then tagged by `tag_game_entities`.

### Pattern 12: init_breaker tests (init-test pattern)
**File:** `breaker-game/src/breaker/systems/init_breaker/tests.rs:71, 84, 99, …`

```
(Breaker, BoundEffects::default())
```
No stats — tests only the `BreakerInitialized` marker and `LivesCount` insertion.

---

## Node Re-Entry Behavior (Breaker Persistence)

The breaker is **not** cleaned up between nodes — it has `CleanupOnRunEnd`, NOT `CleanupOnNodeExit`.

On re-entry to `OnEnter(GameState::Playing)`:
1. `spawn_breaker` — no-op (existing breaker found); still emits `BreakerSpawned`
2. `init_breaker_params` — no-op (entity has `MaxSpeed` from first node)
3. `apply_entity_scale_to_breaker` — always runs, may update `EntityScale`
4. `init_breaker` — no-op (`BreakerInitialized` already present)
5. `reset_breaker` — DOES run: resets position to center, clears velocity/tilt/state/bump

This means stat components (`BreakerWidth`, `MaxSpeed`, etc.) are stamped ONCE and persist. `reset_breaker` only resets dynamic state, not config-derived stats.

---

## Key Files

- `breaker-game/src/breaker/components/core.rs` — `Breaker` component definition with `#[require]`
- `breaker-game/src/breaker/systems/spawn_breaker/system.rs` — production spawn (Layer 1)
- `breaker-game/src/breaker/systems/init_breaker_params.rs` — stat component stamping (Layer 2)
- `breaker-game/src/breaker/systems/apply_entity_scale_to_breaker.rs` — node-scale (Layer 3)
- `breaker-game/src/breaker/systems/init_breaker/system.rs` — behavior init (Layer 4)
- `breaker-game/src/breaker/systems/dispatch_breaker_effects/system.rs` — effect dispatch (Layer 5)
- `breaker-game/src/breaker/resources.rs` — `BreakerConfig` (all 27 fields)
- `breaker-game/src/breaker/definition.rs` — `BreakerDefinition` + `BreakerStatOverrides`
- `breaker-game/src/breaker/plugin.rs` — system scheduling + ordering constraints
- `breaker-game/src/debug/hot_reload/systems/propagate_breaker_config.rs` — dev hot-reload re-stamp
- `breaker-game/src/breaker/systems/dash/tests/helpers.rs` — `breaker_param_bundle`, `spawn_test_breaker`
- `breaker-game/src/breaker/systems/bump/tests/helpers.rs` — `bump_param_bundle`
- `breaker-game/src/chips/systems/dispatch_chip_effects/tests/helpers.rs` — `spawn_breaker`, `spawn_breaker_bare`
- `breaker-game/src/bolt/systems/bolt_lost/tests/shield_tests/helpers.rs` — `spawn_shielded_breaker`
