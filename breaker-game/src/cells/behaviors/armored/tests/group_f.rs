//! Group F — All four facings and multi-hit per tick.
//!
//! Exercises every `ArmorDirection` variant and verifies the per-tick
//! blocklist mechanism handles multiple independent hits without cross-wiring.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::components::PiercingRemaining, cells::behaviors::armored::components::ArmorDirection,
    prelude::*,
};

// ── Behavior 21 ────────────────────────────────────────────────────────────

#[test]
fn armor_direction_top_blocks_vec2_y() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(&mut app, Vec2::new(0.0, 0.0), 1, ArmorDirection::Top, 20.0);
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Top armor blocks Y hit, got hp.current == {}",
        hp.current
    );
    assert_eq!(app.world().get::<PiercingRemaining>(bolt).unwrap().0, 0);
}

// ── Behavior 21 edge: Top armor, NEG_Y (weak) passes ──────────────────────

#[test]
fn armor_direction_top_passes_vec2_neg_y() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(&mut app, Vec2::new(0.0, 0.0), 1, ArmorDirection::Top, 20.0);
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "Bottom weak face should pass through, got hp.current == {}",
        hp.current
    );
    assert_eq!(app.world().get::<PiercingRemaining>(bolt).unwrap().0, 0);
}

// ── Behavior 22 ────────────────────────────────────────────────────────────

#[test]
fn armor_direction_left_blocks_vec2_neg_x() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(&mut app, Vec2::new(0.0, 0.0), 1, ArmorDirection::Left, 20.0);
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_X, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Left armor blocks NEG_X hit, got hp.current == {}",
        hp.current
    );
    assert_eq!(app.world().get::<PiercingRemaining>(bolt).unwrap().0, 0);
}

// ── Behavior 22 edge: Left armor, X (weak) passes ─────────────────────────

#[test]
fn armor_direction_left_passes_vec2_x() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(&mut app, Vec2::new(0.0, 0.0), 1, ArmorDirection::Left, 20.0);
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::X, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "Right weak face should pass through, got hp.current == {}",
        hp.current
    );
    assert_eq!(app.world().get::<PiercingRemaining>(bolt).unwrap().0, 0);
}

// ── Behavior 23 ────────────────────────────────────────────────────────────

#[test]
fn armor_direction_right_blocks_vec2_x() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        1,
        ArmorDirection::Right,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::X, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Right armor blocks X hit, got hp.current == {}",
        hp.current
    );
    assert_eq!(app.world().get::<PiercingRemaining>(bolt).unwrap().0, 0);
}

// ── Behavior 23 edge: Right armor, NEG_X (weak) passes ────────────────────

#[test]
fn armor_direction_right_passes_vec2_neg_x() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        1,
        ArmorDirection::Right,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_X, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "Left weak face should pass through, got hp.current == {}",
        hp.current
    );
    assert_eq!(app.world().get::<PiercingRemaining>(bolt).unwrap().0, 0);
}

// ── Behavior 24 ────────────────────────────────────────────────────────────

#[test]
fn two_armored_cells_one_bolt_breakthrough_on_a_block_on_b() {
    let mut app = build_armored_test_app();

    let cell_a = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let cell_b = spawn_armored_cell(
        &mut app,
        Vec2::new(20.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 3);

    // A: piercing snapshot 3 >= armor 2 -> breakthrough
    push_bolt_impact(&mut app, bolt_impact(bolt, cell_a, Vec2::NEG_Y, 3));
    // B: piercing snapshot 1 < armor 2 -> block
    push_bolt_impact(&mut app, bolt_impact(bolt, cell_b, Vec2::NEG_Y, 1));

    push_damage(&mut app, damage_msg_from(cell_a, 5.0, bolt));
    push_damage(&mut app, damage_msg_from(cell_b, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp_a = app.world().get::<Hp>(cell_a).unwrap();
    assert!(
        (hp_a.current - 15.0).abs() < f32::EPSILON,
        "cell A breakthrough, got hp.current == {}",
        hp_a.current
    );
    let hp_b = app.world().get::<Hp>(cell_b).unwrap();
    assert!(
        (hp_b.current - 20.0).abs() < f32::EPSILON,
        "cell B blocked, got hp.current == {}",
        hp_b.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        1,
        "3 - 2 = 1; only breakthrough on A consumes"
    );
}

// ── Behavior 24 edge: reversed order of bolt impact messages ──────────────

#[test]
fn two_armored_cells_one_bolt_reversed_message_order_same_result() {
    let mut app = build_armored_test_app();

    let cell_a = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let cell_b = spawn_armored_cell(
        &mut app,
        Vec2::new(20.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 3);

    // Reversed: B's block message first, then A's breakthrough
    push_bolt_impact(&mut app, bolt_impact(bolt, cell_b, Vec2::NEG_Y, 1));
    push_bolt_impact(&mut app, bolt_impact(bolt, cell_a, Vec2::NEG_Y, 3));

    push_damage(&mut app, damage_msg_from(cell_a, 5.0, bolt));
    push_damage(&mut app, damage_msg_from(cell_b, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp_a = app.world().get::<Hp>(cell_a).unwrap();
    assert!(
        (hp_a.current - 15.0).abs() < f32::EPSILON,
        "cell A breakthrough regardless of message order, got {}",
        hp_a.current
    );
    let hp_b = app.world().get::<Hp>(cell_b).unwrap();
    assert!(
        (hp_b.current - 20.0).abs() < f32::EPSILON,
        "cell B blocked regardless of message order, got {}",
        hp_b.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        1,
        "order-independent result"
    );
}

// ── Behavior 25 ────────────────────────────────────────────────────────────

#[test]
fn two_bolts_one_cell_same_tick_both_blocked_independently() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt_a = spawn_test_bolt(&mut app, 0);
    let bolt_b = spawn_test_bolt(&mut app, 1);

    push_bolt_impact(&mut app, bolt_impact(bolt_a, cell, Vec2::NEG_Y, 0));
    push_bolt_impact(&mut app, bolt_impact(bolt_b, cell, Vec2::NEG_Y, 1));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt_a));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt_b));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "both hits blocked, got hp.current == {}",
        hp.current
    );
    assert_eq!(app.world().get::<PiercingRemaining>(bolt_a).unwrap().0, 0);
    assert_eq!(app.world().get::<PiercingRemaining>(bolt_b).unwrap().0, 1);
    assert!(app.world().get::<Dead>(cell).is_none());
}

// ── Behavior 25 edge: A breakthrough, B block on same cell ────────────────

#[test]
fn two_bolts_one_cell_bolt_a_breakthrough_bolt_b_block() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt_a = spawn_test_bolt(&mut app, 3);
    let bolt_b = spawn_test_bolt(&mut app, 1);

    push_bolt_impact(&mut app, bolt_impact(bolt_a, cell, Vec2::NEG_Y, 3));
    push_bolt_impact(&mut app, bolt_impact(bolt_b, cell, Vec2::NEG_Y, 1));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt_a));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt_b));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "only bolt A's breakthrough applies, got hp.current == {}",
        hp.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt_a).unwrap().0,
        1,
        "bolt A: 3 - 2 = 1"
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt_b).unwrap().0,
        1,
        "bolt B: blocked, stays at 1"
    );
}

// ── Behavior 26 ────────────────────────────────────────────────────────────

#[test]
fn same_bolt_two_armored_cells_breakthrough_on_both_with_saturating_piercing() {
    let mut app = build_armored_test_app();

    let cell_a = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        1,
        ArmorDirection::Bottom,
        20.0,
    );
    let cell_b = spawn_armored_cell(
        &mut app,
        Vec2::new(20.0, 0.0),
        1,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 2);

    push_bolt_impact(&mut app, bolt_impact(bolt, cell_a, Vec2::NEG_Y, 2));
    push_bolt_impact(&mut app, bolt_impact(bolt, cell_b, Vec2::NEG_Y, 2));
    push_damage(&mut app, damage_msg_from(cell_a, 5.0, bolt));
    push_damage(&mut app, damage_msg_from(cell_b, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp_a = app.world().get::<Hp>(cell_a).unwrap();
    assert!(
        (hp_a.current - 15.0).abs() < f32::EPSILON,
        "cell A breakthrough, got {}",
        hp_a.current
    );
    let hp_b = app.world().get::<Hp>(cell_b).unwrap();
    assert!(
        (hp_b.current - 15.0).abs() < f32::EPSILON,
        "cell B breakthrough, got {}",
        hp_b.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        0,
        "2 - 1 - 1 = 0"
    );
}

// ── Behavior 26 edge: sequential snapshots with ArmorValue(2) ─────────────

#[test]
fn same_bolt_two_armored_cells_sequential_snapshots_armor_value_2() {
    let mut app = build_armored_test_app();

    let cell_a = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let cell_b = spawn_armored_cell(
        &mut app,
        Vec2::new(20.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 2);

    // Sequential snapshots: A sees 2 (before CCD decrement), B sees 1 (after)
    push_bolt_impact(&mut app, bolt_impact(bolt, cell_a, Vec2::NEG_Y, 2));
    push_bolt_impact(&mut app, bolt_impact(bolt, cell_b, Vec2::NEG_Y, 1));
    push_damage(&mut app, damage_msg_from(cell_a, 5.0, bolt));
    push_damage(&mut app, damage_msg_from(cell_b, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp_a = app.world().get::<Hp>(cell_a).unwrap();
    assert!(
        (hp_a.current - 15.0).abs() < f32::EPSILON,
        "cell A: 2 >= 2 breakthrough, got {}",
        hp_a.current
    );
    let hp_b = app.world().get::<Hp>(cell_b).unwrap();
    assert!(
        (hp_b.current - 20.0).abs() < f32::EPSILON,
        "cell B: 1 < 2 block, got {}",
        hp_b.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        0,
        "CCD decremented twice 2->1->0, then armor decremented by 2 saturating at 0"
    );
}
