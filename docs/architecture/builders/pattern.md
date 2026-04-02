# Typestate Builder Pattern

## What It Is

A typestate builder uses Rust's type system to prevent invalid entity construction at **compile time**. Each "dimension" of configuration (position, speed, role, etc.) is a generic type parameter on the builder struct. The parameter starts as an "unconfigured" marker type and transitions to a "configured" marker type when the corresponding method is called. Terminal methods like `build()` and `spawn()` are only available when **every** dimension is in a configured state.

If you forget to call `.movement(...)` on a breaker builder, the code doesn't compile. Not a runtime panic, not a test failure — a compiler error. The invalid state is literally unrepresentable in the type system.

## Why It Works

Entity construction in an ECS is error-prone. A breaker needs ~40 components to be valid. Without a builder, you manually assemble tuple bundles — miss one component and the entity silently malfunctions. Tests might catch it, but often don't because the test helper assembles a different subset than production code.

The typestate builder solves this by:
1. **Compile-time completeness** — you can't build an entity without providing all required data.
2. **Single source of truth** — one `build()` function produces the complete component set. No scattered init systems.
3. **Convenience shortcuts** — `.definition(&BreakerDefinition)` fills many dimensions at once from a definition struct.
4. **Override flexibility** — `.with_max_speed(600.0)` overrides a single value after config.
5. **Test ergonomics** — tests use the same builder as production. No divergent test helpers.

## How It Works

### Marker Types

Each dimension has marker structs:

```rust
pub struct NoDimensions;                        // unconfigured
pub struct HasDimensions { w: f32, h: f32, y: f32 }  // configured
```

### Generic Builder

The builder carries one type parameter per dimension:

```rust
pub struct BreakerBuilder<D, Mv, Da, Sp, Bm, V, R> {
    dimensions: D,
    movement: Mv,
    dashing: Da,
    spread: Sp,
    bump: Bm,
    visual: V,
    role: R,
    optional: OptionalData,
}
```

### Transition Methods

Each method transitions one dimension, consuming the builder and returning a new one:

```rust
impl<Mv, Da, Sp, Bm, V, R> BreakerBuilder<NoDimensions, Mv, Da, Sp, Bm, V, R> {
    pub fn dimensions(self, w: f32, h: f32, y: f32)
        -> BreakerBuilder<HasDimensions, Mv, Da, Sp, Bm, V, R> { ... }
}
```

### Convenience Methods

A definition shortcut transitions multiple dimensions at once:

```rust
impl<V, R> BreakerBuilder<NoDimensions, NoMovement, NoDashing, NoSpread, NoBump, V, R> {
    pub fn definition(self, def: &BreakerDefinition)
        -> BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, V, R> { ... }
}
```

### Terminal Methods

`build()` is only available when all dimensions are satisfied:

```rust
impl BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, Headless, Primary> {
    pub fn build(self) -> impl Bundle { ... }
}
```

One `impl` block per valid combination of exclusive states (e.g., `Rendered` vs `Headless`).

### Optional Fields

Things that don't affect entity validity (lives count, effects, color overrides) live in an `OptionalData` struct with `Option<T>` fields. Optional methods are available at any typestate via a blanket impl.

### Output Paths

| Method | Input | Returns | Use case |
|--------|-------|---------|----------|
| `build()` | nothing | `impl Bundle` | Tests (headless), deferred spawning |
| `spawn()` | `&mut Commands` | `Entity` | Production — spawns entity + queues effect dispatch |

`build()` returns **every component** needed for a valid entity. `spawn()` calls `build()`, spawns via commands, then queues `dispatch_initial_effects` (no entity parameter — resolves targets from world by convention).

## Mutually Exclusive States

Some dimensions have multiple configured states that are mutually exclusive:

| Dimension | States | Meaning |
|-----------|--------|---------|
| Motion (Bolt) | `Serving` / `HasVelocity` | Serving bolt is stationary; launched bolt has velocity |
| Role (Breaker + Bolt) | `Primary` / `Extra` | Primary = persists across nodes; Extra = cleaned up on node exit |
| Visual (Breaker + Bolt) | `Rendered` / `Headless` | Rendered includes mesh/material; Headless omits them |

Each exclusive pair shares the same dimension but produces different terminal impl blocks. With two binary exclusive dimensions (Visual × Role), the breaker builder has **4 terminal `impl` blocks**: Rendered+Primary, Rendered+Extra, Headless+Primary, Headless+Extra.

## Conventions

- Entry point: `Marker::builder()` (e.g., `Bolt::builder()`, `Breaker::builder()`)
- Marker structs are `pub` (they appear in return types) with private fields
- Builder struct is `pub`, module is `pub(crate)`
- `build()` returns `impl Bundle` — test via `World::spawn()` + `world.get()`
- Settings structs group related config values (e.g., `MovementSettings`, `DashSettings`)
- Rendering is outside the builder for `headless()` — only `rendered()` includes mesh/material
- `.definition()` (not `.config()`) is the production convenience shortcut — reads from `BreakerDefinition` or `BoltDefinition`, not a config resource

## Current Implementations

| Entity | Builder | Dimensions | Location |
|--------|---------|-----------|----------|
| **Bolt** | `Bolt::builder()` | P, S, A, M, R, V | `breaker-game/src/bolt/builder/` |
| **Breaker** | `Breaker::builder()` | D, Mv, Da, Sp, Bm, V, R | `breaker-game/src/breaker/builder/` |
| **Spatial** | `Spatial::builder()` | Position, Speed, Angle | `rantzsoft_spatial2d/src/builder.rs` |
