---
name: Transform Interpolation (FixedUpdate visual smoothing)
description: Bevy 0.18.1 has NO built-in transform interpolation. Manual pattern and third-party crate options.
type: reference
---

## Verified: Bevy 0.18.1 has NO built-in transform interpolation

Searched: `bevy::transform` module (only `TransformPlugin`, `TransformSystems`, `StaticTransformOptimizations`, `TransformPoint`), migration guide (0.17→0.18), and `bevy_transform` crate source tree. Nothing named TransformInterpolation, TransformSmoothing, or similar exists.

## Official Manual Pattern (from `examples/movement/physics_in_fixed_timestep.rs`)

Two custom components + a PostUpdate system reading `overstep_fraction()`:

```rust
// Custom components — one per physics entity
#[derive(Component)] struct PhysicalTranslation(Vec3);
#[derive(Component)] struct PreviousPhysicalTranslation(Vec3);

// In FixedUpdate: save previous, advance current
fn advance_physics(
    mut query: Query<(&mut PreviousPhysicalTranslation, &mut PhysicalTranslation, &Velocity)>,
    time: Res<Time<Fixed>>,
) {
    for (mut prev, mut curr, vel) in &mut query {
        prev.0 = curr.0;
        curr.0 += vel.0 * time.delta_secs();
    }
}

// In AfterFixedMainLoop (or PostUpdate): blend into visual Transform
fn interpolate_rendered_transform(
    mut query: Query<(&mut Transform, &PreviousPhysicalTranslation, &PhysicalTranslation)>,
    time: Res<Time<Fixed>>,
) {
    let alpha = time.overstep_fraction();
    for (mut transform, prev, curr) in &mut query {
        transform.translation = prev.0.lerp(curr.0, alpha);
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
