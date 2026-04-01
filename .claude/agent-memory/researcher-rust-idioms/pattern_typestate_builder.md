---
name: Typestate builder for Bevy component bundles
description: How to build mutually-exclusive-method, variable-output-tuple builders for Bevy spawn sites in this project
type: project
---

## Decision

Use separate concrete structs as typestate markers (not PhantomData, not enums) when building a Bevy component bundle where:
- Some methods must be mutually exclusive at compile time
- The output tuple type varies depending on which methods were called
- The result must implement `Bundle` for direct use in `commands.spawn()`

## Pattern

State structs carry their data directly (no PhantomData needed):

```
struct NoSpeed;
struct SpeedOnly { base: f32 }
struct ClampedSpeed { base: f32, min: f32, max: f32 }
```

Builder is generic over state types:

```
struct SpatialDataBuilder<Speed, Angle> { speed: Speed, angle: Angle }
```

Methods are impl'd only on the specific state type they make sense for:
- `.with_speed()` and `.with_clamped_speed()` only on `SpatialDataBuilder<NoSpeed, A>`
- `.clamped()` only on `SpatialDataBuilder<SpeedOnly, A>`
- `.build()` only on fully-configured states (NOT on `SpatialDataBuilder<NoSpeed, _>`)

Multiple `build()` impls, each returning a distinct concrete tuple type — no trait abstraction needed (YAGNI).

## Why Not Alternatives

- Enum state: can't give method-level exclusivity
- Sealed `SpatialBundle` trait: hides concrete tuple type from caller, adds abstraction without a second use case
- Single builder with `Option` fields: loses compile-time guarantees entirely
- Bevy `RequiredComponents`: unconditional, not suitable for optional component groups

## Location

State marker structs + builder should live in `rantzsoft_spatial2d/src/components/` alongside the component types they configure. The crate has no game knowledge and the builder only uses its own types.

## Codebase Context

Researched 2026-03-31. The only existing builder in the project is `RantzDefaultsPluginBuilder` in `rantzsoft_defaults/src/plugin/definition.rs` — it uses consuming `mut self` accumulation but not typestate. No component bundle builders existed when this was researched.
