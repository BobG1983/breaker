//! Group C — Volatile behavior marker and stamp through the builder.
//!
//! These tests exercise `.volatile(damage, radius)`, `.with_behavior(CellBehavior::Volatile)`,
//! and `.definition(&def)` against the `spawn_inner()` match arm. They assert
//! the `VolatileCell` marker and the stamped `BoundEffects` entry — the
//! damage/radius values are carried only in the `BoundEffects` tree's
//! `ExplodeConfig`, which is the single source of truth for the detonation.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::helpers::*;
use crate::{
    cells::{
        behaviors::volatile::stamp::STAMP_SOURCE, components::VolatileCell,
        definition::CellBehavior,
    },
    effect_v3::{
        effects::ExplodeConfig,
        types::{EffectType, Tree, Trigger},
    },
    prelude::*,
};

// Behavior 15
#[test]
fn spawn_with_volatile_sugar_inserts_marker() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .volatile(25.0, 40.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<VolatileCell>(entity).is_some(),
        "entity should have VolatileCell marker"
    );
}

// Behavior 16
#[test]
fn spawn_with_volatile_sugar_stamps_single_bound_effects_entry_keyed_volatile() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .volatile(25.0, 40.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects from volatile stamp");
    assert_eq!(
        bound.0.len(),
        1,
        "exactly one bound effect should be stamped"
    );
    assert_eq!(bound.0[0].0, STAMP_SOURCE);

    let expected_tree = Tree::When(
        Trigger::Died,
        Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
            range:  OrderedFloat(40.0),
            damage: OrderedFloat(25.0),
        }))),
    );
    assert_eq!(bound.0[0].1, expected_tree);
}

// Behavior 16 edge
#[test]
fn spawn_with_volatile_sugar_small_values_stamp_matching_tree() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .volatile(0.001, 0.001)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects");
    let expected_tree = Tree::When(
        Trigger::Died,
        Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
            range:  OrderedFloat(0.001),
            damage: OrderedFloat(0.001),
        }))),
    );
    assert_eq!(bound.0[0].1, expected_tree);
}

// Behavior 17
#[test]
fn spawn_without_volatile_has_no_volatile_components_and_no_bound_effects() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<VolatileCell>(entity).is_none());
    assert!(world.get::<BoundEffects>(entity).is_none());

    // Guard: prove the builder actually ran (prevents false-pass under a no-op stub).
    let hp = world
        .get::<Hp>(entity)
        .expect("entity should have Hp from builder");
    assert!((hp.current - 20.0).abs() < f32::EPSILON);
}

// Behavior 18
#[test]
fn spawn_volatile_through_definition_inserts_markers_and_stamps_bound_effects() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: 25.0,
        radius: 40.0,
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

    assert!(world.get::<VolatileCell>(entity).is_some());

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("definition-sourced volatile should stamp BoundEffects");
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, STAMP_SOURCE);
    let expected_tree = Tree::When(
        Trigger::Died,
        Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
            range:  OrderedFloat(40.0),
            damage: OrderedFloat(25.0),
        }))),
    );
    assert_eq!(bound.0[0].1, expected_tree);
}

// Behavior 18 edge: regen + volatile through definition
#[test]
fn spawn_volatile_with_regen_through_definition_inserts_both_and_stamps_only_volatile() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![
        CellBehavior::Regen { rate: 2.0 },
        CellBehavior::Volatile {
            damage: 5.0,
            radius: 10.0,
        },
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

    let regen_rate = world
        .get::<RegenRate>(entity)
        .expect("entity should have RegenRate");
    assert!((regen_rate.0 - 2.0).abs() < f32::EPSILON);

    assert!(world.get::<VolatileCell>(entity).is_some());

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("volatile through definition should stamp BoundEffects");
    assert_eq!(
        bound.0.len(),
        1,
        "only volatile should contribute a BoundEffects entry (regen does not)"
    );
    assert_eq!(bound.0[0].0, STAMP_SOURCE);
}

// Behavior 19
#[test]
fn spawn_with_behavior_volatile_matches_volatile_sugar() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Volatile {
                damage: 25.0,
                radius: 40.0,
            })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<VolatileCell>(entity).is_some());

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects from volatile stamp");
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, STAMP_SOURCE);
    let expected_tree = Tree::When(
        Trigger::Died,
        Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
            range:  OrderedFloat(40.0),
            damage: OrderedFloat(25.0),
        }))),
    );
    assert_eq!(bound.0[0].1, expected_tree);
}

// Behavior 19 edge: duplicate with_behavior — match arm runs once per vec element
#[test]
fn spawn_with_duplicate_volatile_via_with_behavior_stamps_two_bound_effects() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Volatile {
                damage: 10.0,
                radius: 20.0,
            })
            .with_behavior(CellBehavior::Volatile {
                damage: 25.0,
                radius: 40.0,
            })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    // Two behavior entries mean two match-arm iterations.
    // StampEffectCommand APPENDS to BoundEffects — two stamps = two entries,
    // each carrying its own damage/radius in its ExplodeConfig.
    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        2,
        "two .with_behavior(Volatile) calls should append two BoundEffects entries"
    );
    assert_eq!(bound.0[0].0, STAMP_SOURCE);
    assert_eq!(bound.0[1].0, STAMP_SOURCE);

    let first_tree = Tree::When(
        Trigger::Died,
        Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
            range:  OrderedFloat(20.0),
            damage: OrderedFloat(10.0),
        }))),
    );
    let second_tree = Tree::When(
        Trigger::Died,
        Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
            range:  OrderedFloat(40.0),
            damage: OrderedFloat(25.0),
        }))),
    );
    assert_eq!(bound.0[0].1, first_tree);
    assert_eq!(bound.0[1].1, second_tree);
}
