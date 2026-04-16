//! Group D — `check_armor_direction` BLOCK path.
//!
//! Tests exercising the block branch (`piercing_remaining < armor_value` on
//! the armored face).

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::components::PiercingRemaining, cells::behaviors::armored::components::ArmorDirection,
    prelude::*,
};

// ── Behavior 14 ────────────────────────────────────────────────────────────

#[test]
fn armored_face_with_piercing_less_than_armor_value_blocks_damage() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 1);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 1));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "damage should be blocked, got hp.current == {}",
        hp.current
    );
    assert_eq!(
        app.world().get::<KilledBy>(cell).unwrap().dealer,
        None,
        "KilledBy.dealer should be None — apply_damage never wrote a dealer"
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        1,
        "piercing should NOT be consumed on a block"
    );
    assert!(app.world().get::<Dead>(cell).is_none());
}

// ── Behavior 14 edge: PiercingRemaining(0) with ArmorValue(2) ─────────────

#[test]
fn block_with_piercing_0_armor_value_2() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "blocked, got hp.current == {}",
        hp.current
    );
    assert_eq!(app.world().get::<PiercingRemaining>(bolt).unwrap().0, 0);
}

// ── Behavior 14 edge: PiercingRemaining(0) with ArmorValue(1) ─────────────

#[test]
fn block_with_piercing_0_armor_value_1() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        1,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "0 < 1, blocked, got hp.current == {}",
        hp.current
    );
    assert_eq!(app.world().get::<PiercingRemaining>(bolt).unwrap().0, 0);
}

// ── Behavior 14 edge: PiercingRemaining(2) with ArmorValue(3) ─────────────

#[test]
fn block_with_piercing_2_armor_value_3_saturating_non_consumption() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        3,
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
        (hp.current - 20.0).abs() < f32::EPSILON,
        "2 < 3, blocked, got hp.current == {}",
        hp.current
    );
    assert_eq!(
        app.world().get::<PiercingRemaining>(bolt).unwrap().0,
        2,
        "piercing stays at 2 on block"
    );
}

// ── Behavior 15 ────────────────────────────────────────────────────────────

#[test]
fn blocked_hit_removes_matching_damage_entry_from_queue() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    let matching = collector.0.iter().filter(|d| d.target == cell).count();
    assert_eq!(
        matching, 0,
        "blocked damage message should have been drained and not re-written, found {matching}"
    );
}

// ── Behavior 15 edge: total entries should be zero ────────────────────────

#[test]
fn blocked_hit_total_collector_entries_equals_zero() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        0,
        "only one seeded damage was blocked — total entries should be 0, found {}",
        collector.0.len()
    );
}

// ── Behavior 16 ────────────────────────────────────────────────────────────

#[test]
fn block_absorbs_would_be_lethal_hit_killed_by_dealer_unset() {
    let mut app = build_armored_test_app();

    let cell = spawn_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        5.0,
    );
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 999.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "armor absorbed the lethal hit, got hp.current == {}",
        hp.current
    );
    assert!(app.world().get::<Dead>(cell).is_none());
    assert_eq!(app.world().get::<KilledBy>(cell).unwrap().dealer, None);
    assert!(app.world().get_entity(cell).is_ok(), "cell still exists");
}

// ── Behavior 16 edge (sanity control): weak face lethal ───────────────────

#[test]
fn weak_face_lethal_hit_goes_through_sanity_control() {
    let mut app = build_armored_test_app();

    // ArmorFacing(Top) with impact_normal NEG_Y = Bottom face (weak)
    let cell = spawn_armored_cell(&mut app, Vec2::new(0.0, 0.0), 2, ArmorDirection::Top, 5.0);
    let bolt = spawn_test_bolt(&mut app, 0);
    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 999.0, bolt));

    advance_to_playing(&mut app);
    tick(&mut app);
    tick(&mut app);

    let is_dead_or_gone =
        app.world().get_entity(cell).is_err() || app.world().get::<Dead>(cell).is_some();
    assert!(
        is_dead_or_gone,
        "weak face lethal hit should destroy the cell — harness sanity control"
    );
}
