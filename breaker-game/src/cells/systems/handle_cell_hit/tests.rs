use bevy::prelude::*;

use super::*;
use crate::cells::{
    components::*,
    messages::{DamageCell, RequestCellDestroyed},
};

#[derive(Resource)]
struct TestMessage(Option<DamageCell>);

#[derive(Resource, Default)]
struct TestMessages(Vec<DamageCell>);

#[derive(Resource, Default)]
struct CapturedDestroyed(Vec<RequestCellDestroyed>);

fn enqueue_from_resource(msg_res: Res<TestMessage>, mut writer: MessageWriter<DamageCell>) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

fn enqueue_all(msg_res: Res<TestMessages>, mut writer: MessageWriter<DamageCell>) {
    for msg in &msg_res.0 {
        writer.write(msg.clone());
    }
}

fn capture_destroyed(
    mut reader: MessageReader<RequestCellDestroyed>,
    mut captured: ResMut<CapturedDestroyed>,
) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_message::<DamageCell>()
        .add_message::<RequestCellDestroyed>()
        .add_systems(FixedUpdate, handle_cell_hit);
    app
}

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

fn default_damage_visuals() -> CellDamageVisuals {
    CellDamageVisuals {
        hdr_base: 4.0,
        green_min: 0.2,
        blue_range: 0.4,
        blue_base: 0.2,
    }
}

fn spawn_cell(app: &mut App, hp: f32) -> Entity {
    let material = app
        .world_mut()
        .resource_mut::<Assets<ColorMaterial>>()
        .add(ColorMaterial::from_color(Color::srgb(4.0, 0.2, 0.5)));
    let mesh = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Rectangle::new(1.0, 1.0));
    app.world_mut()
        .spawn((
            Cell,
            CellHealth::new(hp),
            default_damage_visuals(),
            RequiredToClear,
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id()
}

fn spawn_optional_cell(app: &mut App, hp: f32, required: bool) -> Entity {
    let material = app
        .world_mut()
        .resource_mut::<Assets<ColorMaterial>>()
        .add(ColorMaterial::from_color(Color::srgb(4.0, 0.2, 0.5)));
    let mesh = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Rectangle::new(1.0, 1.0));
    let mut entity = app.world_mut().spawn((
        Cell,
        CellHealth::new(hp),
        default_damage_visuals(),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    if required {
        entity.insert(RequiredToClear);
    }
    entity.id()
}

fn spawn_locked_cell(app: &mut App, hp: f32) -> Entity {
    let material = app
        .world_mut()
        .resource_mut::<Assets<ColorMaterial>>()
        .add(ColorMaterial::from_color(Color::srgb(4.0, 0.2, 0.5)));
    let mesh = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Rectangle::new(1.0, 1.0));
    app.world_mut()
        .spawn((
            Cell,
            Locked,
            CellHealth::new(hp),
            default_damage_visuals(),
            RequiredToClear,
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id()
}

// --- Behavior 1: DamageCell sends RequestCellDestroyed at exact HP ---

#[test]
fn damage_cell_10_destroys_10hp_cell() {
    let mut app = test_app();
    let cell = spawn_cell(&mut app, 10.0);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_chip: None,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_from_resource.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    // Two-phase destruction: entity stays alive, RequestCellDestroyed sent
    let captured = app.world().resource::<CapturedDestroyed>();
    assert_eq!(
        captured.0.len(),
        1,
        "exactly one RequestCellDestroyed expected"
    );
    assert_eq!(
        captured.0[0].cell, cell,
        "RequestCellDestroyed should carry the destroyed cell entity"
    );
}

#[test]
fn damage_cell_overkill_15_on_10hp_cell_destroys() {
    let mut app = test_app();
    let cell = spawn_cell(&mut app, 10.0);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 15.0,
        source_chip: None,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_from_resource.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    // Two-phase destruction: entity stays alive, RequestCellDestroyed sent
    let captured = app.world().resource::<CapturedDestroyed>();
    assert_eq!(
        captured.0.len(),
        1,
        "exactly one RequestCellDestroyed expected"
    );
}

// --- Behavior 2: DamageCell leaves cell alive with reduced HP ---

#[test]
fn damage_cell_10_on_30hp_cell_leaves_20hp() {
    let mut app = test_app();
    let cell = spawn_cell(&mut app, 30.0);

    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_chip: None,
    })));
    app.add_systems(FixedUpdate, enqueue_from_resource.before(handle_cell_hit));
    tick(&mut app);

    assert!(
        app.world().get_entity(cell).is_ok(),
        "30-HP cell should survive 10 damage"
    );
    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 20.0).abs() < f32::EPSILON,
        "30.0-HP cell after 10 damage should have 20.0 HP, got {}",
        health.current
    );
}

// --- Behavior 3: Partial damage with non-base amount ---

#[test]
fn damage_cell_15_on_20hp_cell_leaves_5hp() {
    let mut app = test_app();
    let cell = spawn_cell(&mut app, 20.0);

    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 15.0,
        source_chip: None,
    })));
    app.add_systems(FixedUpdate, enqueue_from_resource.before(handle_cell_hit));
    tick(&mut app);

    assert!(
        app.world().get_entity(cell).is_ok(),
        "20-HP cell should survive 15 damage"
    );
    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 5.0).abs() < f32::EPSILON,
        "20.0-HP cell after 15 damage should have 5.0 HP, got {}",
        health.current
    );
}

// --- Behavior 4: Zero damage does nothing ---

#[test]
fn damage_cell_zero_does_not_change_health() {
    let mut app = test_app();
    let cell = spawn_cell(&mut app, 10.0);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 0.0,
        source_chip: None,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_from_resource.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    assert!(
        app.world().get_entity(cell).is_ok(),
        "zero damage should not destroy cell"
    );
    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "zero damage should leave HP unchanged at 10.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert_eq!(
        captured.0.len(),
        0,
        "zero damage should not send RequestCellDestroyed"
    );
}

// --- Behavior 5: Locked cell ignores damage ---

#[test]
fn locked_cell_ignores_damage_cell() {
    let mut app = test_app();
    let cell = spawn_locked_cell(&mut app, 10.0);

    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_chip: None,
    })));
    app.add_systems(FixedUpdate, enqueue_from_resource.before(handle_cell_hit));
    tick(&mut app);

    assert!(
        app.world().get_entity(cell).is_ok(),
        "locked cell should not be despawned"
    );
    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "locked cell HP should remain 10.0, got {}",
        health.current
    );
}

// --- Behavior 6: was_required_to_clear false for non-required cell ---

#[test]
fn destroyed_non_required_cell_sends_request_cell_destroyed() {
    let mut app = test_app();
    let cell = spawn_optional_cell(&mut app, 10.0, false);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_chip: None,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_from_resource.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    // Two-phase destruction: entity stays alive, RequestCellDestroyed sent
    let captured = app.world().resource::<CapturedDestroyed>();
    assert_eq!(
        captured.0.len(),
        1,
        "exactly one RequestCellDestroyed expected"
    );
    assert_eq!(
        captured.0[0].cell, cell,
        "RequestCellDestroyed should carry the destroyed cell entity"
    );
}

// --- Behavior 7: Dedup — two DamageCell same cell, only one RequestCellDestroyed ---

#[test]
fn double_damage_cell_same_cell_only_one_request_cell_destroyed() {
    let mut app = test_app();
    let cell = spawn_optional_cell(&mut app, 10.0, true);

    app.init_resource::<CapturedDestroyed>();
    app.init_resource::<TestMessages>();
    app.world_mut().resource_mut::<TestMessages>().0 = vec![
        DamageCell {
            cell,
            damage: 10.0,
            source_chip: None,
        },
        DamageCell {
            cell,
            damage: 10.0,
            source_chip: None,
        },
    ];
    app.add_systems(
        FixedUpdate,
        (
            enqueue_all.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    // Two-phase destruction: entity stays alive, only one RequestCellDestroyed
    let captured = app.world().resource::<CapturedDestroyed>();
    assert_eq!(
        captured.0.len(),
        1,
        "two DamageCell on same 10-HP cell should produce exactly one RequestCellDestroyed"
    );
}

// =========================================================================
// C7 Wave 2a: Two-Phase Destruction — handle_cell_hit writes
// RequestCellDestroyed instead of despawning (behaviors 29, 32)
// =========================================================================

#[derive(Resource, Default)]
struct CapturedRequestCellDestroyed(Vec<crate::cells::messages::RequestCellDestroyed>);

fn capture_request_cell_destroyed(
    mut reader: MessageReader<crate::cells::messages::RequestCellDestroyed>,
    mut captured: ResMut<CapturedRequestCellDestroyed>,
) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}

fn test_app_two_phase() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_message::<DamageCell>()
        .add_message::<RequestCellDestroyed>()
        .add_systems(FixedUpdate, handle_cell_hit);
    app
}

#[test]
fn handle_cell_hit_writes_request_cell_destroyed_instead_of_despawning() {
    let mut app = test_app_two_phase();
    let cell = spawn_cell(&mut app, 10.0);

    app.init_resource::<CapturedRequestCellDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_chip: None,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_from_resource.before(handle_cell_hit),
            capture_request_cell_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    let captured = app.world().resource::<CapturedRequestCellDestroyed>();
    assert_eq!(
        captured.0.len(),
        1,
        "handle_cell_hit should write RequestCellDestroyed when cell HP reaches 0"
    );
    assert_eq!(
        captured.0[0].cell, cell,
        "RequestCellDestroyed should carry the cell entity"
    );

    // Entity should STILL BE ALIVE (no immediate despawn in two-phase flow)
    assert!(
        app.world().get_entity(cell).is_ok(),
        "cell entity should still be alive — bridge evaluates before cleanup despawns"
    );
}

#[test]
fn handle_cell_hit_dedup_produces_one_request_cell_destroyed() {
    let mut app = test_app_two_phase();
    let cell = spawn_optional_cell(&mut app, 10.0, true);

    app.init_resource::<CapturedRequestCellDestroyed>();
    app.init_resource::<TestMessages>();
    app.world_mut().resource_mut::<TestMessages>().0 = vec![
        DamageCell {
            cell,
            damage: 10.0,
            source_chip: None,
        },
        DamageCell {
            cell,
            damage: 10.0,
            source_chip: None,
        },
    ];
    app.add_systems(
        FixedUpdate,
        (
            enqueue_all.before(handle_cell_hit),
            capture_request_cell_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    let captured = app.world().resource::<CapturedRequestCellDestroyed>();
    assert_eq!(
        captured.0.len(),
        1,
        "dedup should produce exactly one RequestCellDestroyed for same cell hit twice"
    );
}

#[test]
fn handle_cell_hit_non_required_cell_produces_request_cell_destroyed() {
    let mut app = test_app_two_phase();
    let cell = spawn_optional_cell(&mut app, 10.0, false);

    app.init_resource::<CapturedRequestCellDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_chip: None,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_from_resource.before(handle_cell_hit),
            capture_request_cell_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    let captured = app.world().resource::<CapturedRequestCellDestroyed>();
    assert_eq!(
        captured.0.len(),
        1,
        "non-required cell should also produce RequestCellDestroyed"
    );
}

// --- Behavior 8: Double DamageCell on high-HP cell decrements twice ---

#[test]
fn double_damage_cell_on_30hp_cell_decrements_twice() {
    let mut app = test_app();
    let cell = spawn_cell(&mut app, 30.0);

    app.init_resource::<TestMessages>();
    app.world_mut().resource_mut::<TestMessages>().0 = vec![
        DamageCell {
            cell,
            damage: 10.0,
            source_chip: None,
        },
        DamageCell {
            cell,
            damage: 10.0,
            source_chip: None,
        },
    ];
    app.add_systems(FixedUpdate, enqueue_all.before(handle_cell_hit));
    tick(&mut app);

    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "two 10-damage hits on 30-HP cell should leave 10.0 HP, got {}",
        health.current
    );
}

// --- Behavior 9: Two DamageCell on different cells with different damage ---

#[test]
fn two_damage_cell_different_cells_different_damage() {
    let mut app = test_app();
    let cell_a = spawn_cell(&mut app, 30.0);
    let cell_b = spawn_cell(&mut app, 30.0);

    app.init_resource::<TestMessages>();
    app.world_mut().resource_mut::<TestMessages>().0 = vec![
        DamageCell {
            cell: cell_a,
            damage: 20.0,
            source_chip: None,
        },
        DamageCell {
            cell: cell_b,
            damage: 10.0,
            source_chip: None,
        },
    ];
    app.add_systems(FixedUpdate, enqueue_all.before(handle_cell_hit));
    tick(&mut app);

    let health_a = app.world().get::<CellHealth>(cell_a).unwrap();
    assert!(
        (health_a.current - 10.0).abs() < f32::EPSILON,
        "cell A: 30.0 - 20.0 = 10.0 HP, got {}",
        health_a.current
    );

    let health_b = app.world().get::<CellHealth>(cell_b).unwrap();
    assert!(
        (health_b.current - 20.0).abs() < f32::EPSILON,
        "cell B: 30.0 - 10.0 = 20.0 HP, got {}",
        health_b.current
    );
}

// --- Behavior 10: DamageCell for despawned entity is silently skipped ---

#[test]
fn damage_cell_for_despawned_entity_is_silently_skipped() {
    let mut app = test_app();
    let cell = spawn_cell(&mut app, 10.0);

    // Despawn the cell before tick — simulate stale entity reference
    app.world_mut().despawn(cell);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_chip: None,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_from_resource.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    let captured = app.world().resource::<CapturedDestroyed>();
    assert_eq!(
        captured.0.len(),
        0,
        "DamageCell for despawned entity should not produce RequestCellDestroyed"
    );
}
