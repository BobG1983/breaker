//! Group B — `init_sequence_groups` at `OnEnter(NodeState::Playing)`.

use bevy::prelude::*;

use super::helpers::*;
use crate::cells::components::SequenceActive;

// ── Behavior 5 ─────────────────────────────────────────────────────────────

#[test]
fn before_playing_no_sequence_cell_has_active_marker() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    let e2 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 2, 20.0);

    // Before navigating into NodeState::Playing, no cell should be active.
    // We do NOT call app.update() here because NodeState is a sub-state of
    // RunState::Node and has no value until the state hierarchy walks into
    // it — the assertions are on spawn-time state only.
    assert!(
        app.world().get::<SequenceActive>(e0).is_none(),
        "position-0 cell must not be active until OnEnter(NodeState::Playing) fires"
    );
    assert!(app.world().get::<SequenceActive>(e1).is_none());
    assert!(app.world().get::<SequenceActive>(e2).is_none());
}

// ── Behavior 5 edge: spawn two cells, navigate to Playing, then confirm position-0 becomes active
#[test]
fn spawn_before_playing_then_advance_to_playing_marks_position_zero_active() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    let e2 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 2, 20.0);

    advance_to_playing(&mut app);

    assert!(
        app.world().get::<SequenceActive>(e0).is_some(),
        "position-0 cell should be active after OnEnter(NodeState::Playing)"
    );
    assert!(app.world().get::<SequenceActive>(e1).is_none());
    assert!(app.world().get::<SequenceActive>(e2).is_none());
}

// ── Behavior 5b ────────────────────────────────────────────────────────────

#[test]
fn cells_spawned_after_on_enter_playing_never_become_active() {
    let mut app = build_sequence_test_app();

    // Drive into Playing BEFORE any cells exist. `init_sequence_groups` will
    // run on an empty set and fire its OnEnter body once.
    advance_to_playing(&mut app);

    // Now spawn three sequence cells in group 1.
    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    let e2 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 2, 20.0);

    app.update();

    assert!(
        app.world().get::<SequenceActive>(e0).is_none(),
        "cells spawned after OnEnter(NodeState::Playing) must not be auto-activated"
    );
    assert!(app.world().get::<SequenceActive>(e1).is_none());
    assert!(app.world().get::<SequenceActive>(e2).is_none());
}

// ── Behavior 5b edge: also spawn a post-transition cell in a different group
#[test]
fn post_transition_cells_in_other_groups_also_do_not_become_active() {
    let mut app = build_sequence_test_app();

    advance_to_playing(&mut app);

    let g1_p0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let g2_p0 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 2, 0, 20.0);

    app.update();

    assert!(
        app.world().get::<SequenceActive>(g1_p0).is_none(),
        "g1 position-0 spawned after transition must not be auto-activated"
    );
    assert!(
        app.world().get::<SequenceActive>(g2_p0).is_none(),
        "g2 position-0 spawned after transition must not be auto-activated either"
    );
}

// ── Behavior 6 ─────────────────────────────────────────────────────────────

#[test]
fn three_cell_group_marks_position_zero_active_on_playing() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    let e2 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 2, 20.0);

    advance_to_playing(&mut app);

    assert!(app.world().get::<SequenceActive>(e0).is_some());
    assert!(app.world().get::<SequenceActive>(e1).is_none());
    assert!(app.world().get::<SequenceActive>(e2).is_none());
}

// ── Behavior 6 edge: reversed spawn order still resolves e0 as active
#[test]
fn reversed_spawn_order_still_selects_position_zero_as_active() {
    let mut app = build_sequence_test_app();

    let e2 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 2, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);

    advance_to_playing(&mut app);

    assert!(
        app.world().get::<SequenceActive>(e0).is_some(),
        "position-0 cell should be active regardless of spawn order"
    );
    assert!(app.world().get::<SequenceActive>(e1).is_none());
    assert!(app.world().get::<SequenceActive>(e2).is_none());
}

// ── Behavior 7 ─────────────────────────────────────────────────────────────

#[test]
fn multiple_disjoint_groups_each_activate_their_own_position_zero_cell() {
    let mut app = build_sequence_test_app();

    let g0p0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 0, 0, 20.0);
    let g0p1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 0, 1, 20.0);
    let g1p0 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 0, 20.0);
    let g1p1 = spawn_sequence_cell(&mut app, Vec2::new(30.0, 0.0), 1, 1, 20.0);

    advance_to_playing(&mut app);

    assert!(app.world().get::<SequenceActive>(g0p0).is_some());
    assert!(app.world().get::<SequenceActive>(g1p0).is_some());
    assert!(app.world().get::<SequenceActive>(g0p1).is_none());
    assert!(app.world().get::<SequenceActive>(g1p1).is_none());
}

// ── Behavior 7 edge: non-sequential group ids (0, 42, 1000) are each handled
#[test]
fn non_sequential_group_ids_each_activate_their_position_zero_cell() {
    let mut app = build_sequence_test_app();

    let g0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 0, 0, 20.0);
    let g42 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 42, 0, 20.0);
    let g1000 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1000, 0, 20.0);

    advance_to_playing(&mut app);

    assert!(
        app.world().get::<SequenceActive>(g0).is_some(),
        "group 0 position 0 should be active"
    );
    assert!(
        app.world().get::<SequenceActive>(g42).is_some(),
        "group 42 position 0 should be active"
    );
    assert!(
        app.world().get::<SequenceActive>(g1000).is_some(),
        "group 1000 position 0 should be active"
    );
}

// ── Behavior 8 ─────────────────────────────────────────────────────────────

#[test]
fn group_with_no_position_zero_cell_leaves_every_member_inactive() {
    let mut app = build_sequence_test_app();

    let e1 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 5, 1, 20.0);
    let e2 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 5, 2, 20.0);

    advance_to_playing(&mut app);

    assert!(
        app.world().get::<SequenceActive>(e1).is_none(),
        "group 5 has no position 0 — nothing should become active"
    );
    assert!(app.world().get::<SequenceActive>(e2).is_none());
}

// ── Behavior 8 edge: proper second group is independent of the malformed one
#[test]
fn malformed_group_does_not_affect_proper_group() {
    let mut app = build_sequence_test_app();

    let malformed_e1 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 5, 1, 20.0);
    let malformed_e2 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 5, 2, 20.0);
    let proper = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 6, 0, 20.0);

    advance_to_playing(&mut app);

    assert!(app.world().get::<SequenceActive>(malformed_e1).is_none());
    assert!(app.world().get::<SequenceActive>(malformed_e2).is_none());
    assert!(
        app.world().get::<SequenceActive>(proper).is_some(),
        "proper group 6 position 0 should be active regardless of malformed group 5"
    );
}

// ── Behavior 9 ─────────────────────────────────────────────────────────────

#[test]
fn two_position_zero_cells_in_same_group_both_become_active() {
    let mut app = build_sequence_test_app();

    let a = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 2, 0, 20.0);
    let b = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 2, 0, 20.0);

    advance_to_playing(&mut app);

    assert!(
        app.world().get::<SequenceActive>(a).is_some(),
        "malformed duplicate position-0 should still insert on both — surfaces the bad state"
    );
    assert!(app.world().get::<SequenceActive>(b).is_some());
}

// ── Behavior 9 edge: position-1 cell in the same group stays inactive
#[test]
fn duplicate_position_zero_does_not_activate_position_one_in_same_group() {
    let mut app = build_sequence_test_app();

    let a = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 2, 0, 20.0);
    let b = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 2, 0, 20.0);
    let c = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 2, 1, 20.0);

    advance_to_playing(&mut app);

    assert!(app.world().get::<SequenceActive>(a).is_some());
    assert!(app.world().get::<SequenceActive>(b).is_some());
    assert!(
        app.world().get::<SequenceActive>(c).is_none(),
        "only position-0 cells should be touched by init_sequence_groups"
    );
}
