---
name: f32-to-u32 damage cast pattern
description: How to convert small positive f32 damage values to u32 under pedantic clippy lints in this project
type: project
---

## Decision

For `take_boosted_damage`: use f32 arithmetic directly (literal constant avoids u32→f32 cast),
then a single `#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]` on
`value.round() as u32`. Keep `#[expect]` at the statement level with a tight reason string.

## Why the f64 path exists

`BASE_BOLT_DAMAGE` is typed `u32 = 10`. Casting `u32` to `f32` triggers
`cast_precision_loss` (u32 has values f32 can't represent exactly). The f64 path was added
to route around this: `f64::from(u32)` is exact. The f64 path is correct and safe — it is
NOT over-engineered in the general case.

## When f32 is fine instead

If the damage value is written as an f32 *literal* (never cast from u32 at the call site),
`cast_precision_loss` never fires. In `bolt_cell_collision.rs` the value comes from
`BASE_BOLT_DAMAGE: u32`, so f64 is required there. In `take_boosted_damage`, the signature
already takes `base_damage: u32`, so the same constraint applies — f64 stays correct.

## Which lints fire on `f32.round() as u32`

- `cast_possible_truncation` — fires (f32 can exceed u32::MAX or be NaN/inf)
- `cast_sign_loss` — fires (f32 is signed, u32 is unsigned)
- `cast_precision_loss` — does NOT fire on float→int direction

## Minimal `#[expect]` for the two-lint case

```rust
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "value clamped to [0.0, u16::MAX] — fits u32, non-negative"
)]
let amount = value.round() as u32;
```

Two lints, one attribute. This is the floor — don't add more suppression.

## The `u16::try_from(x as i32).unwrap_or(0)` chain

This is over-engineered for this domain. It exists to avoid `cast_possible_truncation`
entirely, but the clamp + two-lint `#[expect]` is cleaner and equally sound.
The chain is acceptable to leave if the caller wants zero suppression attributes;
it is not wrong, just verbose.

## Codebase locations

- `breaker-game/src/cells/components.rs:92-105` — `take_boosted_damage` (the method in question)
- `breaker-game/src/physics/systems/bolt_cell_collision.rs:65-71` — parallel f64 path, kept because source is `u32` constant
- `breaker-game/src/shared/mod.rs:15` — `BASE_BOLT_DAMAGE: u32 = 10`
