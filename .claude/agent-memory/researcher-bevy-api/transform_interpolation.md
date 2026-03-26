---
name: Transform Interpolation (FixedUpdate visual smoothing)
description: Bevy 0.18.1 has NO built-in transform interpolation. Manual pattern and third-party crate options.
type: reference
---

## Verified: Bevy 0.18.1 has NO built-in transform interpolation

Searched: `bevy::transform` module (only `TransformPlugin`, `TransformSystems`, `StaticTransformOptimizations`, `TransformPoint`), migration guide (0.17→0.18), and `bevy_transform` crate source tree. Nothing named TransformInterpolation, TransformSmoothing, or similar exists.

## Official Manual Pattern (from `examples/movement/physics_in_fixed_timestep.rs`)

Two custom components + a PostUpdate system reading `overstep_fraction()`. Verified against Bevy 0.18.1 source in cargo registry.

**CRITICAL**: The snapshot (`prev = curr`) and the physics advance (`curr += vel * dt`) happen in ONE `FixedUpdate` system — NOT in separate systems. There is no separate `FixedFirst` / `FixedPreUpdate` snapshot step in the official example.

```rust
// Custom components — one per physics entity
#[derive(Component, Default, Deref, DerefMut)] struct PhysicalTranslation(Vec3);
#[derive(Component, Default, Deref, DerefMut)] struct PreviousPhysicalTranslation(Vec3);

// In FixedUpdate: snapshot previous THEN advance current — both in same system
fn advance_physics(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut PhysicalTranslation,
        &mut PreviousPhysicalTranslation,
        &Velocity,
    )>,
) {
    for (mut current, mut previous, velocity) in query.iter_mut() {
        previous.0 = current.0;              // snapshot first
        current.0 += velocity.0 * fixed_time.delta_secs();  // then advance
    }
}

// In RunFixedMainLoop / AfterFixedMainLoop: blend into visual Transform
fn interpolate_rendered_transform(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut Transform,
        &PhysicalTranslation,
        &PreviousPhysicalTranslation,
    )>,
) {
    for (mut transform, current, previous) in query.iter_mut() {
        let alpha = fixed_time.overstep_fraction();  // 0.0–1.0
        transform.translation = previous.0.lerp(current.0, alpha);
    }
}
```

## Schedule for the interpolation system

The official example uses `RunFixedMainLoop` + `RunFixedMainLoopSystems::AfterFixedMainLoop`:

```rust
app.add_systems(RunFixedMainLoop, interpolate_rendered_transform
    .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop));
```

`RunFixedMainLoopSystems` enum variants (verified docs.rs 0.18.1):
- `BeforeFixedMainLoop` — before fixed update logic
- `FixedMainLoop` — the fixed update loop itself
- `AfterFixedMainLoop` — after fixed update; correct slot for interpolation

`PostUpdate` also works (it runs after the entire fixed loop), but `AfterFixedMainLoop` is the canonical slot as it runs every *rendered* frame.

## `Time<Fixed>` overstep methods (verified docs.rs 0.18.1)

- `overstep() -> Duration` — raw accumulated overstep
- `overstep_fraction() -> f32` — fraction 0.0–1.0 toward next tick (USE THIS for lerp alpha)
- `overstep_fraction_f64() -> f64` — same as f32 but higher precision
- `accumulate_overstep(Duration)` — testing helper (deposits time into accumulator)
- `discard_overstep(Duration)` — removes overstep; clamped at zero

## Third-party: `bevy_transform_interpolation`

Supports Bevy `^0.18.0`. Drop-in plugin approach:

```toml
bevy_transform_interpolation = "0.3"  # check crates.io for latest compatible version
```

```rust
app.add_plugins(TransformInterpolationPlugin::default());

// Per entity: add marker components to opt in
commands.spawn((
    Transform::default(),
    TranslationInterpolation,  // or RotationInterpolation, ScaleInterpolation
));

// OR interpolate ALL entities:
// TransformInterpolationPlugin::interpolate_all()
```

**Components** (all zero-sized markers):
- `TranslationInterpolation` — smooths translation
- `RotationInterpolation` — smooths rotation
- `ScaleInterpolation` — smooths scale
- `NoTransformEasing` — opt-out marker

**Also provides**: `TransformExtrapolationPlugin`, `TransformHermiteEasingPlugin`

No special Bevy feature flags required beyond standard `bevy` dependency.

## Decision for this project

The manual pattern is sufficient for a brickbreaker — bolt and breaker only need translation smoothing. The third-party crate is worth adding if you want rotation/scale smoothing or prefer zero boilerplate.
