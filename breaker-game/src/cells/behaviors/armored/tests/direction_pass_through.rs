//! Group C — `check_armor_direction` pass-through cases (weak face, non-armored).

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::components::PiercingRemaining, cells::behaviors::armored::components::ArmorDirection,
    prelude::*,
};

// ── Behavior 11 ────────────────────────────────────────────────────────────

#[test]
fn weak_face_passes_through_bottom_armored_cell_top_face_hit() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "weak face hit should apply damage, got hp.current == {}",
        hp.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        0,
        "piercing should remain 0 on weak face hit"
    );
    assert!(app.world().get::<Dead>(cell).is_none());
}

// ── Behavior 11 edge: ArmorValue(3) on weak face still passes ─────────────

#[test]
fn weak_face_passes_through_regardless_of_armor_value() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        3,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "armor value must not matter on weak face, got hp.current == {}",
        hp.current
    );
    assert_eq!(app.world().get::<PiercingRemaining>(bolt).unwrap().0, 0);
}

// ── Behavior 12 ────────────────────────────────────────────────────────────

#[test]
fn non_armored_cell_passes_through_bottom_face_impact_unchanged() {
    let mut app = build_armored_test_app();

    let cell = spawn_plain_cell(&mut app, Vec2::new(0.0, 0.0), 20.0);
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "non-armored cell should take full damage, got hp.current == {}",
        hp.current
    );
    assert_eq!(app.world().get::<PiercingRemaining>(bolt).unwrap().0, 0);
    assert!(app.world().get::<Dead>(cell).is_none());
}

// ── Behavior 12 edge: NEG_X on non-armored with piercing 5 ────────────────

#[test]
fn non_armored_cell_neg_x_impact_passes_through_with_piercing_unchanged() {
    let mut app = build_armored_test_app();

    let cell = spawn_plain_cell(&mut app, Vec2::new(0.0, 0.0), 20.0);
    let bolt = spawn_test_bolt(&mut app, 5);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_X, 5));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "got hp.current == {}",
        hp.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        5,
        "no armor means no piercing consumption"
    );
}

// ── Behavior 12 edge: ZERO normal on non-armored cell ─────────────────────

#[test]
fn non_armored_cell_zero_normal_passes_through() {
    let mut app = build_armored_test_app();

    let cell = spawn_plain_cell(&mut app, Vec2::new(0.0, 0.0), 20.0);
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::ZERO, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "ZERO normal on non-armored cell should pass through, got hp.current == {}",
        hp.current
    );
}

// ── Behavior 12 edge: ZERO normal on ARMORED cell ─────────────────────────

#[test]
fn armored_cell_zero_normal_passes_through_strict_inequality() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::ZERO, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "ZERO normal should match no facing (strict inequalities), got hp.current == {}",
        hp.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        0,
        "piercing unchanged on ZERO normal"
    );
}

// ── Behavior 13 ────────────────────────────────────────────────────────────

#[test]
fn plain_cell_lethal_damage_still_applies_control_test() {
    let mut app = build_armored_test_app();

    let cell = spawn_plain_cell(&mut app, Vec2::new(0.0, 0.0), 5.0);
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    // Two ticks: first applies damage, second lets HandleKill run.
    tick(&mut app);
    tick(&mut app);

    let is_dead_or_gone =
        app.world().get_entity(cell).is_err() || app.world().get::<Dead>(cell).is_some();
    assert!(
        is_dead_or_gone,
        "cell should be destroyed after lethal damage — harness sanity control"
    );
}
