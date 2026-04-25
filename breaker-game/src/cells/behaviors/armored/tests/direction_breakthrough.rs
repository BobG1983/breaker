//! Group E — `check_armor_direction` BREAKTHROUGH path.
//!
//! Tests exercising the breakthrough branch (`piercing_remaining >= armor_value`
//! on the armored face).

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::components::PiercingRemaining, cells::behaviors::armored::components::ArmorDirection,
    prelude::*,
};

// ── Behavior 17 ────────────────────────────────────────────────────────────

#[test]
fn breakthrough_piercing_greater_than_armor_value_applies_damage_decrements_piercing() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 3);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 3));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "breakthrough should apply damage, got hp.current == {}",
        hp.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        1,
        "3 - 2 = 1"
    );
}

// ── Behavior 17 edge: PiercingRemaining(5) ArmorValue(3) ──────────────────

#[test]
fn breakthrough_piercing_5_armor_3_piercing_becomes_2() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        3,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 5);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 5));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "damage applied, got hp.current == {}",
        hp.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        2,
        "5 - 3 = 2"
    );
}

// ── Behavior 18 ────────────────────────────────────────────────────────────

#[test]
fn breakthrough_boundary_piercing_equals_armor_drops_to_zero() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 2);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 2));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "damage applied at exact boundary, got hp.current == {}",
        hp.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        0,
        "2 - 2 = 0"
    );
}

// ── Behavior 18 edge: piercing 1 armor 1 ──────────────────────────────────

#[test]
fn breakthrough_boundary_piercing_1_armor_1_drops_to_zero() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        1,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 1);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 1));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "got hp.current == {}",
        hp.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        0,
        "1 - 1 = 0"
    );
}

// ── Behavior 18 edge: piercing 3 armor 3 ──────────────────────────────────

#[test]
fn breakthrough_boundary_piercing_3_armor_3_drops_to_zero() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        3,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 3);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 3));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "got hp.current == {}",
        hp.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        0,
        "3 - 3 = 0"
    );
}

// ── Behavior 19 ────────────────────────────────────────────────────────────

#[test]
fn breakthrough_preserves_damage_dealt_queue_entry() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 3);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 3));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    let matching: Vec<_> = collector.0.iter().filter(|d| d.target == cell).collect();
    assert_eq!(
        matching.len(),
        1,
        "breakthrough should preserve the DamageDealt entry, found {} matching",
        matching.len()
    );
    assert!(
        (matching[0].amount - 5.0).abs() < f32::EPSILON,
        "damage amount should be 5.0, got {}",
        matching[0].amount
    );
}

// ── Behavior 20 ────────────────────────────────────────────────────────────

#[test]
fn weak_face_hit_with_positive_piercing_consumes_nothing() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 5);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::Y, 5));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "weak face pass-through, got hp.current == {}",
        hp.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        5,
        "piercing NOT touched on weak face hit"
    );
}

// ── Behavior 20 edge: ArmorValue(3), same setup ───────────────────────────

#[test]
fn weak_face_hit_with_positive_piercing_consumes_nothing_armor_value_3() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        3,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 5);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::Y, 5));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "got hp.current == {}",
        hp.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        5,
        "piercing still 5 on weak face"
    );
}
