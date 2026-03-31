# Typestate Builder Pattern

Standard pattern for constructing game entities with compile-time validation of required components.

## Overview

Entity builders use Rust's typestate pattern to prevent invalid entity construction at compile time. Each required dimension (position, speed, role, etc.) is a generic type parameter that transitions from an "unconfigured" marker to a "configured" marker when the corresponding method is called. Terminal methods (`build()`, `spawn()`) are only available when all dimensions are satisfied.

## Pattern

```rust
// Typestate markers
pub struct NoSpeed;
pub struct HasSpeed { base: f32, min: f32, max: f32 }

// Builder with typestate dimensions
pub struct FooBuilder<Speed, Role> {
    speed: Speed,
    role: Role,
    optional: OptionalData,
}

// Transition: NoSpeed → HasSpeed
impl<R> FooBuilder<NoSpeed, R> {
    pub fn with_speed(self, base: f32, min: f32, max: f32) -> FooBuilder<HasSpeed, R> {
        FooBuilder {
            speed: HasSpeed { base, min, max },
            role: self.role,
            optional: self.optional,
        }
    }
}

// Terminal: only available when ALL dimensions satisfied
impl FooBuilder<HasSpeed, HasRole> {
    pub fn build(self) -> impl Bundle { ... }
    pub fn spawn(self, world: &mut World) -> Entity { ... }
}
```

## Dimensions

Dimensions are independent type parameters. Each has exactly one "unconfigured" state and one or more "configured" states. Mutually exclusive options (e.g., serving vs. has-velocity) are separate configured states on the same dimension.

| Dimension kind | Example | States |
|---|---|---|
| Required | Position | `NoPosition` → `HasPosition` |
| Required, exclusive | Motion | `NoMotion` → `Serving` or `HasVelocity` |
| Convenience | Config | `from_config()` satisfies multiple dimensions at once |

## Optional Fields

Fields that don't participate in typestate (e.g., lifespan, spawned-by attribution) live in a private `OptionalData` struct with `Option<T>` fields. Optional methods are available in any typestate via a blanket `impl<P, S, A, M, R>` block.

If an optional field can also be set by a convenience method (like `from_config()`), use `.or()` to let explicit calls take precedence:

```rust
optional.radius = optional.radius.or(Some(config.radius));
```

## Output Paths

| Method | Takes | Returns | Use case |
|---|---|---|---|
| `build()` | nothing | `impl Bundle` | Tests, deferred spawning |
| `spawn()` | `&mut World` | `Entity` | Production code, full component insertion |

`build()` returns mandatory components only. `spawn()` additionally inserts optional components and handles spawn-time concerns (e.g., effect transfer).

## Conventions

- Entry point: `impl Marker { pub fn new() -> FooBuilder<No*, No*, ...> }`
- Markers are `pub` (exposed in return types) with private fields
- Builder struct is `pub` with the module `pub(crate)`
- `build()` returns `impl Bundle` (opaque — test via `World::spawn()` + `world.get()`)
- Functions taking `&references` that return `impl Bundle` need `+ use<>` for Rust 2024 lifetime rules

## Current Implementations

- **Bolt**: `Bolt::builder()` in `breaker-game/src/bolt/builder.rs` — 5 dimensions (Position, Speed, Angle, Motion, Role)
- **Spatial**: `Spatial::builder()` in `rantzsoft_spatial2d/src/builder.rs` — 3 dimensions (Position, Speed, Angle)
