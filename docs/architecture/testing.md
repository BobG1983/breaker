# Testing Architecture

Test infrastructure conventions for Bevy ECS unit and integration tests. Read this before writing tests.

For the TDD cycle (RED → GREEN → REFACTOR), see `.claude/rules/tdd.md`. For test types and scenario coverage, see [standards.md](standards.md). This document covers **how test code is structured**, not what gets tested.

---

## The Problem With 154 `test_app()` Functions

Each system test builds a bespoke `App` registering only the system under test, its messages, and its dependencies. This is the correct Bevy testing pattern — minimal apps catch missing dependency declarations and prevent false passes from ambient state. But copy-pasting the setup across 154 locations creates:

1. **Boilerplate explosion** — every new system needs a new `test_app()` with near-identical setup.
2. **Fragile refactoring** — rename a message or add a required resource, and you touch 20+ `test_app()` functions.
3. **Silent duplication** — `tick()` (73 copies), `spawn_in_world()` (47 copies), `spawn_bolt()` (8 copies), `enter_playing()` (5 copies) are each reimplemented identically in every test suite that needs them.

The fix is **not** a single god `test_app()` — that would hide dependency bugs and slow down tests. The fix is `TestAppBuilder`: a composable, typestate builder that each test suite assembles into a minimal app, plus domain-level spawner modules.

---

## `TestAppBuilder` — `shared/test_utils/`

The central test infrastructure lives in `src/shared/test_utils/` (a directory module):

```
src/shared/test_utils/
├── mod.rs          # Re-exports: TestAppBuilder, MessageCollector, tick
├── builder.rs      # TestAppBuilder typestate builder
├── collector.rs    # MessageCollector<M>, clear_messages, collect_messages
├── tick_helper.rs  # tick()
└── tests/          # Self-tests for the infrastructure
```

Declared in `src/shared/mod.rs` as `#[cfg(test)] pub(crate) mod test_utils;`.

### `TestAppBuilder`

A typestate builder tracking whether the state hierarchy has been registered. One typestate dimension: `NoStates` → `WithStates`. This prevents the `in_state_node_playing()` panic (calling it without state hierarchy) at compile time.

```rust
use crate::shared::test_utils::TestAppBuilder;

let mut app = TestAppBuilder::new()            // MinimalPlugins registered
    .with_physics()                            // adds RantzPhysics2dPlugin
    .with_state_hierarchy()                    // registers full state hierarchy → returns TestAppBuilder<WithStates>
    .in_state_node_playing()                   // drives into NodeState::Playing
    .with_playfield()                          // PlayfieldConfig, CellConfig, Assets<Mesh/ColorMaterial>
    .with_resource::<GameRng>()                // init_resource with Default
    .with_message::<BoltLost>()               // registers message type
    .with_message_capture::<DamageCell>()      // registers + collector resource + collect system
    .with_system(FixedUpdate, bolt_lost)
    .build();
```

#### Available methods

| Method | Available on | Effect |
|--------|-------------|--------|
| `new()` | — | Creates builder with `MinimalPlugins` |
| `with_state_hierarchy()` | `NoStates` | Adds `StatesPlugin` + all sub-states → transitions to `WithStates` |
| `in_state_node_playing()` | `WithStates` | Drives to `NodeState::Playing` via four `NextState` + `app.update()` steps |
| `in_state_chip_selecting()` | `WithStates` | Drives to `ChipSelectState::Selecting` via four steps |
| `with_physics()` | any | Adds `RantzPhysics2dPlugin` |
| `with_playfield()` | any | `PlayfieldConfig`, `CellConfig`, `Assets<Mesh>`, `Assets<ColorMaterial>` |
| `with_resource::<R>()` | any | `init_resource::<R>()` (idempotent) |
| `insert_resource(val)` | any | Inserts a concrete resource value |
| `with_message::<M>()` | any | `add_message::<M>()` |
| `with_message_capture::<M>()` | any | Message + `MessageCollector<M>` + collect systems (idempotent) |
| `with_bolt_registry()` | any | Empty `BoltRegistry` |
| `with_bolt_registry_entry(name, def)` | any | Inserts a bolt definition |
| `with_breaker_registry()` | any | Empty `BreakerRegistry` |
| `with_breaker_registry_entry(name, def)` | any | Inserts a breaker definition |
| `with_cell_registry()` | any | Empty `CellTypeRegistry` |
| `with_cell_registry_entry(alias, def)` | any | Inserts a cell type definition |
| `with_system(schedule, system)` | any | `add_systems(schedule, system)` |
| `build()` | any | Returns `App` |

### `tick()`

```rust
use crate::shared::test_utils::tick;

tick(&mut app);   // advances exactly one FixedUpdate timestep
```

Lives in `shared/test_utils/tick_helper.rs`. Single definition in the codebase — the 73 duplicate copies are replaced with imports.

### `MessageCollector<M>`

Generic message collector for test assertions. Registered via `.with_message_capture::<M>()` — no per-message collector struct needed.

```rust
// Assert after tick:
let msgs = &app.world().resource::<MessageCollector<DamageCell>>().0;
assert_eq!(msgs.len(), 1);
```

`MessageCollector<M>` is cleared at `First` each update cycle and populated at `Last`. Idempotent — calling `.with_message_capture::<M>()` twice for the same type is safe.

---

## Domain Test Utils — `<domain>/test_utils.rs`

Each domain that has tests provides a `test_utils.rs` module at the domain root. These supply spawners and definition factories — **not app builders** (app building is `TestAppBuilder`'s job).

### Location and Visibility

```
src/<domain>/
├── mod.rs
├── plugin.rs
├── components.rs
├── systems/
└── test_utils.rs    # #[cfg(test)] pub(crate) — only compiled in test builds
```

In `mod.rs`:
```rust
#[cfg(test)]
pub(crate) mod test_utils;
```

`pub(crate)` visibility lets other domains' tests use these spawners when they need to spawn entities from your domain. This is intentional — a collision system test needs to spawn both bolts and cells.

### What Lives in `test_utils.rs`

| Category | Examples | Visibility |
|----------|----------|------------|
| **Entity spawners** | `spawn_bolt(app, x, y, vx, vy)`, `spawn_cell(app, x, y)`, `spawn_wall(app, x, y, hw, hh)` | `pub(crate)` |
| **Builder-based spawners** | `spawn_left_wall(app)`, `spawn_right_wall(app)`, `spawn_ceiling_wall(app)` | `pub(crate)` |
| **Definition factories** | `default_bolt_definition()`, `default_breaker_definition()`, `test_cell_definition()` | `pub(crate)` |
| **Dimension factories** | `default_cell_dims()` | `pub(crate)` |
| **Command-queue spawners** | `spawn_cell_in_world(world, build_fn)` | `pub(crate)` |

### What Does NOT Live in `<domain>/test_utils.rs`

- **App builders** — use `TestAppBuilder` instead
- **`tick()`** — import from `crate::shared::test_utils::tick`
- **Message collectors** — use `TestAppBuilder::with_message_capture::<M>()`
- **Test-specific helpers** that only one test suite needs — keep those in `tests/helpers.rs` as `pub(super)`
- **Production code** — `test_utils` is `#[cfg(test)]` only
- **Assertions** — each test asserts its own expectations

---

## Current Domain test_utils Coverage

Domains with `test_utils.rs` as of the consolidation:

| Domain | File | Contents |
|--------|------|----------|
| `shared` | `src/shared/test_utils/` (directory) | `TestAppBuilder`, `MessageCollector<M>`, `tick()` |
| `bolt` | `src/bolt/test_utils.rs` | `default_bolt_definition()`, `spawn_bolt()` |
| `breaker` | `src/breaker/test_utils.rs` | `default_breaker_definition()`, `spawn_breaker()` |
| `cells` | `src/cells/test_utils.rs` | `default_cell_dims()`, `default_damage_visuals()`, `test_cell_definition()`, `spawn_cell()`, `spawn_cell_in_world()` |
| `walls` | `src/walls/test_utils.rs` | `spawn_wall()`, `spawn_left_wall()`, `spawn_right_wall()`, `spawn_ceiling_wall()` |

Domains without `test_utils.rs` (not yet needed or tests inline): `effect`, `chips`, `state`.

---

## Composable App Building Pattern

Tests compose `TestAppBuilder` with domain spawners:

```rust
// bolt/systems/bolt_cell_collision/tests/helpers.rs
use crate::bolt::test_utils::{spawn_bolt, default_bolt_definition};
use crate::cells::test_utils::spawn_cell;
use crate::walls::test_utils::spawn_left_wall;
use crate::shared::test_utils::{TestAppBuilder, tick};

pub(super) fn test_app() -> App {
    TestAppBuilder::new()
        .with_physics()
        .with_message::<BoltImpactCell>()
        .with_message::<DamageCell>()
        .with_message::<BoltImpactWall>()
        .with_system(FixedUpdate, bolt_cell_collision.after(PhysicsSystems::MaintainQuadtree))
        .build()
}
// tick() and spawn_bolt() / spawn_cell() — imported, not reimplemented
```

The suite-level `test_app()` still exists and still composes a minimal app for that specific suite — it's just short now (typically 4–8 lines) and delegates to `TestAppBuilder`.

---

## `spawn_in_world` Elimination

The 47 `spawn_in_world()` helper copies are **eliminated entirely** — not migrated to a shared location. Bevy 0.18 provides `World::commands()` + `World::flush()` natively:

```rust
// Old pattern (deleted):
pub(super) fn spawn_in_world(world: &mut World, ...) -> Entity { ... }

// New pattern — inline or in domain test_utils:
let entity = {
    let world = app.world_mut();
    let entity = Bolt::builder().....spawn(&mut world.commands());
    world.flush();
    entity
};
```

Domain spawners in `test_utils.rs` encapsulate this internally. Direct component spawning uses `World::spawn()` which is immediate (no flush needed).

---

## Rules

1. **One `tick()` in the codebase.** Lives in `shared/test_utils/tick_helper.rs`. Import as `use crate::shared::test_utils::tick`.
2. **One `TestAppBuilder` in the codebase.** Lives in `shared/test_utils/builder.rs`. All test app construction goes through it.
3. **No per-message collector structs.** Use `TestAppBuilder::with_message_capture::<M>()` and `MessageCollector<M>`.
4. **No domain-level app builders.** Domain `test_utils` supply spawners and definitions only. App wiring is the test suite's job via `TestAppBuilder`.
5. **`spawn_in_world` is eliminated.** Use `World::commands()` + `World::flush()` directly or via domain spawners.
6. **`pub(crate)` for test_utils, `pub(super)` for suite helpers.** Cross-domain tests can use another domain's `test_utils`. Suite helpers are private to the suite.
7. **Suite-level `test_app()` still exists.** It calls `TestAppBuilder::new()....build()`. It's just shorter now.
8. **No test_utils in `crate::prelude`.** Import explicitly: `use crate::shared::test_utils::tick`.
9. **New code follows this convention immediately.** Legacy code migrates incrementally (see migration todo).
10. **App builders don't spawn entities.** Spawning is the test's responsibility via domain spawners.
