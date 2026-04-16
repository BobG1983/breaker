//! Group D — `advance_sequence` on death.

use bevy::prelude::*;

use super::helpers::*;
use crate::{cells::components::SequenceActive, prelude::*};

// ── Behavior 16 ────────────────────────────────────────────────────────────

#[test]
fn killing_active_position_zero_promotes_position_one_to_active() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    let e2 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 2, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e0, 25.0));
    let destroyed = run_ticks_capture_destroyed(&mut app, 2);

    assert!(
        destroyed.iter().any(|m| m.victim == e0),
        "e0 should be in the Destroyed<Cell> set after lethal damage"
    );
    assert!(
        app.world().get::<SequenceActive>(e1).is_some(),
        "e1 (position 1) should have SequenceActive after e0 dies"
    );
    assert!(
        app.world().get::<SequenceActive>(e2).is_none(),
        "e2 (position 2) should not yet be active"
    );
}

// ── Behavior 16 edge: promoting e1 did not touch its HP
#[test]
fn promoting_next_position_does_not_alter_its_hp() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    let _e2 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 2, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e0, 25.0));
    run_ticks_capture_destroyed(&mut app, 2);

    let hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "promoting e1 must not alter HP, got {}",
        hp.current
    );
}

// ── Behavior 17 ────────────────────────────────────────────────────────────

#[test]
fn sequential_promotion_from_position_zero_through_position_two() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    let e2 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 2, 20.0);
    advance_to_playing(&mut app);

    // Tick 1: kill e0 (active) — promotes e1.
    push_damage(&mut app, damage_msg(e0, 25.0));
    tick(&mut app);

    // Tick 2: kill e1 (now active) — promotes e2.
    push_damage(&mut app, damage_msg(e1, 25.0));
    tick(&mut app);

    assert!(
        app.world().get_entity(e0).is_err() || app.world().get::<Dead>(e0).is_some(),
        "e0 should be dead or despawned"
    );
    assert!(
        app.world().get_entity(e1).is_err() || app.world().get::<Dead>(e1).is_some(),
        "e1 should be dead or despawned"
    );
    assert!(
        app.world().get::<SequenceActive>(e2).is_some(),
        "e2 should be active after e1 dies"
    );
}

// ── Behavior 17 edge: killing the last cell is a graceful no-op
#[test]
fn killing_last_cell_in_group_is_graceful_noop() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    let e2 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 2, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e0, 25.0));
    tick(&mut app);
    push_damage(&mut app, damage_msg(e1, 25.0));
    tick(&mut app);
    push_damage(&mut app, damage_msg(e2, 25.0));
    tick(&mut app);

    assert!(
        app.world().get_entity(e2).is_err() || app.world().get::<Dead>(e2).is_some(),
        "e2 should be dead after the final hit"
    );
    // No cell should have SequenceActive after all three are dead — advance_sequence
    // must not panic when there is no position + 1 cell to promote.
}

// ── Behavior 18 ────────────────────────────────────────────────────────────

#[test]
fn killing_non_existent_sequence_position_is_graceful_noop() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 9, 0, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e0, 25.0));
    run_ticks_capture_destroyed(&mut app, 2);

    assert!(
        app.world().get_entity(e0).is_err() || app.world().get::<Dead>(e0).is_some(),
        "e0 should be dead"
    );
    // No panic expected, no other cell to activate.
}

// ── Behavior 18 edge: unrelated plain cell is not touched
#[test]
fn killing_lone_sequence_cell_does_not_leak_to_non_sequence_neighbors() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 9, 0, 20.0);
    let x = spawn_plain_cell(&mut app, Vec2::new(50.0, 0.0), 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e0, 25.0));
    run_ticks_capture_destroyed(&mut app, 2);

    assert!(
        app.world().get::<SequenceActive>(x).is_none(),
        "plain cell must not receive SequenceActive from a lone-group death"
    );
    assert!(app.world().get_entity(x).is_ok());
}

// ── Behavior 19 ────────────────────────────────────────────────────────────

#[test]
fn advance_sequence_fires_on_destroyed_from_volatile_chain_reaction() {
    let mut app = build_sequence_test_app();

    let v0 = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let e0 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(50.0, 0.0), 1, 1, 20.0);
    let e2 = spawn_sequence_cell(&mut app, Vec2::new(80.0, 0.0), 1, 2, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(v0, 100.0));
    let (destroyed, _damage) = run_ticks_capture_destroyed_and_damage(&mut app, 3);

    let victims: std::collections::HashSet<Entity> = destroyed.iter().map(|m| m.victim).collect();
    assert!(victims.contains(&v0), "v0 must be destroyed");
    assert!(
        victims.contains(&e0),
        "e0 must be destroyed by the volatile blast (active cell takes damage normally)"
    );
    assert!(
        !victims.contains(&e1),
        "e1 must NOT be destroyed — it is outside the radius"
    );

    assert!(
        app.world().get::<SequenceActive>(e1).is_some(),
        "e1 should be active after e0's death promotes it"
    );
    assert!(
        app.world().get::<SequenceActive>(e2).is_none(),
        "e2 should still be inactive"
    );

    let e1_hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (e1_hp.current - 20.0).abs() < f32::EPSILON,
        "e1 should still be at full HP, got {}",
        e1_hp.current
    );
}

// ── Behavior 19 edge: e1 inside the explosion radius and non-active — still promoted
#[test]
fn volatile_chain_into_non_active_cell_resets_hp_then_promotes_on_e0_death() {
    let mut app = build_sequence_test_app();

    let v0 = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let _e0 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(30.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(v0, 100.0));
    let (_destroyed, _damage) = run_ticks_capture_destroyed_and_damage(&mut app, 3);

    let e1_hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (e1_hp.current - 20.0).abs() < f32::EPSILON,
        "e1 should have been reset (blast damage reverted), got {}",
        e1_hp.current
    );
    assert!(app.world().get::<Dead>(e1).is_none());
    assert!(
        app.world().get::<SequenceActive>(e1).is_some(),
        "e1 should be active after e0's death promotes it"
    );
}

// ── Behavior 20 ────────────────────────────────────────────────────────────

#[test]
fn advance_sequence_ignores_deaths_of_cells_without_sequence_group() {
    let mut app = build_sequence_test_app();

    let p = spawn_plain_cell(&mut app, Vec2::new(0.0, 0.0), 20.0);
    let e0 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 0, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(p, 25.0));
    run_ticks_capture_destroyed(&mut app, 2);

    let e0_hp = app.world().get::<Hp>(e0).expect("e0 should still have Hp");
    assert!(
        (e0_hp.current - 20.0).abs() < f32::EPSILON,
        "e0 should be unchanged, got {}",
        e0_hp.current
    );
    assert!(
        app.world().get::<SequenceActive>(e0).is_some(),
        "e0 should still be active"
    );
}

// ── Behavior 20 edge: two plain cells die — still no cross-leak
#[test]
fn two_plain_deaths_in_one_tick_do_not_affect_sequence_state() {
    let mut app = build_sequence_test_app();

    let p1 = spawn_plain_cell(&mut app, Vec2::new(0.0, 0.0), 20.0);
    let p2 = spawn_plain_cell(&mut app, Vec2::new(40.0, 0.0), 20.0);
    let e0 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(30.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(p1, 25.0));
    push_damage(&mut app, damage_msg(p2, 25.0));
    run_ticks_capture_destroyed(&mut app, 2);

    assert!(
        app.world().get::<SequenceActive>(e0).is_some(),
        "e0 still active"
    );
    assert!(
        app.world().get::<SequenceActive>(e1).is_none(),
        "e1 still inactive"
    );
}
