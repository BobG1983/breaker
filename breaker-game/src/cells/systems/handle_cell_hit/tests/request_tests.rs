use bevy::prelude::*;

use super::helpers::*;
use crate::cells::messages::DamageCell;

// =========================================================================
// C7 Wave 2a: Two-Phase Destruction — handle_cell_hit writes
// RequestCellDestroyed instead of despawning (behaviors 29, 32)
// =========================================================================

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

// =========================================================================
// Phase 1B: RequestCellDestroyed carries was_required_to_clear
// =========================================================================

// --- Behavior 1: RequestCellDestroyed.was_required_to_clear=true for required cells ---

#[test]
fn request_cell_destroyed_was_required_true_for_required_cell() {
    let mut app = test_app();
    let cell = spawn_cell_at(&mut app, 10.0, Vec2::ZERO, true);

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
        1,
        "exactly one RequestCellDestroyed expected"
    );
    assert!(
        captured.0[0].was_required_to_clear,
        "RequestCellDestroyed.was_required_to_clear should be true for a cell with RequiredToClear"
    );
}

// --- Behavior 2: RequestCellDestroyed.was_required_to_clear=false for non-required cells ---

#[test]
fn request_cell_destroyed_was_required_false_for_non_required_cell() {
    let mut app = test_app();
    let cell = spawn_cell_at(&mut app, 10.0, Vec2::ZERO, false);

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
        1,
        "exactly one RequestCellDestroyed expected"
    );
    assert!(
        !captured.0[0].was_required_to_clear,
        "RequestCellDestroyed.was_required_to_clear should be false for a cell without RequiredToClear"
    );
}
