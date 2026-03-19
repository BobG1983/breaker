---
name: numeric_cast_patterns
description: Established project pattern for f64→u32 conversion under strict clippy pedantic/cast lints, verified from cells/components.rs
type: reference
---

# Numeric Cast Patterns (Bevy 0.18.1, strict clippy pedantic)

## The Problem

With `cast_possible_truncation`, `cast_sign_loss`, `cast_precision_loss` all denied, there is
**no `as` cast from f32/f64 to any integer type that passes without a lint attribute**.

Rust std provides no safe checked float→int conversion. `to_int_unchecked()` is unsafe.
`TryFrom` is not implemented for float→integer in std.
Bevy 0.18 provides no numeric conversion utilities for this.

## Established Project Pattern (from `cells/components.rs`)

The pattern for `f64 → u32` when the domain value is bounded (e.g., damage 10–300):

```rust
// Step 1: compute in f64 (widening, lossless)
let d = f64::from(base_damage) * (1.0_f64 + f64::from(boost));

// Step 2: clamp to a range that fits safely in i32 (u16::MAX is sufficient for game values)
let clamped = d.round().clamp(0.0, f64::from(u16::MAX));

// Step 3: cast to i32 — now safe because clamped is in [0.0, 65535.0]
// #[expect] is required and is the accepted approach in this codebase
#[expect(
    clippy::cast_possible_truncation,
    reason = "clamped to [0.0, 65535.0] — always fits in i32"
)]
let amount = u32::from(u16::try_from(clamped as i32).unwrap_or(0));
```

**Why this works:**
- `f64::from(u32)` and `f64::from(f32)` are exact widening — no precision loss lints.
- `.clamp(0.0, f64::from(u16::MAX))` ensures sign safety AND eliminates truncation risk
  for the i32 cast (0–65535 always fits in i32 without loss).
- `cast_possible_truncation` on `clamped as i32` is the ONE remaining lint — suppressed with
  `#[expect(..., reason = "...")]` because the clamp proves it is safe.
- `u16::try_from(i32)` is infallible for [0, 65535] but `.unwrap_or(0)` satisfies the
  `unwrap_used = "warn"` lint by using a safe fallback.
- `u32::from(u16)` is lossless widening — no lint.

## Key Insight: `#[expect]` is the accepted approach

- `#[allow]` without reason is denied by `allow_attributes_without_reason`.
- `#[expect(..., reason = "...")]` IS the project's accepted mechanism for unavoidable casts.
- The reason must explain WHY the cast is safe (the clamp invariant), not just acknowledge the lint.
- `clippy::cast_sign_loss` and `clippy::cast_precision_loss` are avoided by design (clamping
  and using f64); only `cast_possible_truncation` needs suppression.

## Int → Float Pattern (for completeness)

```rust
// u32 → f32: go through u16 to guarantee no precision loss lint
f32::from(u16::try_from(val).unwrap_or(u16::MAX))
```

This avoids `cast_precision_loss` because u16 → f32 is exactly representable (u16 max = 65535,
within f32's 24-bit mantissa).

## Bevy does not help here

`bevy::math` provides no float→int conversion utilities. The pattern is pure Rust std.
