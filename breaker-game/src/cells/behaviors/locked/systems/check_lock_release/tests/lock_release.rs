//! Behaviors 9, 9b, 10: lock release logic and `Destroyed<Cell>` migration.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::helpers::*;
use crate::{cells::components::*, prelude::*, shared::death_pipeline::invulnerable::Invulnerable};

fn make_destroyed_msg() -> Destroyed<Cell> {
    Destroyed::<Cell> {
        victim:     Entity::PLACEHOLDER,
        killer:     None,
        victim_pos: Vec2::ZERO,
        killer_pos: None,
        _marker:    PhantomData,
    }
}

// ---------------------------------------------------------------
// Behavior 9: Lock releases when all adjacents destroyed (despawn branch)
// ---------------------------------------------------------------

#[test]
fn lock_releases_when_all_adjacents_destroyed() {
    let mut app = lock_release_app();

    // Spawn two adjacent cell entities (they just need to exist, then be despawned).
    let adj_a = app.world_mut().spawn_empty().id();
    let adj_b = app.world_mut().spawn_empty().id();

    // Spawn the lock cell with LockCell + Locked + Invulnerable + Locks + Hp + KilledBy.
    let lock_cell = app
        .world_mut()
        .spawn((
            Cell,
            LockCell,
            Locked,
            Invulnerable,
            Locks(vec![adj_a, adj_b]),
            Hp::new(10.0),
            KilledBy::default(),
        ))
        .id();

    // Despawn the adjacent entities (simulating their destruction).
    app.world_mut().despawn(adj_a);
    app.world_mut().despawn(adj_b);

    // Send Destroyed<Cell> messages for both adjacents (victim field irrelevant — the
    // system only counts messages).
    app.world_mut().resource_mut::<TestDestroyedMessages>().0 =
        vec![make_destroyed_msg(), make_destroyed_msg()];

    tick(&mut app);

    // Locked component should be removed from the lock cell.
    assert!(
        app.world().get::<Locked>(lock_cell).is_none(),
        "Locked should be removed when all adjacents are destroyed"
    );
    // Unlocked component should be inserted.
    assert!(
        app.world().get::<Unlocked>(lock_cell).is_some(),
        "Unlocked should be inserted when lock releases"
    );
    // Invulnerable should be removed alongside Locked (load-bearing — proves
    // the Locked ↔ Invulnerable coupling runs end-to-end).
    assert!(
        app.world().get::<Invulnerable>(lock_cell).is_none(),
        "Invulnerable should be removed when Locked is removed"
    );
}

// ---------------------------------------------------------------
// Behavior 9 edge: Lock stays locked when only some adjacents destroyed
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
            LockCell,
            Locked,
            Invulnerable,
            Locks(vec![adj_a, adj_b]),
            Hp::new(10.0),
            KilledBy::default(),
        ))
        .id();

    // Despawn only adj_a.
    app.world_mut().despawn(adj_a);

    // Send Destroyed<Cell> only for adj_a.
    app.world_mut().resource_mut::<TestDestroyedMessages>().0 = vec![make_destroyed_msg()];

    tick(&mut app);

    // adj_b still alive => Locked should remain.
    assert!(
        app.world().get::<Locked>(lock_cell).is_some(),
        "Locked should remain when adj_b is still alive"
    );
    // Unlocked should NOT be present.
    assert!(
        app.world().get::<Unlocked>(lock_cell).is_none(),
        "Unlocked should not be present when lock has not released"
    );
    // Invulnerable should still be present because Locked was not removed.
    assert!(
        app.world().get::<Invulnerable>(lock_cell).is_some(),
        "Invulnerable should remain while Locked is present"
    );
}

// ---------------------------------------------------------------
// Behavior 9b: Lock releases when all adjacents are alive-but-Dead-marked
// (same-tick Has<Dead> branch)
// ---------------------------------------------------------------

#[test]
fn lock_releases_when_all_adjacents_marked_dead_same_tick() {
    let mut app = lock_release_app();

    // Spawn two adjacent cells as real alive entities.
    let adj_a = app
        .world_mut()
        .spawn((Cell, Hp::new(1.0), KilledBy::default()))
        .id();
    let adj_b = app
        .world_mut()
        .spawn((Cell, Hp::new(1.0), KilledBy::default()))
        .id();

    // Mark both adjacents `Dead` synchronously (NOT via commands — the insert
    // must land before the tick runs, mirroring the same-tick case where
    // `handle_kill::<Cell>` has already inserted `Dead` but
    // `process_despawn_requests` has not yet run).
    app.world_mut().entity_mut(adj_a).insert(Dead);
    app.world_mut().entity_mut(adj_b).insert(Dead);

    let lock_cell = app
        .world_mut()
        .spawn((
            Cell,
            LockCell,
            Locked,
            Invulnerable,
            Locks(vec![adj_a, adj_b]),
            Hp::new(10.0),
            KilledBy::default(),
        ))
        .id();

    // Enqueue two Destroyed<Cell> messages (victim fields arbitrary — system counts).
    app.world_mut().resource_mut::<TestDestroyedMessages>().0 =
        vec![make_destroyed_msg(), make_destroyed_msg()];

    tick(&mut app);

    assert!(
        app.world().get::<Locked>(lock_cell).is_none(),
        "Locked should be removed when adjacents are Dead-marked in the same tick"
    );
    assert!(
        app.world().get::<Unlocked>(lock_cell).is_some(),
        "Unlocked should be inserted"
    );
    assert!(
        app.world().get::<Invulnerable>(lock_cell).is_none(),
        "Invulnerable should be removed alongside Locked"
    );

    // Adjacents should still be alive (not despawned by check_lock_release).
    assert!(
        app.world().get_entity(adj_a).is_ok(),
        "adj_a should still be alive (Dead-marked but not despawned)"
    );
    assert!(
        app.world().get_entity(adj_b).is_ok(),
        "adj_b should still be alive"
    );

    // Adjacents should still carry `Dead` — check_lock_release must not remove it.
    assert!(
        app.world().get::<Dead>(adj_a).is_some(),
        "adj_a should still have Dead"
    );
    assert!(
        app.world().get::<Dead>(adj_b).is_some(),
        "adj_b should still have Dead"
    );
}

/// Behavior 9b edge: mixed — one adjacent despawned, the other Dead-marked. Both
/// count toward destruction; lock cell unlocks.
#[test]
fn lock_releases_when_one_adjacent_despawned_and_one_dead_marked() {
    let mut app = lock_release_app();

    let adj_a = app.world_mut().spawn_empty().id();
    let adj_b = app
        .world_mut()
        .spawn((Cell, Hp::new(1.0), KilledBy::default()))
        .id();

    // adj_a despawned; adj_b Dead-marked.
    app.world_mut().despawn(adj_a);
    app.world_mut().entity_mut(adj_b).insert(Dead);

    let lock_cell = app
        .world_mut()
        .spawn((
            Cell,
            LockCell,
            Locked,
            Invulnerable,
            Locks(vec![adj_a, adj_b]),
            Hp::new(10.0),
            KilledBy::default(),
        ))
        .id();

    app.world_mut().resource_mut::<TestDestroyedMessages>().0 =
        vec![make_destroyed_msg(), make_destroyed_msg()];

    tick(&mut app);

    assert!(
        app.world().get::<Locked>(lock_cell).is_none(),
        "Locked should be removed for mixed despawn/Dead-marked adjacents"
    );
    assert!(
        app.world().get::<Invulnerable>(lock_cell).is_none(),
        "Invulnerable should be removed for mixed case"
    );
}

/// Behavior 9b edge: one adjacent Dead-marked, the other still fully alive.
/// Only one counts as destroyed → lock cell does NOT unlock.
#[test]
fn lock_stays_locked_when_one_adjacent_dead_marked_and_one_alive() {
    let mut app = lock_release_app();

    let adj_a = app
        .world_mut()
        .spawn((Cell, Hp::new(1.0), KilledBy::default()))
        .id();
    let adj_b = app
        .world_mut()
        .spawn((Cell, Hp::new(1.0), KilledBy::default()))
        .id();

    // Only adj_a marked Dead.
    app.world_mut().entity_mut(adj_a).insert(Dead);

    let lock_cell = app
        .world_mut()
        .spawn((
            Cell,
            LockCell,
            Locked,
            Invulnerable,
            Locks(vec![adj_a, adj_b]),
            Hp::new(10.0),
            KilledBy::default(),
        ))
        .id();

    app.world_mut().resource_mut::<TestDestroyedMessages>().0 =
        vec![make_destroyed_msg(), make_destroyed_msg()];

    tick(&mut app);

    assert!(
        app.world().get::<Locked>(lock_cell).is_some(),
        "Locked should remain — adj_b is still fully alive"
    );
    assert!(
        app.world().get::<Unlocked>(lock_cell).is_none(),
        "Unlocked should NOT be inserted"
    );
    assert!(
        app.world().get::<Invulnerable>(lock_cell).is_some(),
        "Invulnerable should remain because Locked remains"
    );
}

// ---------------------------------------------------------------
// Behavior 9 edge: Lock cell with empty adjacents unlocks immediately
// ---------------------------------------------------------------

#[test]
fn lock_cell_with_empty_adjacents_unlocks_immediately() {
    let mut app = lock_release_app();

    // Lock cell with empty locks list -- edge case.
    let lock_cell = app
        .world_mut()
        .spawn((
            Cell,
            LockCell,
            Locked,
            Invulnerable,
            Locks(vec![]),
            Hp::new(10.0),
            KilledBy::default(),
        ))
        .id();

    // No Destroyed<Cell> messages needed -- the locks list is empty.
    tick(&mut app);

    // Empty locks vec => all adjacents are "destroyed" => Locked removed.
    assert!(
        app.world().get::<Locked>(lock_cell).is_none(),
        "Locked should be removed immediately when locks list is empty"
    );
    assert!(
        app.world().get::<Unlocked>(lock_cell).is_some(),
        "Unlocked should be inserted when lock releases immediately"
    );
}
