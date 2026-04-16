//! Group E — Single-cell groups and positional edges.

use bevy::prelude::*;

use super::helpers::*;
use crate::{cells::components::SequenceActive, prelude::*};

// ── Behavior 21 ────────────────────────────────────────────────────────────

#[test]
fn single_cell_group_becomes_active_and_dies_without_promotion() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 3, 0, 20.0);
    advance_to_playing(&mut app);

    assert!(
        app.world().get::<SequenceActive>(e0).is_some(),
        "lone member should be active on Playing"
    );

    push_damage(&mut app, damage_msg(e0, 25.0));
    run_ticks_capture_destroyed(&mut app, 2);

    assert!(
        app.world().get_entity(e0).is_err() || app.world().get::<Dead>(e0).is_some(),
        "e0 should be dead after lethal damage"
    );
    // No other cell; no panic expected.
}

// ── Behavior 21 edge: gap of 6 — advance does a strict +1 lookup
#[test]
fn strict_position_plus_one_does_not_skip_to_position_seven() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 3, 0, 20.0);
    let e7 = spawn_sequence_cell(&mut app, Vec2::new(70.0, 0.0), 3, 7, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e0, 25.0));
    run_ticks_capture_destroyed(&mut app, 2);

    assert!(
        app.world().get_entity(e0).is_err() || app.world().get::<Dead>(e0).is_some(),
        "e0 should be dead"
    );
    assert!(
        app.world().get::<SequenceActive>(e7).is_none(),
        "e7 at position 7 must NOT be promoted — advance is strict +1"
    );
}

// ── Behavior 22 ────────────────────────────────────────────────────────────

#[test]
fn gap_in_positions_stalls_the_group() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 4, 0, 20.0);
    let e2 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 4, 2, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e0, 25.0));
    run_ticks_capture_destroyed(&mut app, 2);

    assert!(
        app.world().get_entity(e0).is_err() || app.world().get::<Dead>(e0).is_some(),
        "e0 should be dead"
    );
    assert!(
        app.world().get::<SequenceActive>(e2).is_none(),
        "e2 at position 2 must remain inactive — no position 1 exists"
    );
}

// ── Behavior 22 edge: manually-seeded SequenceActive is not removed
#[test]
fn advance_sequence_is_additive_never_removes_active_marker() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 4, 0, 20.0);
    let e2 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 4, 2, 20.0);
    advance_to_playing(&mut app);

    // Seed e2 as active — advance_sequence must not remove this.
    app.world_mut().entity_mut(e2).insert(SequenceActive);

    push_damage(&mut app, damage_msg(e0, 25.0));
    tick(&mut app);

    assert!(
        app.world().get::<SequenceActive>(e2).is_some(),
        "manually-seeded SequenceActive on e2 must remain after advance"
    );
}

// ── Behavior 23 ────────────────────────────────────────────────────────────

#[test]
fn duplicate_positions_in_same_group_both_receive_active_on_promotion() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 5, 0, 20.0);
    let e1a = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 5, 1, 20.0);
    let e1b = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 5, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e0, 25.0));
    run_ticks_capture_destroyed(&mut app, 2);

    assert!(
        app.world().get_entity(e0).is_err() || app.world().get::<Dead>(e0).is_some(),
        "e0 should be dead"
    );
    assert!(
        app.world().get::<SequenceActive>(e1a).is_some(),
        "both duplicate position-1 cells should be promoted"
    );
    assert!(
        app.world().get::<SequenceActive>(e1b).is_some(),
        "both duplicate position-1 cells should be promoted"
    );
}

// ── Behavior 23 edge: killing both duplicates then advancing graceful-no-ops
#[test]
fn killing_duplicate_position_cells_advances_to_gap_gracefully() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 5, 0, 20.0);
    let e1a = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 5, 1, 20.0);
    let e1b = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 5, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e0, 25.0));
    tick(&mut app);
    push_damage(&mut app, damage_msg(e1a, 25.0));
    push_damage(&mut app, damage_msg(e1b, 25.0));
    tick(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(e1a).is_err() || app.world().get::<Dead>(e1a).is_some(),
        "e1a should be dead"
    );
    assert!(
        app.world().get_entity(e1b).is_err() || app.world().get::<Dead>(e1b).is_some(),
        "e1b should be dead"
    );
    // No panic — no position 2 cell exists to promote to.
}
