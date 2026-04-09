# Testing Architecture

Test infrastructure conventions for Bevy ECS unit and integration tests. Read this before writing tests.

For the TDD cycle (RED → GREEN → REFACTOR), see `.claude/rules/tdd.md`. For test types and scenario coverage, see [standards.md](standards.md). This document covers **how test code is structured**, not what gets tested.

---

## The Problem With 154 `test_app()` Functions

Each system test builds a bespoke `App` registering only the system under test, its messages, and its dependencies. This is the correct Bevy testing pattern — minimal apps catch missing dependency declarations and prevent false passes from ambient state. But copy-pasting the setup across 154 locations creates:

1. **Boilerplate explosion** — every new system needs a new `test_app()` with near-identical setup.
2. **Fragile refactoring** — rename a message or add a required resource, and you touch 20+ `test_app()` functions.
3. **Silent duplication** — `tick()` (73 copies), `spawn_in_world()` (47 copies), `spawn_bolt()` (8 copies), `enter_playing()` (5 copies) are each reimplemented identically in every test suite that needs them.

The fix is **not** a single god `test_app()` — that would hide dependency bugs and slow down tests. The fix is **composable, domain-level test building blocks** that each test suite assembles into a minimal app.

---

## Domain Test Utils — `test_utils.rs`

Each domain that has tests provides a `test_utils.rs` module at the domain root. This module is the **single source of truth** for that domain's test infrastructure.

### Location and Visibility

```
src/<domain>/
├── mod.rs
├── plugin.rs
├── components.rs
├── systems/
└── test_utils.rs    # #[cfg(test)] — only compiled in test builds
```

In `mod.rs`:
```rust
#[cfg(test)]
pub(crate) mod test_utils;
```

`pub(crate)` visibility lets other domains' tests use these building blocks when they need to spawn entities from your domain. This is intentional — a collision system test needs to spawn both bolts and cells.

### What Lives in `test_utils.rs`

| Category | Examples | Visibility |
|----------|----------|------------|
| **App builders** | `bolt_test_app()`, `bolt_collision_app()` | `pub(crate)` |
| **App extensions** | `with_physics(app)`, `with_state_hierarchy(app)` | `pub(crate)` |
| **Entity spawners** | `spawn_bolt(app, pos, vel)`, `spawn_cell(app, pos)` | `pub(crate)` |
| **Definition factories** | `default_bolt_definition()`, `default_cell_definition()` | `pub(crate)` |
| **Frame advance** | `tick(app)` | `pub(crate)` |
| **Message collectors** | `DamageCellCollector`, `collect_damage_cells` | `pub(crate)` |

### What Does NOT Live in `test_utils.rs`

- **Test-specific helpers** that only one test suite needs — keep those in `tests/helpers.rs` as `pub(super)`
- **Production code** — test_utils is `#[cfg(test)]` only
- **Assertions** — each test asserts its own expectations; shared assertions hide what's being tested

---

## Shared Test Utils — `shared/test_utils.rs`

Cross-cutting helpers that don't belong to any single domain live in `shared/test_utils.rs`:

```rust
// shared/test_utils.rs

/// Advances one FixedUpdate timestep. Every test suite that runs FixedUpdate
/// systems needs this — there MUST be only one definition in the codebase.
pub(crate) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

/// Registers the full state hierarchy (AppState → GameState → RunState →
/// NodeState → ChipSelectState → RunEndState) and transitions into Playing.
/// Use when the system under test has `run_if(in_state(...))` guards.
pub(crate) fn with_state_hierarchy(app: &mut App) -> &mut App {
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<AppState>();
    app.add_sub_state::<GameState>();
    app.add_sub_state::<RunState>();
    app.add_sub_state::<NodeState>();
    app.add_sub_state::<ChipSelectState>();
    app.add_sub_state::<RunEndState>();
    app
}

/// Drives the state machine from initial states into NodeState::Playing.
/// Call after `with_state_hierarchy` when the system under test needs
/// to be in a playing state.
pub(crate) fn enter_playing(app: &mut App) {
    // transition through AppState → GameState → RunState → NodeState
    // ... (single canonical implementation)
}
```

---

## Composable App Building

Tests compose domain builders with app extensions. The pattern is:

```rust
// bolt/systems/bolt_cell_collision/tests/helpers.rs
use crate::bolt::test_utils::{bolt_collision_app, spawn_bolt, default_bolt_definition};
use crate::shared::test_utils::tick;

pub(super) fn test_app() -> App {
    bolt_collision_app()  // MinimalPlugins + physics + bolt collision system + messages
}

// Uses shared tick(), domain-specific spawn_bolt()
```

Each `test_app()` at the test-suite level becomes a thin composition call — typically 1-3 lines — instead of 10-15 lines of setup. The suite-level `test_app()` still exists (different suites may compose differently), but it delegates to shared building blocks.

### Domain App Builders

Each domain provides one or more app builders for its common test configurations:

```rust
// bolt/test_utils.rs

/// Minimal app with bolt components and the physics plugin.
/// Does NOT register any bolt systems — the caller adds what they need.
pub(crate) fn bolt_base_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(RantzPhysics2dPlugin);
    app
}

/// App for testing bolt-cell collision.
/// Registers the collision system, damage messages, and impact messages.
pub(crate) fn bolt_collision_app() -> App {
    let mut app = bolt_base_app();
    app.add_message::<BoltImpactCell>()
       .add_message::<DamageCell>()
       .add_message::<BoltImpactWall>()
       .add_systems(
           FixedUpdate,
           bolt_cell_collision
               .after(PhysicsSystems::MaintainQuadtree),
       );
    app
}
```

**Rule: app builders register systems; they don't spawn entities.** Entity spawning is the test's job (via spawner helpers).

### App Extension Functions

For cross-cutting concerns that multiple domains need:

```rust
/// Registers the physics plugin. Call from domain app builders that need
/// collision queries or quadtree access.
pub(crate) fn with_physics(app: &mut App) -> &mut App {
    app.add_plugins(RantzPhysics2dPlugin);
    app
}
```

Extension functions take `&mut App` and return `&mut App` for chaining. They add capabilities, not systems — the domain app builder decides which systems to register.

---

## Entity Spawners

Each domain provides spawner functions for its entity types. These use the domain's builder (if one exists) and return the `Entity` for assertions:

```rust
// bolt/test_utils.rs

/// Standard bolt definition matching values previously provided by BoltConfig::default().
pub(crate) fn default_bolt_definition() -> BoltDefinition {
    BoltDefinition {
        name: "Bolt".to_string(),
        base_speed: 400.0,
        min_speed: 200.0,
        max_speed: 800.0,
        radius: 8.0,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    }
}

/// Spawns a bolt at the given position with the given velocity.
pub(crate) fn spawn_bolt(app: &mut App, x: f32, y: f32, vx: f32, vy: f32) -> Entity {
    let def = default_bolt_definition();
    // ... uses Bolt::builder() ...
}
```

**Consistency rule**: all spawners in `test_utils.rs` use the same default values. When a domain has multiple spawner variants, the base version uses defaults and named variants document what's different:

```rust
pub(crate) fn spawn_bolt_with_damage(app: &mut App, x: f32, y: f32, damage: f32) -> Entity { ... }
pub(crate) fn spawn_bolt_heavy(app: &mut App, x: f32, y: f32) -> Entity { ... }
```

---

## Message Collectors

When tests need to assert that a message was sent, the standard pattern is a collector resource:

```rust
// In the domain's test_utils.rs (if multiple test suites need it)
// or in tests/helpers.rs (if only one suite needs it)

#[derive(Resource, Default)]
pub(crate) struct DamageCellCollector(pub(crate) Vec<DamageCell>);

pub(crate) fn collect_damage_cells(
    mut reader: MessageReader<DamageCell>,
    mut collector: ResMut<DamageCellCollector>,
) {
    for msg in reader.read() {
        collector.0.push(msg.clone());
    }
}

/// Registers the DamageCell message and a collector system.
pub(crate) fn with_damage_cell_collector(app: &mut App) -> &mut App {
    app.add_message::<DamageCell>()
       .init_resource::<DamageCellCollector>()
       .add_systems(Update, collect_damage_cells);
    app
}
```

---

## What This Replaces

The old pattern (still found in the codebase, being migrated):

```rust
// bolt/systems/bolt_lost/tests/helpers.rs — private to this one test suite
pub(super) fn test_app() -> App { /* 8 lines of setup */ }
pub(super) fn tick(app: &mut App) { /* 4 lines — identical to 72 other copies */ }
pub(super) fn spawn_bolt(app: &mut App, ...) -> Entity { /* 15 lines */ }
pub(super) fn make_default_bolt_definition() -> BoltDefinition { /* 14 lines */ }
```

The new pattern:

```rust
// bolt/systems/bolt_lost/tests/helpers.rs — thin wrappers
use crate::bolt::test_utils::{bolt_base_app, spawn_bolt};
use crate::shared::test_utils::tick;

pub(super) fn test_app() -> App {
    let mut app = bolt_base_app();
    app.init_resource::<PlayfieldConfig>()
       .init_resource::<GameRng>()
       .add_message::<BoltLost>()
       .add_systems(FixedUpdate, bolt_lost);
    app
}
// tick() and spawn_bolt() — imported, not reimplemented
```

The suite-level `helpers.rs` still exists for suite-specific setup, but it delegates shared work to `test_utils`. The `tick()`, `spawn_bolt()`, `enter_playing()`, and `spawn_in_world()` duplicates are eliminated entirely.

---

## Rules

1. **One `tick()` in the codebase.** It lives in `shared/test_utils.rs`. All 73 copies get replaced with imports.
2. **One `enter_playing()` in the codebase.** It lives in `shared/test_utils.rs`. All 5 copies get replaced.
3. **One `default_bolt_definition()` per domain.** It lives in `<domain>/test_utils.rs`. Duplicates within the domain get replaced.
4. **One `spawn_<entity>()` per domain.** It lives in `<domain>/test_utils.rs`. Suite-specific variants in `tests/helpers.rs` can wrap it with additional setup.
5. **App builders don't spawn entities.** Spawning is the test's responsibility.
6. **App extensions return `&mut App`.** For chaining: `with_physics(with_state_hierarchy(&mut app))`.
7. **Suite-level `test_app()` still exists.** It composes domain builders and extensions to create the minimal app for that specific suite. It's just shorter now.
8. **`pub(crate)` for test_utils, `pub(super)` for suite helpers.** Cross-domain tests can use another domain's `test_utils`. Suite helpers are private to the suite.
9. **No test_utils in `crate::prelude`.** Import explicitly: `use crate::bolt::test_utils::spawn_bolt`.
10. **New code follows this convention immediately.** Legacy code migrates incrementally (see migration todo).

---

## File Index

After migration, the test infrastructure files are:

```
src/shared/test_utils.rs          # tick(), with_state_hierarchy(), enter_playing()
src/bolt/test_utils.rs            # bolt_base_app(), bolt_collision_app(), spawn_bolt(), default_bolt_definition()
src/breaker/test_utils.rs         # breaker_base_app(), spawn_breaker(), default_breaker_definition()
src/cells/test_utils.rs           # cells_base_app(), spawn_cell(), default_cell_definition()
src/effect/test_utils.rs          # effect_base_app(), with_effect_dispatch(), spawn_in_world()
src/chips/test_utils.rs           # chips_base_app(), default_chip_definition()
src/walls/test_utils.rs           # walls_base_app(), spawn_wall(), default_wall_definition()
src/state/test_utils.rs           # state_base_app() (if needed)
```

Each domain decides what to expose based on what other domains' tests actually need.
