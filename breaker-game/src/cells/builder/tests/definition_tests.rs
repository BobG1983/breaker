//! Section D: .definition(&def) method
//! Section E: Override semantics (override > definition > default)

use bevy::{ecs::world::CommandQueue, prelude::*};

use crate::{
    cells::{
        builder::core::*,
        components::{
            Cell, CellDamageVisuals, CellHealth, CellTypeAlias, RegenRate, RequiredToClear,
        },
        definition::{CellBehavior, CellTypeDefinition},
    },
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, Target},
};

/// Creates a test `CellTypeDefinition` with known values.
fn test_cell_definition() -> CellTypeDefinition {
    CellTypeDefinition {
        id: "test".to_owned(),
        alias: "T".to_owned(),
        hp: 20.0,
        color_rgb: [1.0, 0.5, 0.2],
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.2,
        damage_blue_range: 0.4,
        damage_blue_base: 0.2,
        behaviors: None,

        effects: None,
    }
}

/// Spawns a cell via Commands backed by a `CommandQueue`, then applies the queue.
fn spawn_cell_in_world(
    world: &mut World,
    build_fn: impl FnOnce(&mut Commands) -> Entity,
) -> Entity {
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        build_fn(&mut commands)
    };
    queue.apply(world);
    entity
}

// ── Section D: .definition(&def) ────────────────────────────────────────────

// Behavior 13: .definition(&def) transitions Health dimension
#[test]
fn definition_transitions_health_to_has_health() {
    let def = test_cell_definition();
    let _builder: CellBuilder<NoPosition, NoDimensions, HasHealth, Unvisual> =
        Cell::builder().definition(&def);
    // Type annotation compiles — that is the assertion.
}

// Behavior 13 edge case: definition does NOT transition Position or Dimensions
#[test]
fn definition_does_not_transition_position_or_dimensions() {
    let def = test_cell_definition();
    // The builder still has NoPosition and NoDimensions after .definition()
    let _builder: CellBuilder<NoPosition, NoDimensions, HasHealth, Unvisual> =
        Cell::builder().definition(&def);
}

// Behavior 14: .definition(&def) stores hp from definition
#[test]
fn definition_stores_hp_from_definition() {
    let mut def = test_cell_definition();
    def.hp = 30.0;
    def.alias = "R".to_owned();
    def.color_rgb = [0.3, 4.0, 0.3];
    def.required_to_clear = true;
    def.damage_hdr_base = 4.0;
    def.damage_green_min = 0.4;
    def.damage_blue_range = 0.3;
    def.damage_blue_base = 0.1;
    def.behaviors = Some(vec![CellBehavior::Regen { rate: 2.0 }]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let health = world
        .get::<CellHealth>(entity)
        .expect("entity should have CellHealth");
    assert!(
        (health.current - 30.0).abs() < f32::EPSILON && (health.max - 30.0).abs() < f32::EPSILON,
        "CellHealth should be {{ current: 30.0, max: 30.0 }}, got {{ current: {}, max: {} }}",
        health.current,
        health.max
    );
}

// Behavior 14 edge case: definition with tiny hp
#[test]
fn definition_stores_tiny_hp() {
    let mut def = test_cell_definition();
    def.hp = 0.001;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let health = world
        .get::<CellHealth>(entity)
        .expect("entity should have CellHealth");
    assert!(
        (health.current - 0.001).abs() < f32::EPSILON && (health.max - 0.001).abs() < f32::EPSILON,
        "CellHealth should be {{ current: 0.001, max: 0.001 }}"
    );
}

// Behavior 16: .definition(&def) stores damage_visuals
#[test]
fn definition_stores_damage_visuals() {
    let mut def = test_cell_definition();
    def.damage_hdr_base = 4.0;
    def.damage_green_min = 0.4;
    def.damage_blue_range = 0.3;
    def.damage_blue_base = 0.1;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let visuals = world
        .get::<CellDamageVisuals>(entity)
        .expect("entity should have CellDamageVisuals");
    assert!(
        (visuals.hdr_base - 4.0).abs() < f32::EPSILON,
        "hdr_base should be 4.0, got {}",
        visuals.hdr_base
    );
    assert!(
        (visuals.green_min - 0.4).abs() < f32::EPSILON,
        "green_min should be 0.4"
    );
    assert!(
        (visuals.blue_range - 0.3).abs() < f32::EPSILON,
        "blue_range should be 0.3"
    );
    assert!(
        (visuals.blue_base - 0.1).abs() < f32::EPSILON,
        "blue_base should be 0.1"
    );
}

// Behavior 17: .definition(&def) stores required_to_clear
#[test]
fn definition_stores_required_to_clear_true() {
    let mut def = test_cell_definition();
    def.required_to_clear = true;

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
        world.get::<RequiredToClear>(entity).is_some(),
        "entity should have RequiredToClear marker"
    );
}

// Behavior 17 edge case: required_to_clear false means no marker
#[test]
fn definition_required_to_clear_false_has_no_marker() {
    let mut def = test_cell_definition();
    def.required_to_clear = false;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    // Guard: non-#[require] component ensures builder actually ran (hp from definition)
    let health = world
        .get::<CellHealth>(entity)
        .expect("entity should have CellHealth from definition");
    assert!(
        (health.current - 20.0).abs() < f32::EPSILON,
        "CellHealth should come from definition hp"
    );

    assert!(
        world.get::<RequiredToClear>(entity).is_none(),
        "entity should NOT have RequiredToClear when required_to_clear is false"
    );
}

// Behavior 18: .definition(&def) stores alias
#[test]
fn definition_stores_alias() {
    let mut def = test_cell_definition();
    def.alias = "R".to_owned();

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let alias = world
        .get::<CellTypeAlias>(entity)
        .expect("entity should have CellTypeAlias");
    assert_eq!(alias.0, "R", "CellTypeAlias should be 'R'");
}

// Behavior 18 edge case: multi-character alias
#[test]
fn definition_stores_multi_char_alias() {
    let mut def = test_cell_definition();
    def.alias = "Gu".to_owned();

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let alias = world
        .get::<CellTypeAlias>(entity)
        .expect("entity should have CellTypeAlias");
    assert_eq!(alias.0, "Gu", "CellTypeAlias should be 'Gu'");
}

// Behavior 19: .definition(&def) stores behaviors
#[test]
fn definition_stores_regen_behavior() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::Regen { rate: 2.0 }]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let regen = world
        .get::<RegenRate>(entity)
        .expect("entity should have RegenRate");
    assert!(
        (regen.0 - 2.0).abs() < f32::EPSILON,
        "RegenRate rate should be 2.0, got {}",
        regen.0
    );
}

// Behavior 19 edge case: behaviors None means no RegenRate
#[test]
fn definition_behaviors_none_has_no_regen() {
    let mut def = test_cell_definition();
    def.behaviors = None;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    // Guard: CellHealth ensures builder ran with definition hp
    let health = world
        .get::<CellHealth>(entity)
        .expect("entity should have CellHealth");
    assert!(
        (health.current - 20.0).abs() < f32::EPSILON,
        "CellHealth should come from definition"
    );

    assert!(
        world.get::<RegenRate>(entity).is_none(),
        "entity should NOT have RegenRate when behaviors is None"
    );
}

// Behavior 19 edge case: behaviors empty vec means no RegenRate
#[test]
fn definition_behaviors_empty_vec_has_no_regen() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    // Guard: CellHealth ensures builder ran with definition hp
    let health = world
        .get::<CellHealth>(entity)
        .expect("entity should have CellHealth");
    assert!(
        (health.current - 20.0).abs() < f32::EPSILON,
        "CellHealth should come from definition"
    );

    assert!(
        world.get::<RegenRate>(entity).is_none(),
        "entity should NOT have RegenRate when behaviors is Some(vec![])"
    );
}

// Behavior 20: .definition(&def) stores effects in optional data
#[test]
fn definition_stores_effects() {
    let mut def = test_cell_definition();
    def.effects = Some(vec![RootEffect::On {
        target: Target::Bolt,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(5.0))],
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

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects from definition effects");
    assert!(!bound.0.is_empty(), "BoundEffects should not be empty");
}

// Behavior 20 edge case: effects None means no effects stored
#[test]
fn definition_effects_none_has_no_bound_effects() {
    let mut def = test_cell_definition();
    def.effects = None;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    // Guard against false pass from stub
    assert!(
        world.get::<Cell>(entity).is_some(),
        "entity should have Cell marker from builder"
    );
    assert!(
        world.get::<BoundEffects>(entity).is_none(),
        "entity should NOT have BoundEffects when definition effects is None"
    );
}

// ── Section E: Override Semantics ───────────────────────────────────────────

// Behavior 21: .override_hp() after .definition() overrides definition hp
#[test]
fn override_hp_after_definition_overrides() {
    let mut def = test_cell_definition();
    def.hp = 30.0;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .override_hp(50.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let health = world
        .get::<CellHealth>(entity)
        .expect("entity should have CellHealth");
    assert!(
        (health.current - 50.0).abs() < f32::EPSILON && (health.max - 50.0).abs() < f32::EPSILON,
        "CellHealth should be {{ current: 50.0, max: 50.0 }} (override), got {{ current: {}, max: {} }}",
        health.current,
        health.max
    );
}

// Behavior 21 edge case: override to tiny hp
#[test]
fn override_hp_tiny_value() {
    let mut def = test_cell_definition();
    def.hp = 30.0;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .override_hp(0.001)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let health = world
        .get::<CellHealth>(entity)
        .expect("entity should have CellHealth");
    assert!(
        (health.current - 0.001).abs() < f32::EPSILON && (health.max - 0.001).abs() < f32::EPSILON,
        "CellHealth should be {{ current: 0.001, max: 0.001 }}"
    );
}

// Behavior 21b: .override_hp() after .hp() overrides explicit hp
#[test]
fn override_hp_after_explicit_hp_overrides() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .hp(10.0)
            .override_hp(50.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let health = world
        .get::<CellHealth>(entity)
        .expect("entity should have CellHealth");
    assert!(
        (health.current - 50.0).abs() < f32::EPSILON && (health.max - 50.0).abs() < f32::EPSILON,
        "CellHealth should be {{ current: 50.0, max: 50.0 }} (override), got {{ current: {}, max: {} }}",
        health.current,
        health.max
    );
}

// Behavior 21b edge case: override to smaller value
#[test]
fn override_hp_to_smaller_value() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .hp(100.0)
            .override_hp(1.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let health = world
        .get::<CellHealth>(entity)
        .expect("entity should have CellHealth");
    assert!(
        (health.current - 1.0).abs() < f32::EPSILON && (health.max - 1.0).abs() < f32::EPSILON,
        "CellHealth should be {{ current: 1.0, max: 1.0 }} (override to smaller)"
    );
}

// Behavior 22: Definition values propagate when no override is called
#[test]
fn definition_values_propagate_without_override() {
    let mut def = test_cell_definition();
    def.hp = 30.0;
    def.alias = "R".to_owned();
    def.color_rgb = [0.3, 4.0, 0.3];
    def.required_to_clear = true;
    def.damage_hdr_base = 4.0;
    def.damage_green_min = 0.4;
    def.damage_blue_range = 0.3;
    def.damage_blue_base = 0.1;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let health = world
        .get::<CellHealth>(entity)
        .expect("should have CellHealth");
    assert!(
        (health.current - 30.0).abs() < f32::EPSILON && (health.max - 30.0).abs() < f32::EPSILON,
        "CellHealth should be {{ current: 30.0, max: 30.0 }}"
    );

    let visuals = world
        .get::<CellDamageVisuals>(entity)
        .expect("should have CellDamageVisuals");
    assert!(
        (visuals.hdr_base - 4.0).abs() < f32::EPSILON,
        "hdr_base should be 4.0"
    );

    assert!(
        world.get::<RequiredToClear>(entity).is_some(),
        "should have RequiredToClear"
    );

    let alias = world
        .get::<CellTypeAlias>(entity)
        .expect("should have CellTypeAlias");
    assert_eq!(alias.0, "R");
}

// Behavior 22 edge case: required_to_clear false from definition
#[test]
fn definition_required_to_clear_false_no_marker() {
    let mut def = test_cell_definition();
    def.required_to_clear = false;

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
        world.get::<RequiredToClear>(entity).is_none(),
        "should NOT have RequiredToClear when definition has required_to_clear: false"
    );
}

// Behavior 23: Default values used when neither definition nor override is present
#[test]
fn no_definition_no_override_uses_defaults() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .hp(20.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    // Guard: CellHealth is non-#[require] — verifies builder populated the entity
    let health = world
        .get::<CellHealth>(entity)
        .expect("should have CellHealth from builder");
    assert!(
        (health.current - 20.0).abs() < f32::EPSILON && (health.max - 20.0).abs() < f32::EPSILON,
        "CellHealth should be {{ current: 20.0, max: 20.0 }}"
    );

    assert!(
        world.get::<CellDamageVisuals>(entity).is_none(),
        "should NOT have CellDamageVisuals without definition"
    );
    assert!(
        world.get::<CellTypeAlias>(entity).is_none(),
        "should NOT have CellTypeAlias without definition"
    );
    assert!(
        world.get::<RequiredToClear>(entity).is_none(),
        "should NOT have RequiredToClear without definition"
    );
    assert!(
        world.get::<RegenRate>(entity).is_none(),
        "should NOT have RegenRate without definition"
    );
}
