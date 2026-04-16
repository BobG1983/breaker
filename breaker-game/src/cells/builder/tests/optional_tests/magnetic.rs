//! Part E: Builder `.magnetic(radius, strength)` tests (behaviors 18-22).
//!
//! These tests exercise `.magnetic()`, `.with_behavior(CellBehavior::Magnetic)`,
//! and `.definition(&def)` against the `spawn_inner()` match arm. They assert
//! the `MagneticCell` marker and `MagneticField` components are inserted.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::{
        behaviors::magnetic::components::{MagneticCell, MagneticField},
        definition::CellBehavior,
    },
    prelude::*,
};

// ── Behavior 18: .magnetic() inserts MagneticCell marker ──

#[test]
fn spawn_with_magnetic_sugar_inserts_marker() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .magnetic(200.0, 1000.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<MagneticCell>(entity).is_some(),
        "entity should have MagneticCell marker"
    );
    // Guard: prove the builder ran (not a no-op stub)
    let hp = world
        .get::<Hp>(entity)
        .expect("entity should have Hp from builder");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Hp should be 20.0, got {}",
        hp.current
    );
}

// ── Behavior 19: .magnetic() inserts MagneticField with correct values ──

#[test]
fn spawn_with_magnetic_sugar_inserts_field_with_correct_values() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .magnetic(150.0, 500.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let field = world
        .get::<MagneticField>(entity)
        .expect("entity should have MagneticField");
    assert!(
        (field.radius - 150.0).abs() < f32::EPSILON,
        "MagneticField radius should be 150.0, got {}",
        field.radius
    );
    assert!(
        (field.strength - 500.0).abs() < f32::EPSILON,
        "MagneticField strength should be 500.0, got {}",
        field.strength
    );
}

// ── Behavior 20: Cell without .magnetic() has no magnetic components ──

#[test]
fn spawn_without_magnetic_has_no_magnetic_components() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<MagneticCell>(entity).is_none(),
        "entity without .magnetic() should not have MagneticCell"
    );
    assert!(
        world.get::<MagneticField>(entity).is_none(),
        "entity without .magnetic() should not have MagneticField"
    );
    // Guard: prove the builder ran
    let hp = world
        .get::<Hp>(entity)
        .expect("entity should have Hp from builder");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Hp should be 20.0, got {}",
        hp.current
    );
}

// ── Behavior 21: .with_behavior(CellBehavior::Magnetic) matches .magnetic() sugar ──

#[test]
fn spawn_with_behavior_magnetic_matches_magnetic_sugar() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Magnetic {
                radius:   200.0,
                strength: 1000.0,
            })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<MagneticCell>(entity).is_some(),
        "entity should have MagneticCell marker via with_behavior"
    );
    let field = world
        .get::<MagneticField>(entity)
        .expect("entity should have MagneticField via with_behavior");
    assert!(
        (field.radius - 200.0).abs() < f32::EPSILON,
        "MagneticField radius should be 200.0, got {}",
        field.radius
    );
    assert!(
        (field.strength - 1000.0).abs() < f32::EPSILON,
        "MagneticField strength should be 1000.0, got {}",
        field.strength
    );
}

// ── Behavior 22: Magnetic behavior through .definition() inserts components ──

#[test]
fn spawn_magnetic_through_definition_inserts_components() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::Magnetic {
        radius:   200.0,
        strength: 1000.0,
    }]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<MagneticCell>(entity).is_some(),
        "definition-sourced magnetic should insert MagneticCell marker"
    );
    let field = world
        .get::<MagneticField>(entity)
        .expect("definition-sourced magnetic should insert MagneticField");
    assert!(
        (field.radius - 200.0).abs() < f32::EPSILON,
        "MagneticField radius should be 200.0, got {}",
        field.radius
    );
    assert!(
        (field.strength - 1000.0).abs() < f32::EPSILON,
        "MagneticField strength should be 1000.0, got {}",
        field.strength
    );
}

#[test]
fn spawn_magnetic_with_regen_through_definition_inserts_both() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![
        CellBehavior::Magnetic {
            radius:   200.0,
            strength: 1000.0,
        },
        CellBehavior::Regen { rate: 2.0 },
    ]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<MagneticCell>(entity).is_some(),
        "entity should have MagneticCell"
    );
    assert!(
        world.get::<MagneticField>(entity).is_some(),
        "entity should have MagneticField"
    );
    let regen_rate = world
        .get::<RegenRate>(entity)
        .expect("entity should have RegenRate from Regen behavior");
    assert!(
        (regen_rate.0 - 2.0).abs() < f32::EPSILON,
        "RegenRate should be 2.0, got {}",
        regen_rate.0
    );
}
