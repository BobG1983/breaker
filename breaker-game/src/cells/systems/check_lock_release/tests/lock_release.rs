//! Behaviors 2, 3, 5: lock release logic and `CellDestroyedAt` migration.

use bevy::prelude::*;

use super::helpers::*;
use crate::cells::{components::*, messages::CellDestroyedAt};

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
            was_required_to_clear: true,
        },
        CellDestroyedAt {
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
        was_required_to_clear: true,
    }];

    tick(&mut app);

    // adj_b still alive => Locked should remain.
    assert!(
        app.world().get::<Locked>(lock_cell).is_some(),
        "Locked should remain when adj_b is still alive"
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
    use crate::cells::{
        messages::CellDestroyedAt, systems::check_lock_release::check_lock_release,
    };

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
            was_required_to_clear: true,
        },
        crate::cells::messages::CellDestroyedAt {
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

    // Lock cell with empty adjacents list -- edge case.
    let lock_cell = app
        .world_mut()
        .spawn((Cell, Locked, LockAdjacents(vec![]), CellHealth::new(10.0)))
        .id();

    // No CellDestroyed messages needed -- the adjacents list is empty.
    tick(&mut app);

    // Empty adjacents vec => all adjacents are "destroyed" => Locked removed.
    assert!(
        app.world().get::<Locked>(lock_cell).is_none(),
        "Locked should be removed immediately when adjacents list is empty"
    );
}
