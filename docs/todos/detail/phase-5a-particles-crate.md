# 5a: rantzsoft_particles2d Crate

## Summary

Create a standalone, game-agnostic CPU particle system crate at `rantzsoft_particles2d/`. Provides per-particle entity spawning, emitter components with multiple emission modes, a custom additive-blend `ParticleMaterial`, update/cleanup systems, a soft cap of 8192 concurrent particles, and common presets. Exposes `RantzParticles2dPlugin` with `default()` and `headless()` constructors. Zero game vocabulary. No dependency on `rantzsoft_spatial2d` or `rantzsoft_physics2d`.

## Context

The original Phase 5c planned a monolithic `rantzsoft_vfx` crate that bundled particles, post-processing, recipes, modifiers, entity shaders, and game-specific visual types into one package. The revised architecture (see `docs/todos/detail/phase-5-rethink/architecture.md`) splits this into focused crates and a game-side `visuals/` domain:

| Old plan | New plan |
|----------|----------|
| `rantzsoft_vfx` (monolithic) | `rantzsoft_particles2d` (this phase) + `rantzsoft_postprocess` (5d) + `breaker-game/src/visuals/` (5e) |
| Recipe system (RON-driven VFX sequences) | Eliminated — direct Rust functions per VFX effect |
| `AttachVisuals` god-message | Eliminated — builders attach visual components directly |
| Particle primitives named after game concepts (SparkBurst, ShardBurst, etc.) | Generic presets (RadialBurst, DirectionalBurst, ContinuousEmitter) — game maps its vocabulary to these |

The particle system is the first infrastructure crate to build. It has no dependencies on 5d (postprocess) or 5e (visuals domain) and can be built in parallel with both.

## What to Build

### 1. Crate Skeleton

Create `rantzsoft_particles2d/` at workspace root as a new workspace member. Follow the conventions established by `rantzsoft_spatial2d`:

- `Cargo.toml` — `name = "rantzsoft_particles2d"`, `edition = "2024"`, depends on `bevy` with `default-features = false, features = ["2d"]`. Workspace lints. No dependency on any other `rantzsoft_*` crate.
- `src/lib.rs` — module declarations, `#![cfg_attr(test, allow(...))]` block matching spatial2d pattern.
- `src/prelude.rs` — re-exports of public API types for consumer convenience.
- Add `"rantzsoft_particles2d"` to workspace members in root `Cargo.toml`.

### 2. Core Types

#### `Particle` Component

Per-particle data attached to each particle entity:

```
Particle {
    velocity: Vec2,
    lifetime: Timer,          // counts down from spawn, particle dies at zero
    rotation_speed: f32,      // radians/sec
    gravity: f32,             // downward acceleration (positive = down)
    initial_color: Color,     // HDR linear RGBA — values >1.0 trigger bloom
    initial_size: f32,        // starting scale
    fade_curve: FadeCurve,    // how alpha decays over lifetime
}
```

`FadeCurve` enum: `Linear`, `EaseOut`, `Hold { visible_fraction: f32 }` (stays full alpha for N% of lifetime, then fades). Default: `Linear`.

#### `ParticleEmitter` Component

Attached to an emitter entity (which has a `Transform` for world position):

```
ParticleEmitter {
    mode: EmissionMode,
    spawn_params: SpawnParams,
    active: bool,             // paused emitters skip spawning
}
```

#### `EmissionMode` Enum

Three modes covering all use cases:

| Variant | Behavior |
|---------|----------|
| `Continuous { rate: f32 }` | Spawns `rate` particles/sec while active. Accumulates fractional time. |
| `Burst { count: u32 }` | Spawns `count` particles immediately, then sets `active = false`. Emitter persists. |
| `OneShot { count: u32 }` | Spawns `count` particles immediately, then despawns the emitter entity when all spawned particles have expired. |

For `OneShot`, the emitter tracks how many of its particles are still alive. When the count reaches zero, the emitter entity is despawned.

#### `SpawnParams` Struct

Controls how each particle is initialized at spawn time. Every field uses ranges for randomization:

```
SpawnParams {
    lifetime: RangeInclusive<f32>,        // seconds
    velocity_shape: VelocityShape,
    speed: RangeInclusive<f32>,           // units/sec
    size: RangeInclusive<f32>,            // initial scale
    color: Color,                         // HDR linear RGBA
    gravity: f32,                         // downward acceleration
    rotation_speed: RangeInclusive<f32>,  // radians/sec
    fade_curve: FadeCurve,
}
```

#### `VelocityShape` Enum

Determines the initial velocity direction for spawned particles:

| Variant | Behavior |
|---------|----------|
| `Radial` | Random direction in 360 degrees from emitter center |
| `Cone { direction: Vec2, spread_angle: f32 }` | Random direction within `spread_angle` radians of `direction` |
| `Directional { direction: Vec2 }` | All particles go in the same direction (no spread) |

Speed from `SpawnParams::speed` is applied as the magnitude of the resulting velocity vector.

### 3. ParticleMaterial

Custom `Material2d` implementation with additive blending via `specialize()`:

- Override `MaterialPipeline::specialize()` to set `BlendState` with `BlendFactor::One` for `dst_factor` on color and alpha attachments. This produces additive compositing (light-on-dark) without relying on `AlphaMode2d` (which has no `Add` variant in Bevy 0.18).
- Fragment shader: sample the sprite/mesh color, multiply by particle alpha (computed from lifetime + fade curve). Output HDR color — values above 1.0 produce bloom via Bevy's built-in Bloom post-process.
- The material handles a simple quad mesh. The game can provide custom meshes for specific particle types by overriding the mesh on the spawned entity, but the default is a unit quad.
- Shader file: `rantzsoft_particles2d/assets/shaders/particle.wgsl`.

### 4. Plugin — `RantzParticles2dPlugin`

Two constructors:

| Constructor | Behavior |
|-------------|----------|
| `RantzParticles2dPlugin::default()` | Registers all systems, material, asset loading |
| `RantzParticles2dPlugin::headless()` | Registers only the logic systems (update, cleanup, emitter tick). No material, no mesh, no rendering. For scenario runner and tests. |

The plugin registers:
- `ParticleSystems` system set enum for ordering: `Emit`, `Update`, `Cleanup`
- `emit_particles` system in `Update` schedule, set `ParticleSystems::Emit`
- `update_particles` system in `Update` schedule, set `ParticleSystems::Update`, after `Emit`
- `cleanup_particles` system in `Update` schedule, set `ParticleSystems::Cleanup`, after `Update`
- `ParticleMaterial` as a Material2d asset (non-headless only)
- `ParticleCount` resource (tracks active particle count for soft cap enforcement)

### 5. Systems

#### `emit_particles`

- Query all `ParticleEmitter` entities where `active == true`.
- For `Continuous`: accumulate `dt` against rate, spawn fractional particles. If rate is 30.0 and dt is 0.033, spawn ~1 particle and carry the remainder.
- For `Burst` and `OneShot`: spawn `count` particles on the first tick after activation, then mark done (set `active = false` for Burst; begin tracking for OneShot).
- Before spawning, check `ParticleCount`. If at or above the soft cap (8192), skip spawning for this emitter this tick. Do not queue — just drop. Emitters resume spawning naturally once the count drops.
- Each spawned particle entity gets: `Particle` component (randomized from `SpawnParams`), `Transform` (at emitter position), `Mesh2d` (default quad), `MeshMaterial2d<ParticleMaterial>`.

#### `update_particles`

- Query all `Particle` entities.
- Apply gravity: `velocity.y -= gravity * dt`.
- Advance position: `transform.translation += velocity.extend(0.0) * dt`.
- Rotate: `transform.rotation = Quat::from_rotation_z(current_angle + rotation_speed * dt)`.
- Tick lifetime timer. Compute alpha from `fade_curve` and remaining lifetime fraction.
- Update material color alpha (or sprite color alpha) accordingly.

#### `cleanup_particles`

- Query all `Particle` entities whose lifetime timer has finished.
- Despawn them.
- Decrement `ParticleCount`.
- For `OneShot` emitters: if all tracked particles are dead, despawn the emitter entity.

### 6. `ParticleCount` Resource

```
ParticleCount {
    active: u32,
    soft_cap: u32,   // default 8192
}
```

- Incremented by `emit_particles` on each spawn.
- Decremented by `cleanup_particles` on each despawn.
- `emit_particles` checks `active >= soft_cap` before spawning.
- The cap is "soft" — existing particles are never killed to make room. Emitters simply skip spawning when the cap is reached.
- Publicly readable so game code or debug UI can display the count.

### 7. Common Presets

Builder functions that return configured `ParticleEmitter` + `SpawnParams` combinations. These are convenience constructors, not new types. They live in a `presets` module.

All presets use generic, game-agnostic names. The game maps its vocabulary to these presets (e.g., game's "SparkBurst" calls `RadialBurst::new()`).

#### `RadialBurst`

Builder for a radial particle burst:

```
RadialBurst::new()
    .count(24)
    .speed(200.0..=400.0)
    .lifetime(0.2..=0.5)
    .size(1.0..=3.0)
    .color(Color::WHITE)
    .gravity(200.0)
    .fade(FadeCurve::EaseOut)
    .build() -> ParticleEmitter
```

Produces a `OneShot` emitter with `VelocityShape::Radial`. Useful for impacts, explosions, bursts.

#### `DirectionalBurst`

Builder for a directional particle burst:

```
DirectionalBurst::new()
    .count(16)
    .direction(Vec2::Y)
    .spread_angle(0.3)
    .speed(150.0..=300.0)
    .lifetime(0.3..=0.6)
    .size(1.0..=2.0)
    .color(Color::WHITE)
    .gravity(0.0)
    .fade(FadeCurve::Linear)
    .build() -> ParticleEmitter
```

Produces a `OneShot` emitter with `VelocityShape::Cone`. Useful for directional sprays, trails, beam aftereffects.

#### `ContinuousEmitter`

Builder for a steady-state particle stream:

```
ContinuousEmitter::new()
    .rate(20.0)
    .speed(50.0..=100.0)
    .lifetime(1.0..=2.0)
    .size(2.0..=4.0)
    .color(Color::WHITE)
    .gravity(0.0)
    .velocity_shape(VelocityShape::Radial)
    .fade(FadeCurve::Hold { visible_fraction: 0.7 })
    .build() -> ParticleEmitter
```

Produces a `Continuous` emitter. Useful for ambient effects, auras, energy fields, persistent sources.

### 8. Tests

Unit tests for all core logic:

- **Particle lifetime**: particle with 1.0s lifetime is alive at 0.5s, dead at 1.0s.
- **Gravity application**: particle with gravity 100.0 accelerates downward correctly over multiple ticks.
- **Velocity application**: particle at (0, 0) with velocity (100, 0) moves to (100*dt, 0) after one tick.
- **Fade curves**: Linear fades from 1.0 to 0.0 proportionally. EaseOut fades faster at start. Hold stays at 1.0 for visible_fraction then fades.
- **Emission modes**: Continuous spawns correct count over time. Burst spawns count and stops. OneShot spawns count and despawns emitter after particles die.
- **Soft cap**: emitter skips spawning when ParticleCount is at cap. Resumes when count drops.
- **OneShot cleanup**: emitter entity is despawned only after all its particles expire.
- **Preset builders**: each preset produces a correctly configured ParticleEmitter with expected defaults.
- **Headless mode**: plugin in headless mode registers systems but not materials or rendering components.
- **Inactive emitter**: emitter with `active = false` does not spawn particles.

Tests use Bevy's `App` + `Update` schedule for system tests. No rendering required — all tests run headless.

## What NOT to Do

- Do NOT add game-specific particle types (SparkBurst, ShardBurst, GlowMotes, ElectricArc, TrailBurst). Those are game-side presets in `breaker-game` that use this crate's generic API. This crate provides the engine and generic presets; the game defines its vocabulary.
- Do NOT reference bolt, breaker, cell, node, bump, flux, or any game vocabulary from `docs/design/terminology/`.
- Do NOT depend on `rantzsoft_spatial2d` or `rantzsoft_physics2d`. Particles use Bevy's built-in `Transform` for positioning. The crate is fully standalone.
- Do NOT depend on `rantzsoft_postprocess`. Bloom happens via Bevy's built-in Bloom pass reading HDR emissive values — no coordination needed.
- Do NOT implement debug menu integration. The game wires `ParticleCount` to its debug UI.
- Do NOT implement mesh variety (angular shards, elongated streaks). The crate spawns default quads. The game can override `Mesh2d` on spawned entities for custom shapes.
- Do NOT implement particle-to-particle interaction, collision, or attraction. This is a simple fire-and-forget particle system.
- Do NOT implement GPU instancing or batching optimizations in this phase. Per-entity rendering is the starting point. Optimization is a separate concern if profiling shows particle rendering as a bottleneck.

## Crate Structure

```
rantzsoft_particles2d/
  Cargo.toml
  assets/
    shaders/
      particle.wgsl
  src/
    lib.rs                        // module declarations, cfg_attr test allow block
    prelude.rs                    // re-exports: Particle, ParticleEmitter, EmissionMode,
                                  //   SpawnParams, VelocityShape, FadeCurve, ParticleCount,
                                  //   ParticleMaterial, RantzParticles2dPlugin, presets::*
    components/
      mod.rs                      // pub mod definitions; #[cfg(test)] mod tests;
      definitions.rs              // Particle, ParticleEmitter, EmissionMode, SpawnParams,
                                  //   VelocityShape, FadeCurve
      tests/
        mod.rs
        fade_curve_tests.rs       // FadeCurve::compute_alpha() unit tests
        spawn_params_tests.rs     // SpawnParams randomization tests
    material/
      mod.rs                      // pub mod definition; (re-export ParticleMaterial)
      definition.rs               // ParticleMaterial impl Material2d, specialize()
    plugin/
      mod.rs                      // pub mod definition; #[cfg(test)] mod tests;
      definition.rs               // RantzParticles2dPlugin, ParticleSystems set,
                                  //   default() vs headless() constructors
      tests/
        mod.rs
        plugin_basics.rs          // headless vs default registration
        system_ordering.rs        // system set ordering verification
    presets/
      mod.rs                      // pub mod radial_burst; pub mod directional_burst;
                                  //   pub mod continuous_emitter;
      radial_burst.rs             // RadialBurst builder
      directional_burst.rs        // DirectionalBurst builder
      continuous_emitter.rs       // ContinuousEmitter builder
    resources/
      mod.rs                      // pub mod definitions;
      definitions.rs              // ParticleCount resource
    systems/
      mod.rs                      // pub(crate) mod emit; pub(crate) mod update;
                                  //   pub(crate) mod cleanup;
      emit/
        mod.rs                    // pub(crate) mod system; #[cfg(test)] mod tests;
        system.rs                 // emit_particles system
        tests/
          mod.rs
          continuous_tests.rs     // rate accumulation, fractional spawning
          burst_tests.rs          // burst spawns count and stops
          oneshot_tests.rs        // oneshot spawns count, tracks children
          soft_cap_tests.rs       // cap enforcement, resume after cap drops
      update/
        mod.rs                    // pub(crate) mod system; #[cfg(test)] mod tests;
        system.rs                 // update_particles system
        tests/
          mod.rs
          gravity_tests.rs        // gravity acceleration
          movement_tests.rs       // velocity -> position
          fade_tests.rs           // alpha decay over lifetime
          rotation_tests.rs       // rotation_speed application
      cleanup/
        mod.rs                    // pub(crate) mod system; #[cfg(test)] mod tests;
        system.rs                 // cleanup_particles system
        tests/
          mod.rs
          despawn_tests.rs        // lifetime expiry -> despawn
          oneshot_cleanup_tests.rs // emitter despawn after all particles dead
          count_tracking_tests.rs // ParticleCount decrement accuracy
```

## Dependencies

### This Crate Depends On

| Crate | Version | Features | Why |
|-------|---------|----------|-----|
| `bevy` | `0.18.1` | `default-features = false, features = ["2d"]` | ECS, Transform, Mesh2d, Material2d, Timer, asset loading |

No other dependencies. Standalone.

### Nothing Depends On This Crate Yet

`breaker-game` will add `rantzsoft_particles2d` as a dependency when game-side particle presets are implemented (5k: bump/failure VFX, 5l: combat effect VFX). The crate must be usable before that — verified via its own tests.

### Relationship to Other Crates

| Crate | Relationship |
|-------|-------------|
| `rantzsoft_spatial2d` | Independent. Particles use Bevy `Transform`, not `Position2D`. No import. |
| `rantzsoft_physics2d` | Independent. No collision, no quadtree, no physics interaction. |
| `rantzsoft_postprocess` (5d) | Independent. HDR particle colors bloom naturally via Bevy's Bloom — no coordination. |
| `rantzsoft_defaults` | Not used. Particle config is code-side (SpawnParams), not RON-driven. |

## Verification

- Crate compiles as a workspace member (`cargo particles2dcheck` alias to add).
- All unit tests pass (`cargo particles2dtest` alias to add).
- Clippy clean (`cargo particles2dclippy` alias to add).
- `RantzParticles2dPlugin::default()` registers material, all three systems, and ParticleCount resource.
- `RantzParticles2dPlugin::headless()` registers systems and ParticleCount but not material or rendering components.
- `Continuous` emitter spawns particles at the configured rate over time.
- `Burst` emitter spawns exact count and stops.
- `OneShot` emitter spawns exact count, then emitter entity despawns after all particles expire.
- `ParticleCount` accurately tracks active particles and emitters respect the soft cap.
- Particles move according to velocity, accelerate under gravity, rotate at configured speed.
- Particle alpha fades correctly per the configured `FadeCurve`.
- `ParticleMaterial` uses additive blending (verified visually in a manual test or by inspecting the specialize output).
- Each preset builder produces a correctly configured `ParticleEmitter`.
- Add cargo aliases to `.cargo/config.toml`: `particles2dcheck`, `particles2dclippy`, `particles2dtest`.
- `cargo all-dtest` and `cargo all-dclippy` updated to include the new crate.

## NEEDS DETAIL — API Design

These questions must be answered before implementation:

- What does a consumer's code look like to spawn a particle burst? (exact API call)
- What does a continuous emitter look like from a gameplay system?
- Which types need to be `pub` vs `pub(crate)`?
- What can be enforced at compile time? (e.g., `EmissionMode::OneShot` can't have `rate` — should these be separate types or an enum?)
- What goes in `app.add_plugins(RantzParticles2dPlugin)` vs runtime spawning?
- Should particle presets return a bundle, a component tuple, or take `&mut Commands`?
- What's the headless story — does `headless()` skip `ParticleMaterial` creation entirely, or stub it? Do particles still spawn as entities (for test verification) or are they no-ops?
- How does the soft cap work — per-emitter or global? Does the emitter know it's being throttled?
- Trait bounds on the plugin — does it take a generic for draw layer ordering (like spatial2d does)?

## Status
`[NEEDS DETAIL]` — API design questions above must be resolved
