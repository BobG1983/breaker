# Test Infrastructure — Shortcuts & Conventions

## Builder Compound Methods

The builder already bundles common registration groups as compound methods:

| Method | Registers |
|--------|-----------|
| `.with_playfield()` | `PlayfieldConfig` + `CellConfig` + `Assets<Mesh>` + `Assets<ColorMaterial>` |
| `.with_state_hierarchy()` | `StatesPlugin` + `AppState` + 5 sub-states |
| `.with_message_capture::<M>()` | message + `MessageCollector<M>` + collector system |
| `.with_*_registry_entry()` | registry (if needed) + entry |
| `.in_state_node_playing()` | 4 state transitions + 4 updates |

These cover the most common capability blocks found across 101 `test_app()` implementations.

---

## Domain Test Utils — Entity Spawners & Definitions

Each domain's `test_utils.rs` provides entity spawners and default definitions. These are NOT on the builder — they operate on `&mut App` or `&mut World` after building.

### bolt/test_utils.rs

```rust
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

/// Uses World::commands() + World::flush() internally — callers never see it.
pub(crate) fn spawn_bolt(app: &mut App, x: f32, y: f32, vx: f32, vy: f32) -> Entity {
    let def = default_bolt_definition();
    let world = app.world_mut();
    let entity = Bolt::builder()
        .at_position(Vec2::new(x, y))
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(vx, vy)))
        .primary()
        .headless()
        .spawn(&mut world.commands());
    world.flush();
    entity
}

pub(crate) fn spawn_bolt_with_damage(app: &mut App, x: f32, y: f32, damage: f32) -> Entity { ... }
```

### breaker/test_utils.rs

```rust
pub(crate) fn default_breaker_definition() -> BreakerDefinition { ... }
pub(crate) fn spawn_breaker(app: &mut App, x: f32, y: f32) -> Entity { ... }
```

### cells/test_utils.rs

```rust
pub(crate) fn default_cell_definition() -> CellTypeDefinition { ... }
pub(crate) fn default_cell_dims() -> (CellWidth, CellHeight) { ... }
pub(crate) fn spawn_cell(app: &mut App, x: f32, y: f32) -> Entity { ... }
pub(crate) fn spawn_cell_with_hp(app: &mut App, x: f32, y: f32, hp: f32) -> Entity { ... }
```

### walls/test_utils.rs

```rust
pub(crate) fn spawn_wall(app: &mut App, x: f32, y: f32, width: f32, height: f32) -> Entity { ... }
```

### effect/test_utils.rs

```rust
/// Inserts effect evaluation components onto an existing entity.
pub(crate) fn with_effect_components(app: &mut App, entity: Entity) {
    app.world_mut().entity_mut(entity).insert((
        BoundEffects::default(),
        StagedEffects::default(),
        ActiveDamageBoosts::default(),
        ActiveSpeedBoosts::default(),
    ));
}
```

---

## When to Extract vs. Inline

**Inline the builder** (most common case):
- Fewer than 3 tests in the file share the exact same builder config
- The builder chain is 6 lines or fewer

```rust
#[test]
fn bolt_lost_when_below_playfield() {
    let mut app = TestAppBuilder::new()
        .with_physics()
        .with_resource::<PlayfieldConfig>()
        .with_message_capture::<BoltLost>()
        .with_system(FixedUpdate, bolt_lost)
        .build();

    spawn_bolt(&mut app, 0.0, -100.0, 0.0, -400.0);
    tick(&mut app);

    let lost = app.world().resource::<MessageCollector<BoltLost>>();
    assert_eq!(lost.0.len(), 1);
}
```

**Extract to a local function** in `tests/helpers.rs`:
- 3+ tests in the same file share identical builder config
- The config has file-specific customization (custom resources, test-only capture systems)

```rust
// bolt/systems/bolt_cell_collision/tests/helpers.rs
pub(super) fn test_app() -> App {
    TestAppBuilder::new()
        .with_physics()
        .with_message_capture::<BoltImpactCell>()
        .with_message_capture::<DamageCell>()
        .with_message_capture::<BoltImpactWall>()
        .with_system(FixedUpdate, bolt_cell_collision.after(PhysicsSystems::MaintainQuadtree))
        .build()
}
```

**Extract to domain `test_utils.rs`**:
- Same builder config used across 2+ test files within the domain
- The config represents a reusable domain testing scenario

---

## Common Patterns as Builder Calls

### Collision test (bolt-cell, bolt-breaker, bolt-wall, breaker-cell, cell-wall)
```rust
TestAppBuilder::new()
    .with_physics()
    .with_message_capture::<ImpactMessage>()
    .with_message_capture::<DamageMessage>()
    .with_system(FixedUpdate, collision_system.after(PhysicsSystems::MaintainQuadtree))
    .build()
```

### Effect trigger bridge test (~20 trigger files)
```rust
TestAppBuilder::new()
    .with_message_capture::<TriggerMessage>()
    .with_system(FixedUpdate, (
        enqueue_trigger.before(bridge_trigger),
        bridge_trigger,
    ))
    .build()
```

### State-guarded system test (tracking systems, lifecycle)
```rust
TestAppBuilder::new()
    .with_state_hierarchy()
    .in_state_node_playing()
    .with_message_capture::<OutputMessage>()
    .with_system(FixedUpdate, system_under_test)
    .build()
```

### Node layout/spawning test
```rust
TestAppBuilder::new()
    .with_playfield()
    .with_message_capture::<CellsSpawned>()
    .with_cell_registry_entry("S", standard_cell_definition())
    .insert_resource(ActiveNodeLayout(layout))
    .with_system(Startup, spawn_cells_from_layout)
    .build()
```

### Setup/initialization test (registries)
```rust
TestAppBuilder::new()
    .with_playfield()
    .with_resource::<GameRng>()
    .with_bolt_registry_entry("Bolt", default_bolt_definition())
    .with_breaker_registry_entry("Aegis", make_aegis_breaker_definition())
    .insert_resource(SelectedBreaker::default())
    .with_message_capture::<BreakerSpawned>()
    .with_message_capture::<BoltSpawned>()
    .with_system(Startup, setup_run)
    .build()
```

### Builder unit test (World only — no TestAppBuilder needed)
```rust
let mut world = World::new();
let entity = Bolt::builder()
    .at_position(Vec2::ZERO)
    .definition(&default_bolt_definition())
    .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
    .primary()
    .headless()
    .spawn(&mut world.commands());
world.flush();
// assert on world.entity(entity).get::<Component>()
```

### Chip selection test (requires specific state)
```rust
TestAppBuilder::new()
    .with_state_hierarchy()
    .in_state_chip_selecting()
    .with_resource::<ChipInventory>()
    .with_resource::<ChipCatalog>()
    .with_resource::<PendingChipSelections>()
    .with_message_capture::<ChipSelected>()
    .with_system(Update, (
        send_chip_selections.before(dispatch_chip_effects),
        dispatch_chip_effects,
    ))
    .build()
```
