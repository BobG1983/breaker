//! Section F: Optional Chainable Methods (any typestate)

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::{
        components::{Cell, CellDamageVisuals, CellTypeAlias, RequiredToClear},
        definition::CellBehavior,
    },
    effect_v3::{
        effects::DamageBoostConfig,
        types::{EffectType, RootNode, StampTarget, Tree},
    },
    prelude::*,
};

// Behavior 24: .with_behavior(CellBehavior::Regen { rate: 3.0 }) inserts RegenRate
#[test]
fn with_behavior_regen_inserts_cell_regen() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Regen { rate: 3.0 })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let regen = world
        .get::<RegenRate>(entity)
        .expect("entity should have RegenRate");
    assert!(
        (regen.0 - 3.0).abs() < f32::EPSILON,
        "RegenRate rate should be 3.0, got {}",
        regen.0
    );
}

// Behavior 24 edge case: calling .with_behavior() twice — last write wins
#[test]
fn with_behavior_twice_last_write_wins() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Regen { rate: 3.0 })
            .with_behavior(CellBehavior::Regen { rate: 1.0 })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let regen = world
        .get::<RegenRate>(entity)
        .expect("entity should have RegenRate");
    assert!(
        (regen.0 - 1.0).abs() < f32::EPSILON,
        "RegenRate rate should be 1.0 (last write wins), got {}",
        regen.0
    );
}

// Behavior 25: .with_behavior() combines with definition behaviors
#[test]
fn with_behavior_after_definition_explicit_wins() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::Regen { rate: 2.0 }]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .with_behavior(CellBehavior::Regen { rate: 5.0 })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let regen = world
        .get::<RegenRate>(entity)
        .expect("entity should have RegenRate");
    assert!(
        (regen.0 - 5.0).abs() < f32::EPSILON,
        "RegenRate rate should be 5.0 (explicit appended after definition, last write wins), got {}",
        regen.0
    );
}

// Behavior 25 edge case: definition behaviors None plus explicit
#[test]
fn with_behavior_without_definition_behaviors() {
    let mut def = test_cell_definition();
    def.behaviors = None;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .with_behavior(CellBehavior::Regen { rate: 3.0 })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let regen = world
        .get::<RegenRate>(entity)
        .expect("entity should have RegenRate");
    assert!(
        (regen.0 - 3.0).abs() < f32::EPSILON,
        "RegenRate rate should be 3.0 (explicit only), got {}",
        regen.0
    );
}

// Behavior 26: .with_effects(vec![root_effect]) sets effects
#[test]
fn with_effects_stores_effects_for_dispatch() {
    let root_node = RootNode::Stamp(
        StampTarget::Bolt,
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: ordered_float::OrderedFloat(5.0),
        })),
    );

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_effects(vec![root_node])
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects after effects dispatch");
    assert!(!bound.0.is_empty(), "BoundEffects should not be empty");
}

// Behavior 26 edge case: .with_effects(vec![]) stores empty effects
#[test]
fn with_effects_empty_vec_no_bound_effects() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_effects(vec![])
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    // Empty effects should be filtered — no BoundEffects inserted
    assert!(
        world.get::<BoundEffects>(entity).is_none(),
        "entity should NOT have BoundEffects when effects are empty (filtered)"
    );
}

// Behavior 27: .color_rgb() overrides definition color — tested in spawn_tests.rs (rendered)

// Behavior 28: .damage_visuals() overrides definition damage visuals
#[test]
fn damage_visuals_override_definition() {
    let mut def = test_cell_definition();
    def.damage_hdr_base = 4.0;
    def.damage_green_min = 0.4;
    def.damage_blue_range = 0.3;
    def.damage_blue_base = 0.1;

    let override_visuals = CellDamageVisuals {
        hdr_base:   8.0,
        green_min:  0.1,
        blue_range: 0.6,
        blue_base:  0.3,
    };

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .damage_visuals(override_visuals.clone())
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let visuals = world
        .get::<CellDamageVisuals>(entity)
        .expect("entity should have CellDamageVisuals");
    assert!(
        (visuals.hdr_base - 8.0).abs() < f32::EPSILON,
        "hdr_base should be 8.0 (override), got {}",
        visuals.hdr_base
    );
    assert!(
        (visuals.green_min - 0.1).abs() < f32::EPSILON,
        "green_min should be 0.1 (override)"
    );
    assert!(
        (visuals.blue_range - 0.6).abs() < f32::EPSILON,
        "blue_range should be 0.6 (override)"
    );
    assert!(
        (visuals.blue_base - 0.3).abs() < f32::EPSILON,
        "blue_base should be 0.3 (override)"
    );
}

// Behavior 28 edge case: .damage_visuals() without definition
#[test]
fn damage_visuals_without_definition_sets_directly() {
    let visuals = CellDamageVisuals {
        hdr_base:   8.0,
        green_min:  0.1,
        blue_range: 0.6,
        blue_base:  0.3,
    };

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .damage_visuals(visuals.clone())
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let found = world
        .get::<CellDamageVisuals>(entity)
        .expect("entity should have CellDamageVisuals");
    assert!(
        (found.hdr_base - 8.0).abs() < f32::EPSILON,
        "hdr_base should be 8.0"
    );
}

// Behavior 29: .alias() overrides definition alias
#[test]
fn alias_override_definition() {
    let mut def = test_cell_definition();
    def.alias = "R".to_owned();

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .alias("X".to_owned())
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let alias = world
        .get::<CellTypeAlias>(entity)
        .expect("entity should have CellTypeAlias");
    assert_eq!(alias.0, "X", "CellTypeAlias should be 'X' (override)");
}

// Behavior 29 edge case: .alias() without definition
#[test]
fn alias_without_definition_sets_directly() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .alias("test".to_owned())
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let alias = world
        .get::<CellTypeAlias>(entity)
        .expect("entity should have CellTypeAlias");
    assert_eq!(alias.0, "test", "CellTypeAlias should be 'test'");
}

// Behavior 30: .required_to_clear(true) overrides definition
#[test]
fn required_to_clear_true_overrides_false_definition() {
    let mut def = test_cell_definition();
    def.required_to_clear = false;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .required_to_clear(true)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<RequiredToClear>(entity).is_some(),
        "should have RequiredToClear (override true)"
    );
}

// Behavior 30 edge case: .required_to_clear(false) overrides definition true
#[test]
fn required_to_clear_false_overrides_true_definition() {
    let mut def = test_cell_definition();
    def.required_to_clear = true;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .required_to_clear(false)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<RequiredToClear>(entity).is_none(),
        "should NOT have RequiredToClear (override false)"
    );
}

// ── Part K: .toughness() chainable method ─────────────────────────────

use crate::cells::definition::Toughness;

// Behavior 32: .toughness() is chainable
#[test]
fn toughness_is_chainable() {
    let _builder = Cell::builder().toughness(Toughness::Tough);
    // Compiles — that is the assertion.
}

// Behavior 32 edge case: calling .toughness() multiple times — last one wins
#[test]
fn toughness_last_write_wins() {
    let _builder = Cell::builder()
        .toughness(Toughness::Weak)
        .toughness(Toughness::Tough);
    // The builder stores the last value internally.
    // Full verification happens when .tier_hp() reads it.
}

// Behavior 32 edge case: not calling .toughness() at all — legacy .hp() path still works
#[test]
fn no_toughness_hp_path_still_works() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });
    let health = world.get::<Hp>(entity).expect("should have Hp");
    assert!(
        (health.current - 20.0).abs() < f32::EPSILON,
        "legacy .hp() should still set health"
    );
}
