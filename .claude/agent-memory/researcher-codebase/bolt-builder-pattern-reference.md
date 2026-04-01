---
name: bolt-builder-pattern-reference
description: Complete structural reference for the BoltBuilder typestate pattern — for use when designing the BreakerBuilder
type: reference
---

# Bolt Builder Pattern Reference

Source: `breaker-game/src/bolt/builder/builder.rs`
Related: `rantzsoft_spatial2d/src/builder.rs` (nested builder used internally)

---

## Generic Signature

```rust
pub struct BoltBuilder<P, S, A, M, R> {
    position: P,
    speed: S,
    angle: A,
    motion: M,
    role: R,
    optional: OptionalBoltData,
}
```

Five independent typestate dimensions. All five must reach a satisfied state before `build()` or `spawn()` are available. The compiler enforces this — partially configured builders simply do not have those methods.

---

## Typestate Marker Pairs

Each dimension has an unsatisfied (`No*`) and one or more satisfied states:

| Dimension | Unsatisfied | Satisfied state(s) | Transition method(s) |
|-----------|------------|-------------------|----------------------|
| Position  | `NoPosition` | `HasPosition { pos: Vec2 }` | `.at_position(Vec2)` |
| Speed     | `NoSpeed`    | `HasSpeed { base, min, max: f32 }` | `.with_speed(base, min, max)` or `.config()` |
| Angle     | `NoAngle`    | `HasAngle { h, v: f32 }` | `.with_angle(h, v)` or `.config()` |
| Motion    | `NoMotion`   | `Serving` | `.serving()` |
|           |             | `HasVelocity { vel: Velocity2D }` | `.with_velocity(Velocity2D)` |
| Role      | `NoRole`     | `Primary` | `.primary()` |
|           |             | `Extra` | `.extra()` |

`Serving` vs `HasVelocity` are **mutually exclusive** — both satisfy `NoMotion`, but choosing one permanently forecloses the other via typestate. The compiler makes calling both on the same builder a type error.

Similarly, `Primary` vs `Extra` are mutually exclusive. A builder cannot be both.

---

## The `config()` Shortcut

```rust
impl<P, M, R> BoltBuilder<P, NoSpeed, NoAngle, M, R> {
    pub fn config(self, config: &BoltConfig) -> BoltBuilder<P, HasSpeed, HasAngle, M, R>
```

`config()` is only available when **both** Speed and Angle are unsatisfied (the impl is generic over `NoSpeed` AND `NoAngle` simultaneously). It satisfies both dimensions in one call, using `BoltConfig` values. It also stores `BoltConfigParams` in `optional` (respawn offsets, initial angle) and sets `optional.radius` from `config.radius` (only if radius not already set).

**Why this constraint matters:** You cannot call `.with_speed()` then `.config()`, because by that point Speed is already `HasSpeed`, not `NoSpeed`. The `config()` shortcut is only for the fully-unconfigured (Speed+Angle) case.

---

## Optional Methods (any typestate)

Implemented on `BoltBuilder<P, S, A, M, R>` with no bounds — available at any point in the chain:

```rust
.spawned_by(&str)                     // → Option<String> in optional
.with_lifespan(duration: f32)         // → Some(Timer) if Some
.with_radius(r: f32)                  // → overrides config-derived radius
.with_inherited_effects(&BoundEffects)
.with_effects(Vec<(String, EffectNode)>)
```

`with_radius()` after `config()` overrides the config-derived radius. `config()` uses `.or(Some(config.radius))` so a prior `.with_radius()` call also wins.

---

## Terminal Impls: Where `build()` and `spawn()` Appear

There are exactly four fully-satisfied configurations, producing four concrete impl blocks:

| Configuration | Monomorphized type |
|---|---|
| Primary + Serving | `BoltBuilder<HasPosition, HasSpeed, HasAngle, Serving, Primary>` |
| Extra + Serving   | `BoltBuilder<HasPosition, HasSpeed, HasAngle, Serving, Extra>` |
| Primary + Velocity | `BoltBuilder<HasPosition, HasSpeed, HasAngle, HasVelocity, Primary>` |
| Extra + Velocity   | `BoltBuilder<HasPosition, HasSpeed, HasAngle, HasVelocity, Extra>` |

Each of the four impls has both `build() -> impl Bundle` and `spawn(world: &mut World) -> Entity`.

**`build()` vs `spawn()`:**
- `build()` returns an `impl Bundle` for use with `commands.spawn(bundle)` in normal Bevy systems with `Commands` access.
- `spawn()` takes `&mut World` directly, for exclusive-world systems like `spawn_bolt`. It calls `world.spawn(core)` then imperatively inserts optional components, returning the `Entity`.
- `build()` can only attach the flat component tuple (core + role marker + cleanup + `BoltServing` if applicable). It cannot conditionalize insertion, so `spawn_inner` handles optionals by imperative `entity.insert()` calls.

---

## Internal Split: `build_core` + `spawn_inner`

`build_core(params, optional) -> impl Bundle` assembles the shared core tuple:
- `(Bolt, Velocity2D, GameDrawLayer::Bolt, CollisionLayers)`
- `Spatial::builder().at_position().with_clamped_speed().with_clamped_angle().build()` — the nested `SpatialDataBuilder` produces `(Spatial, BaseSpeed, MinSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, Position2D, PreviousPosition)`
- `(Scale2D, PreviousScale, Aabb2D, BoltRadius)` — all from `radius`

`spawn_inner(world, core, is_serving, is_primary, optional) -> Entity` inserts the core bundle, then conditionally inserts:
1. Role marker + cleanup: `PrimaryBolt + CleanupOnRunEnd` or `ExtraBolt + CleanupOnNodeExit`
2. `BoltServing` if `is_serving`
3. `BoltConfigParams` components if `optional.bolt_params.is_some()` (4 components: spawn/respawn offsets, angle spread, initial angle)
4. `SpawnedByEvolution(name)` if `optional.spawned_by.is_some()`
5. `BoltLifespan(Timer)` if `optional.lifespan.is_some()`
6. `BoundEffects` assembled from `with_effects` + `inherited_effects` if either is `Some`

---

## `#[require]` Interaction

`Bolt` is defined as:
```rust
#[derive(Component, Debug, Default)]
pub struct Bolt;
```
No `#[require]` on `Bolt` itself. The `Spatial` component (from the nested `SpatialDataBuilder`) carries:
```rust
// In rantzsoft_spatial2d
#[require(Spatial2D, InterpolateTransform2D)]
```
So spawning `Bolt` + the spatial bundle auto-inserts `Spatial2D` and `InterpolateTransform2D` via `Spatial`'s `#[require]`.

`Breaker` has its own `#[require]`:
```rust
#[require(Spatial2D, InterpolateTransform2D)]
pub struct Breaker;
```
The Breaker marker handles the render pipeline requirements directly, unlike Bolt which gets them via `Spatial`.

**Important:** `#[require]` inserts default values. The builder explicitly provides `Position2D`, `PreviousPosition`, `Scale2D`, `PreviousScale` via the bundle, which overrides the defaults that `Spatial2D`'s `#[require]` chain would otherwise supply.

---

## Rendering Components Are NOT in the Builder

Neither `Mesh2d` nor `MeshMaterial2d` are inserted by the builder. `spawn_bolt` adds them post-spawn:
```rust
let entity = Bolt::builder()...spawn(world);
world.entity_mut(entity).insert((Mesh2d(mesh), MeshMaterial2d(material)));
```
**Why:** Rendering is a separate concern; the builder owns physics/gameplay state only. The same pattern applies to `spawn_breaker` — rendering components are in the `commands.spawn((...))` tuple but separate from gameplay components.

---

## Entry Point Convention

```rust
impl Bolt {
    pub fn builder() -> BoltBuilder<NoPosition, NoSpeed, NoAngle, NoMotion, NoRole> { ... }
}
```
The entry point is a static method on the **marker component struct itself**. This makes usage read naturally: `Bolt::builder()`.

---

## Design Decisions: What Gets Typestate-Gated vs Optional

**Typestate-gated** (must be explicitly set before `build`/`spawn`):
- Position — every bolt entity needs an explicit spawn location
- Speed — every bolt needs speed bounds for the physics clamp system
- Angle — every bolt needs angle bounds for the physics clamp system
- Motion — serving vs velocity determines gameplay state at spawn; this must be an explicit intent
- Role — primary vs extra affects cleanup lifecycle (run vs node); getting this wrong silently breaks cleanup

**Optional / defaulted**:
- Radius — defaults to `DEFAULT_RADIUS = 8.0`; can be overridden by `config()` or `with_radius()`
- Lifespan — absent means bolt lives until despawned externally
- Effects — absent means no `BoundEffects` component
- `spawned_by` — absent means no `SpawnedByEvolution` component
- Config params (respawn offsets etc.) — absent when not using `.config()`, such as for extra bolts spawned by effects

**Why motion and role are separate typestate dimensions (not combined):**
A serving bolt is always primary in practice, but the builder allows `serving().extra()` for future flexibility. The role and motion are orthogonal concerns: motion describes initial velocity state, role describes cleanup scope.

---

## Designing a Breaker Builder: Mapping to Breaker Domain

The current `spawn_breaker` system (non-builder, uses `commands.spawn()` with manual component tuple) provides the reference for what components a BreakerBuilder would cover.

**Components currently in `spawn_breaker`:**

Core gameplay state (no config overrides):
- `Breaker` (marker — also carries `#[require(Spatial2D, InterpolateTransform2D)]`)
- `BreakerVelocity::default()`
- `BreakerState::default()`
- `BreakerTilt::default()`
- `BumpState::default()`
- `BreakerStateTimer::default()`

Spatial (from `BreakerConfig`):
- `Position2D(Vec2::new(0.0, config.y_position))`
- `PreviousPosition(Vec2::new(0.0, config.y_position))`
- `Scale2D { x: config.width, y: config.height }`
- `PreviousScale { x: config.width, y: config.height }`

Physics (from `BreakerConfig`):
- `Aabb2D::new(Vec2::ZERO, Vec2::new(config.width/2.0, config.height/2.0))`
- `CollisionLayers::new(BREAKER_LAYER, BOLT_LAYER)`

Rendering + cleanup:
- `Mesh2d(...)` — rendering, would stay outside builder
- `MeshMaterial2d(...)` — rendering, would stay outside builder
- `CleanupOnRunEnd` — should be inside builder

**Components stamped by `init_breaker`** (post-spawn):
- `BreakerInitialized`
- `LivesCount(life_pool)` — optional, from breaker definition

**Components NOT currently stamped by `spawn_breaker`** but expected by systems:
- `BreakerWidth(config.width)` — used by move_breaker
- `BreakerHeight(config.height)` — used by collision systems
- `BreakerBaseY(config.y_position)` — used by reset_breaker
- `BreakerAcceleration(config.acceleration)` — used by move_breaker
- `BreakerDeceleration(config.deceleration)` — used by move_breaker
- `DecelEasing { ease, strength }` — used by move_breaker
- `BreakerReflectionSpread(config.reflection_spread)` — used by bump system
- `GameDrawLayer::Breaker`

These are currently stamped by `init_breaker_params` (a separate system). A builder might consolidate stamping.

**Suggested typestate dimensions for a BreakerBuilder:**

| Dimension | Rationale |
|-----------|-----------|
| Position (Y) | Every breaker needs an explicit Y spawn position |
| Config | `BreakerConfig` populates width, height, speed, physics — analogous to `config()` on BoltBuilder satisfying Speed+Angle |

The Breaker has no motion typestate analog because it always spawns at rest (`BreakerVelocity::default()`, no initial velocity needed). It has no role typestate because there is always exactly one Breaker. The simpler the dimensions, the fewer terminal impls are needed.

---

## Module Structure

```
breaker-game/src/bolt/
  builder/
    mod.rs         — pub(crate) mod builder; pub use builder::*; #[cfg(test)] mod tests;
    builder.rs     — all typestate types, OptionalBoltData, BoltBuilder, all impls
    tests/
      mod.rs
      typestate_tests.rs
      config_tests.rs
      optional_methods_tests.rs
      build_tests.rs
      spawn_and_effects_tests.rs
      ordering_and_layers_tests.rs
```

The builder module is a sibling to `components/`, `systems/`, `resources/`. It is `pub(crate)` — only the game crate uses it.
