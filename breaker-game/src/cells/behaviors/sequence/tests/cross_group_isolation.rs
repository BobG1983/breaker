//! Group G — Cross-group isolation and commands-based state manipulation.

use bevy::prelude::*;

use super::helpers::*;
use crate::{cells::components::SequenceActive, prelude::*};

// ── Behavior 27 ────────────────────────────────────────────────────────────

#[test]
fn two_groups_with_overlapping_positions_operate_independently() {
    let mut app = build_sequence_test_app();

    let a0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 0, 0, 20.0);
    let a1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 0, 1, 20.0);
    let b0 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 0, 20.0);
    let b1 = spawn_sequence_cell(&mut app, Vec2::new(30.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(a0, 25.0));
    run_ticks_capture_destroyed(&mut app, 2);

    assert!(
        app.world().get_entity(a0).is_err() || app.world().get::<Dead>(a0).is_some(),
        "a0 should be dead"
    );
    assert!(
        app.world().get::<SequenceActive>(a1).is_some(),
        "a1 should be active after a0 dies"
    );

    let b0_hp = app.world().get::<Hp>(b0).unwrap();
    assert!((b0_hp.current - 20.0).abs() < f32::EPSILON);
    assert!(
        app.world().get::<SequenceActive>(b0).is_some(),
        "b0 should still be active (unrelated group)"
    );
    assert!(app.world().get::<SequenceActive>(b1).is_none());
}

// ── Behavior 27 edge: simultaneous damage to both active cells
#[test]
fn simultaneous_damage_on_two_active_cells_promotes_both_groups() {
    let mut app = build_sequence_test_app();

    let a0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 0, 0, 20.0);
    let a1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 0, 1, 20.0);
    let b0 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 0, 20.0);
    let b1 = spawn_sequence_cell(&mut app, Vec2::new(30.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(a0, 25.0));
    push_damage(&mut app, damage_msg(b0, 25.0));
    run_ticks_capture_destroyed(&mut app, 2);

    assert!(
        app.world().get_entity(a0).is_err() || app.world().get::<Dead>(a0).is_some(),
        "a0 should be dead"
    );
    assert!(
        app.world().get_entity(b0).is_err() || app.world().get::<Dead>(b0).is_some(),
        "b0 should be dead"
    );
    assert!(
        app.world().get::<SequenceActive>(a1).is_some(),
        "a1 should be promoted to active"
    );
    assert!(
        app.world().get::<SequenceActive>(b1).is_some(),
        "b1 should be promoted to active"
    );
}

// ── Behavior 28 ────────────────────────────────────────────────────────────

#[test]
fn removing_sequence_active_mid_run_reverts_cell_to_non_active_gating() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    advance_to_playing(&mut app);

    assert!(
        app.world().get::<SequenceActive>(e0).is_some(),
        "e0 should be active after init_sequence_groups runs"
    );

    app.world_mut().entity_mut(e0).remove::<SequenceActive>();
    push_damage(&mut app, damage_msg(e0, 5.0));
    tick(&mut app);

    let hp = app.world().get::<Hp>(e0).expect("e0 should still have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "after removing SequenceActive, damage should be reverted, got {}",
        hp.current
    );
    assert!(app.world().get::<Dead>(e0).is_none());

    let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
    assert!(!destroyed.0.iter().any(|m| m.victim == e0));
}

// ── Behavior 28 edge: re-inserting SequenceActive restores normal damage
#[test]
fn re_inserting_sequence_active_restores_damage_gating() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    advance_to_playing(&mut app);

    app.world_mut().entity_mut(e0).remove::<SequenceActive>();
    push_damage(&mut app, damage_msg(e0, 5.0));
    tick(&mut app);
    let hp = app.world().get::<Hp>(e0).unwrap();
    assert!((hp.current - 20.0).abs() < f32::EPSILON);

    app.world_mut().entity_mut(e0).insert(SequenceActive);
    push_damage(&mut app, damage_msg(e0, 5.0));
    tick(&mut app);

    let hp = app.world().get::<Hp>(e0).expect("e0 should still have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "with SequenceActive restored, damage should land normally, got {}",
        hp.current
    );
}

// ── Behavior 29 ────────────────────────────────────────────────────────────

#[test]
fn manually_adding_sequence_active_to_non_zero_position_bypasses_reset() {
    let mut app = build_sequence_test_app();

    let _e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    let _e2 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 2, 20.0);
    advance_to_playing(&mut app);

    app.world_mut().entity_mut(e1).insert(SequenceActive);
    push_damage(&mut app, damage_msg(e1, 5.0));
    tick(&mut app);

    let hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "forced-active position-1 cell should take damage normally, got {}",
        hp.current
    );
    assert!(app.world().get::<Dead>(e1).is_none());
    assert!(
        app.world().get::<SequenceActive>(e1).is_some(),
        "e1 should still be active"
    );
}

// ── Behavior 29 edge: both e0 and e1 active simultaneously
#[test]
fn both_forced_active_cells_take_damage_normally_in_same_tick() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    app.world_mut().entity_mut(e1).insert(SequenceActive);

    push_damage(&mut app, damage_msg(e0, 5.0));
    push_damage(&mut app, damage_msg(e1, 5.0));
    tick(&mut app);

    let e0_hp = app.world().get::<Hp>(e0).unwrap();
    let e1_hp = app.world().get::<Hp>(e1).unwrap();
    assert!((e0_hp.current - 15.0).abs() < f32::EPSILON);
    assert!((e1_hp.current - 15.0).abs() < f32::EPSILON);
}
