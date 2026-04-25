//! Group A — Builder and component attachment.
//!
//! Pure builder-terminal assertions. No plugin wiring, no state navigation —
//! just `spawn_cell_in_world` + `Cell::builder()` variants.

use bevy::prelude::*;

use crate::{
    cells::{
        components::{ArmorDirection, ArmorFacing, ArmorValue, ArmoredCell, VolatileCell},
        test_utils::spawn_cell_in_world,
    },
    prelude::*,
};

// ── Behavior 1 ─────────────────────────────────────────────────────────────

#[test]
fn armored_sugar_inserts_armored_cell_marker_and_value_and_default_bottom_facing() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .armored(2)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<ArmoredCell>(entity).is_some(),
        "entity should have ArmoredCell marker"
    );
    let value = world
        .get::<ArmorValue>(entity)
        .expect("entity should have ArmorValue");
    assert_eq!(value.0, 2);
    let facing = world
        .get::<ArmorFacing>(entity)
        .expect("entity should have ArmorFacing");
    assert_eq!(facing.0, ArmorDirection::Bottom);
}

// ── Behavior 1 edge: .armored(1) ──────────────────────────────────────────

#[test]
fn armored_sugar_value_1_inserts_armor_value_1_bottom() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .armored(1)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<ArmoredCell>(entity).is_some());
    assert_eq!(world.get::<ArmorValue>(entity).unwrap().0, 1);
    assert_eq!(
        world.get::<ArmorFacing>(entity).unwrap().0,
        ArmorDirection::Bottom
    );
}

// ── Behavior 1 edge: .armored(3) ──────────────────────────────────────────

#[test]
fn armored_sugar_value_3_inserts_armor_value_3_bottom() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .armored(3)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<ArmoredCell>(entity).is_some());
    assert_eq!(world.get::<ArmorValue>(entity).unwrap().0, 3);
    assert_eq!(
        world.get::<ArmorFacing>(entity).unwrap().0,
        ArmorDirection::Bottom
    );
}

// ── Behavior 2 ─────────────────────────────────────────────────────────────

#[test]
fn armored_facing_sugar_inserts_explicit_facing_top() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .armored_facing(1, ArmorDirection::Top)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<ArmoredCell>(entity).is_some());
    assert_eq!(world.get::<ArmorValue>(entity).unwrap().0, 1);
    assert_eq!(
        world.get::<ArmorFacing>(entity).unwrap().0,
        ArmorDirection::Top
    );
}

// ── Behavior 2 edge: Left ─────────────────────────────────────────────────

#[test]
fn armored_facing_sugar_value_2_left() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .armored_facing(2, ArmorDirection::Left)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert_eq!(world.get::<ArmorValue>(entity).unwrap().0, 2);
    assert_eq!(
        world.get::<ArmorFacing>(entity).unwrap().0,
        ArmorDirection::Left
    );
}

// ── Behavior 2 edge: Right ────────────────────────────────────────────────

#[test]
fn armored_facing_sugar_value_3_right() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .armored_facing(3, ArmorDirection::Right)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert_eq!(world.get::<ArmorValue>(entity).unwrap().0, 3);
    assert_eq!(
        world.get::<ArmorFacing>(entity).unwrap().0,
        ArmorDirection::Right
    );
}

// ── Behavior 2 edge: explicit Bottom matches .armored() default ───────────

#[test]
fn armored_facing_explicit_bottom_matches_armored_sugar_default() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .armored_facing(2, ArmorDirection::Bottom)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert_eq!(world.get::<ArmorValue>(entity).unwrap().0, 2);
    assert_eq!(
        world.get::<ArmorFacing>(entity).unwrap().0,
        ArmorDirection::Bottom
    );
}

// ── Behavior 3 ─────────────────────────────────────────────────────────────

#[test]
fn armor_direction_default_is_bottom() {
    assert_eq!(ArmorDirection::default(), ArmorDirection::Bottom);
}

// ── Behavior 3 edge: default propagates through newtype ───────────────────

#[test]
fn armor_facing_default_direction_propagates_through_newtype() {
    assert_eq!(
        ArmorFacing(ArmorDirection::default()).0,
        ArmorDirection::Bottom
    );
}

// ── Behavior 4 ─────────────────────────────────────────────────────────────

#[test]
fn cell_without_armored_sugar_has_no_armor_components() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<ArmoredCell>(entity).is_none());
    assert!(world.get::<ArmorValue>(entity).is_none());
    assert!(world.get::<ArmorFacing>(entity).is_none());

    // Guard: prove the builder actually ran
    let hp = world
        .get::<Hp>(entity)
        .expect("entity should have Hp from builder");
    assert!((hp.current - 20.0).abs() < f32::EPSILON);
}

// ── Behavior 4 edge: volatile cell has no armor components ────────────────

#[test]
fn volatile_cell_has_no_armor_components() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .volatile(10.0, 30.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<ArmoredCell>(entity).is_none());
    assert!(world.get::<ArmorValue>(entity).is_none());
    assert!(world.get::<ArmorFacing>(entity).is_none());

    // Guard: prove volatile marker was inserted
    assert!(
        world.get::<VolatileCell>(entity).is_some(),
        "volatile cell should have VolatileCell marker — builders must not cross-contaminate"
    );
}
