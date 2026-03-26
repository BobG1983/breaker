use bevy::prelude::*;

use super::*;
use crate::cells::{
    components::*,
    messages::{CellDestroyedAt, DamageCell},
};

// ---------------------------------------------------------------
// Test helpers — message injection for CellDestroyedAt
// ---------------------------------------------------------------

#[derive(Resource, Default)]
struct TestDestroyedMessages(Vec<CellDestroyedAt>);

fn enqueue_destroyed(
    msg_res: Res<TestDestroyedMessages>,
    mut writer: MessageWriter<CellDestroyedAt>,
) {
    for msg in &msg_res.0 {
        writer.write(msg.clone());
    }
}

// ---------------------------------------------------------------
// Test helpers — message injection for DamageCell
// ---------------------------------------------------------------

#[derive(Resource)]
struct TestDamageCellMessage(Option<DamageCell>);

fn enqueue_damage_cell(msg_res: Res<TestDamageCellMessage>, mut writer: MessageWriter<DamageCell>) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

// ---------------------------------------------------------------
// Test app factories
// ---------------------------------------------------------------

/// App for testing `check_lock_release` (behaviors 2, 3, 5).
fn lock_release_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellDestroyedAt>()
        .init_resource::<TestDestroyedMessages>()
        .add_systems(
            FixedUpdate,
            (
                enqueue_destroyed.before(check_lock_release),
                check_lock_release,
            ),
        );
    app
}

/// App for testing `handle_cell_hit` with lock interaction (behaviors 1, 4).
fn hit_app() -> App {
    use crate::cells::{messages::RequestCellDestroyed, systems::handle_cell_hit};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_message::<DamageCell>()
        .add_message::<RequestCellDestroyed>()
        .add_systems(
            FixedUpdate,
            (enqueue_damage_cell.before(handle_cell_hit), handle_cell_hit),
        );
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

/// Spawns a cell with `Locked` marker and visuals for the hit-system tests.
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

/// Spawns an unlocked cell with visuals for the hit-system tests.
fn spawn_unlocked_cell(app: &mut App, hp: f32) -> Entity {
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

// ---------------------------------------------------------------
// Behavior 1: Locked cell immune to bolt damage
// ---------------------------------------------------------------

#[test]
fn locked_cell_hp_unchanged_after_damage_cell() {
    let mut app = hit_app();
    let cell = spawn_locked_cell(&mut app, 10.0);

    app.insert_resource(TestDamageCellMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_bolt: None,
        source_chip: None,
    })));
    tick(&mut app);

    // Locked cell should still exist (not destroyed)
    assert!(
        app.world().get_entity(cell).is_ok(),
        "locked cell should not be despawned by DamageCell"
    );
    // HP should be untouched
    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "locked cell HP should remain 10.0, got {}",
        health.current
    );
}

// ---------------------------------------------------------------
// Behavior 2: Lock releases when all adjacents destroyed
// ---------------------------------------------------------------

#[test]
fn lock_releases_when_all_adjacents_destroyed() {
    let mut app = lock_release_app();

    // Spawn two adjacent cell entities (they just need to exist, then be destroyed).
    let adj_a = app.world_mut().spawn_empty().id();
    let adj_b = app.world_mut().spawn_empty().id();

    // Spawn the lock cell with Locked + LockAdjacents pointing at the two adjacents.
    let lock_cell = app
        .world_mut()
        .spawn((
            Cell,
            Locked,
            LockAdjacents(vec![adj_a, adj_b]),
            CellHealth::new(10.0),
        ))
        .id();

    // Despawn the adjacent entities (simulating their destruction).
    app.world_mut().despawn(adj_a);
    app.world_mut().despawn(adj_b);

    // Send CellDestroyedAt messages for both adjacents.
    app.world_mut().resource_mut::<TestDestroyedMessages>().0 = vec![
        CellDestroyedAt {
            position: Vec2::new(10.0, 20.0),
            was_required_to_clear: true,
        },
        CellDestroyedAt {
            position: Vec2::new(30.0, 40.0),
            was_required_to_clear: true,
        },
    ];

    tick(&mut app);

    // Locked component should be removed from the lock cell.
    assert!(
        app.world().get::<Locked>(lock_cell).is_none(),
        "Locked should be removed when all adjacents are destroyed"
    );
}

// ---------------------------------------------------------------
// Behavior 3: Lock stays locked when only some adjacents destroyed
// ---------------------------------------------------------------

#[test]
fn lock_stays_locked_when_only_some_adjacents_destroyed() {
    let mut app = lock_release_app();

    // Two adjacent cells; only one will be destroyed.
    let adj_a = app.world_mut().spawn_empty().id();
    let adj_b = app.world_mut().spawn_empty().id();

    let lock_cell = app
        .world_mut()
        .spawn((
            Cell,
            Locked,
            LockAdjacents(vec![adj_a, adj_b]),
            CellHealth::new(10.0),
        ))
        .id();

    // Despawn only adj_a.
    app.world_mut().despawn(adj_a);

    // Send CellDestroyedAt only for adj_a.
    app.world_mut().resource_mut::<TestDestroyedMessages>().0 = vec![CellDestroyedAt {
        position: Vec2::new(10.0, 20.0),
        was_required_to_clear: true,
    }];

    tick(&mut app);

    // adj_b still alive => Locked should remain.
    assert!(
        app.world().get::<Locked>(lock_cell).is_some(),
        "Locked should remain when adj_b is still alive"
    );
}

// ---------------------------------------------------------------
// Behavior 4: Unlocked cell takes normal damage
// ---------------------------------------------------------------

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

#[test]
fn unlocked_cell_takes_damage_and_sends_request_destroyed() {
    use crate::cells::systems::handle_cell_hit;

    let mut app = hit_app();
    let cell = spawn_unlocked_cell(&mut app, 10.0);

    app.init_resource::<CapturedRequestCellDestroyed>();
    app.insert_resource(TestDamageCellMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_bolt: None,
        source_chip: None,
    })));
    app.add_systems(
        FixedUpdate,
        capture_request_cell_destroyed.after(handle_cell_hit),
    );
    tick(&mut app);

    // Two-phase destruction: entity stays alive, RequestCellDestroyed sent
    let captured = app.world().resource::<CapturedRequestCellDestroyed>();
    assert_eq!(
        captured.0.len(),
        1,
        "unlocked 10-HP cell should produce RequestCellDestroyed from 10.0 damage"
    );
    assert_eq!(
        captured.0[0].cell, cell,
        "RequestCellDestroyed should carry the destroyed cell entity"
    );
}

// =========================================================================
// C7 Wave 2a: CellDestroyed -> CellDestroyedAt migration (behavior 32e)
// =========================================================================

#[derive(Resource, Default)]
struct TestCellDestroyedAtMessages(Vec<crate::cells::messages::CellDestroyedAt>);

fn enqueue_cell_destroyed_at(
    msg_res: Res<TestCellDestroyedAtMessages>,
    mut writer: MessageWriter<crate::cells::messages::CellDestroyedAt>,
) {
    for msg in &msg_res.0 {
        writer.write(msg.clone());
    }
}

fn lock_release_app_cell_destroyed_at() -> App {
    use crate::cells::messages::CellDestroyedAt;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellDestroyedAt>()
        .init_resource::<TestCellDestroyedAtMessages>()
        .add_systems(
            FixedUpdate,
            (
                enqueue_cell_destroyed_at.before(check_lock_release),
                check_lock_release,
            ),
        );
    app
}

#[test]
fn check_lock_release_reads_cell_destroyed_at() {
    let mut app = lock_release_app_cell_destroyed_at();

    let adj_a = app.world_mut().spawn_empty().id();
    let adj_b = app.world_mut().spawn_empty().id();

    let lock_cell = app
        .world_mut()
        .spawn((
            Cell,
            Locked,
            LockAdjacents(vec![adj_a, adj_b]),
            CellHealth::new(10.0),
        ))
        .id();

    // Despawn adjacents (simulating cleanup_destroyed_cells)
    app.world_mut().despawn(adj_a);
    app.world_mut().despawn(adj_b);

    // Send CellDestroyedAt messages
    app.world_mut()
        .resource_mut::<TestCellDestroyedAtMessages>()
        .0 = vec![
        crate::cells::messages::CellDestroyedAt {
            position: Vec2::new(10.0, 20.0),
            was_required_to_clear: true,
        },
        crate::cells::messages::CellDestroyedAt {
            position: Vec2::new(30.0, 40.0),
            was_required_to_clear: true,
        },
    ];

    tick(&mut app);

    assert!(
        app.world().get::<Locked>(lock_cell).is_none(),
        "Locked should be removed when reading CellDestroyedAt and all adjacents are gone"
    );
}

// ---------------------------------------------------------------
// Behavior 5: Lock cell with empty adjacents unlocks immediately
// ---------------------------------------------------------------

#[test]
fn lock_cell_with_empty_adjacents_unlocks_immediately() {
    let mut app = lock_release_app();

    // Lock cell with empty adjacents list — edge case.
    let lock_cell = app
        .world_mut()
        .spawn((Cell, Locked, LockAdjacents(vec![]), CellHealth::new(10.0)))
        .id();

    // No CellDestroyed messages needed — the adjacents list is empty.
    tick(&mut app);

    // Empty adjacents vec => all adjacents are "destroyed" => Locked removed.
    assert!(
        app.world().get::<Locked>(lock_cell).is_none(),
        "Locked should be removed immediately when adjacents list is empty"
    );
}
