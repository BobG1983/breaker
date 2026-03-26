---
name: Rot2 Math Type
description: Verified Bevy 0.18.1 Rot2 — constructors, accessors, interpolation, Default, Reflect
type: reference
---

## Rot2 (verified docs.rs bevy_math 0.18.1)

Module: `bevy::math::Rot2` — in prelude via `bevy::prelude::*`

### Constructors

```rust
Rot2::radians(radians: f32) -> Rot2
Rot2::degrees(degrees: f32) -> Rot2
Rot2::turn_fraction(fraction: f32) -> Rot2  // fraction of full turn
Rot2::from_sin_cos(sin: f32, cos: f32) -> Rot2
```

### Accessors

```rust
rot.as_radians() -> f32    // in range (-π, π]
rot.as_degrees() -> f32    // in range (-180, 180]
rot.as_turn_fraction() -> f32  // in range (-0.5, 0.5]
rot.sin_cos() -> (f32, f32)
```

### Associated Constants

```rust
Rot2::IDENTITY   // no rotation (equivalent to 0°/360°)
Rot2::PI         // half-turn (180°)
Rot2::FRAC_PI_2  // 90°
Rot2::FRAC_PI_4  // 45°
Rot2::FRAC_PI_3  // 60°
Rot2::FRAC_PI_6  // 30°
Rot2::FRAC_PI_8  // 22.5°
```

### Interpolation

```rust
rot.slerp(end: Rot2, s: f32) -> Rot2   // spherical linear interp, constant angular velocity
rot.nlerp(end: Rot2, s: f32) -> Rot2   // normalized linear interp, ease-in-out effect
```

**No `lerp()` method.** Use `slerp` for constant-velocity rotation, `nlerp` for eased rotation.

### Other Methods

```rust
rot.inverse() -> Rot2
rot.normalize() -> Rot2
rot.is_normalized() -> bool
rot.angle_to(other: Rot2) -> f32
rot.length() -> f32
rot.is_finite() -> bool
rot.is_nan() -> bool
```

### Trait Implementations (verified bevy_math 0.18.1 source)

- `Default` — returns `Rot2::IDENTITY`
- `Reflect` — yes, `impl Reflect for Rot2`
- `PartialReflect` — yes
- `FromReflect` — yes
- `StableInterpolate` — yes (enables generic interpolation pipelines)
- `Mul<Rot2>`, `MulAssign<Rot2>`, `Mul<Vec2>` — rotation composition and vector rotation
- `Neg` — negation (inverse)
- `Copy`, `Clone`, `Debug`, `PartialEq`, `Serialize`, `Deserialize`

### Using Rot2 in a #[derive(Reflect)] component

Since Rot2 implements Reflect, it can be a field in a `#[derive(Reflect)]` component without special handling.

```rust
#[derive(Component, Reflect)]
struct MyRotation {
    angle: Rot2,
}
```
