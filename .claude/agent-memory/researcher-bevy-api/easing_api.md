---
name: Bevy 0.18.1 Easing API
description: Verified EaseFunction variants, EasingCurve, Ease trait, serde support, and curve composition for bevy_math 0.18.1
type: reference
---

## Module path

`bevy::math::curve::easing` (re-exported from `bevy_math::curve::easing`)
All items are in `bevy::prelude::*` via `bevy_math`.

## EaseFunction enum — 39 variants (verified docs.rs 0.18.1)

Variants:
- Linear
- QuadraticIn, QuadraticOut, QuadraticInOut
- CubicIn, CubicOut, CubicInOut
- QuarticIn, QuarticOut, QuarticInOut
- QuinticIn, QuinticOut, QuinticInOut
- SmoothStepIn, SmoothStepOut, SmoothStep
- SmootherStepIn, SmootherStepOut, SmootherStep
- SineIn, SineOut, SineInOut
- CircularIn, CircularOut, CircularInOut
- ExponentialIn, ExponentialOut, ExponentialInOut
- ElasticIn, ElasticOut, ElasticInOut
- BackIn, BackOut, BackInOut
- BounceIn, BounceOut, BounceInOut
- Steps(usize, JumpAt)       — discrete step function
- Elastic(f32)               — parametric elastic (spring constant)

## EaseFunction traits

- Clone, Copy, Debug, PartialEq — all derived
- **Serialize, Deserialize<'de>** — YES, serde support exists
- Reflect, PartialReflect, FromReflect, Enum, GetTypeRegistration, TypePath, Typed
- **Curve<f32>** — EaseFunction itself IS a Curve<f32>, usable directly

## EaseFunction as Curve<f32>

Because EaseFunction implements Curve<f32>, it has:
- `ease_fn.sample_clamped(t: f32) -> f32`  — what the project already uses
- `ease_fn.sample(t: f32) -> Option<f32>`
- `ease_fn.sample_unchecked(t: f32) -> f32`
- `ease_fn.domain() -> Interval`           — returns Interval::UNIT ([0.0, 1.0])

## EasingCurve<T> — full animation curve

```rust
pub fn EasingCurve::new(start: T, end: T, ease_fn: EaseFunction) -> EasingCurve<T>
```
- Parametrized over unit interval [0, 1]
- Implements `Curve<T>` where `T: Ease + Clone`
- `.sample_clamped(t)` returns T directly (interpolated value)
- Bound: `T` must implement `Ease`

## Ease trait

Required method:
```rust
fn interpolating_curve_unbounded(start: Self, end: Self) -> impl Curve<Self>
```

Implementors (verified):
- All types implementing `VectorSpace<Scalar = f32>` — includes Vec2, Vec3, f32, etc.
- Rot2, Quat, Dir2, Dir3, Dir3A
- Isometry2d, Isometry3d
- Tuples of Ease types (up to 11 elements)

## No built-in power curve variant

There is NO `Power(f32)` variant in EaseFunction. For power curves (t.powf(n)), you must
hand-roll OR use Quintic/Quartic/Cubic as fixed approximations. Alternatively, use the
low-level `EaseFunction` as a Curve<f32> and then call `.map()` on it.

## Serde / RON config support

EaseFunction derives Serialize + Deserialize. Combined with the project's "serialize" feature
flag in Cargo.toml, EaseFunction CAN be stored in RON config files. RON encoding:
- Unit variants: `QuadraticOut`, `CubicIn`, etc.
- Tuple variants: `Steps(4, End)`, `Elastic(2.5)`

JumpAt enum (for Steps): None, Start, End, Both — also serializable.

## Curve composition (verified)

The Curve trait has adaptor methods (iterator-style API):
- `.chain(other_curve)` — sequence two curves end-to-end; combined domain extends
- `.zip(other_curve)` — merge into Curve<(A, B)>
- `.map(f)` — transform output type; returns new Curve<U>
- `.reparametrize(interval, f)` — remap parameter space
- `.reparametrize_linear(new_interval)` — linear domain remap
- `.reverse()` — traverse backward
- `.repeat(n)` — loop n times
- `.ping_pong()` — oscillate forward/backward
- `.by_ref()` — borrow without consuming

These live on `CurveExt` extension trait (blanket-impl'd on all Curve<T>).

## Practical: compose a custom S-curve

```rust
use bevy::math::curve::easing::EaseFunction;
// Chain QuadraticIn for first half, QuadraticOut for second half manually:
// Just use SmoothStep — it IS QuadraticInOut (3t² - 2t³)
let s_curve_val = EaseFunction::SmoothStep.sample_clamped(t);

// Or use map() to apply power on top of an easing function:
let powered = EaseFunction::QuadraticOut.map(|v| v.powf(0.5));
// powered: impl Curve<f32>, call powered.sample_clamped(t)
```

## Warning: Elastic(f32) vs ElasticIn/ElasticOut

- `EaseFunction::Elastic(f32)` is a SEPARATE parametric elastic (pass spring constant)
- `EaseFunction::ElasticIn`, `ElasticOut`, `ElasticInOut` are fixed-parameter variants
- `ElasticCurve` struct also exists as a standalone struct with its own constructor

## Sources

- https://docs.rs/bevy_math/0.18.1/bevy_math/curve/easing/enum.EaseFunction.html
- https://docs.rs/bevy_math/0.18.1/bevy_math/curve/easing/struct.EasingCurve.html
- https://docs.rs/bevy_math/0.18.1/bevy_math/curve/easing/trait.Ease.html
- https://docs.rs/bevy_math/0.18.1/bevy_math/curve/index.html
