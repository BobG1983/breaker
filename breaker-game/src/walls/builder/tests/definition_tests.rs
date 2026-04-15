use bevy::prelude::*;

use super::helpers::{custom_wall_definition, default_playfield, test_wall_definition};
use crate::{
    prelude::*,
    walls::{components::Wall, definition::WallDefinition},
};

// ── Behavior 6: .definition() stores definition values ──

#[test]
fn definition_stores_half_thickness_from_custom_definition() {
    let pf = default_playfield();
    let def = custom_wall_definition(); // half_thickness: 45.0

    let mut world = World::new();
    let entity = Wall::builder()
        .left(&pf)
        .definition(&def)
        .spawn(&mut world.commands());
    world.flush();

    // With ht = 45.0, Left position should be (-400.0 - 45.0, 0.0) = (-445.0, 0.0)
    let pos = world.get::<Position2D>(entity);
    assert!(pos.is_some(), "entity should have Position2D");
    let pos = pos.unwrap();
    assert!(
        (pos.0.x - (-445.0)).abs() < f32::EPSILON,
        "Position2D.x should be -445.0 (playfield_left - 45.0), got {}",
        pos.0.x
    );
    assert!(
        pos.0.y.abs() < f32::EPSILON,
        "Position2D.y should be 0.0, got {}",
        pos.0.y
    );
}

#[test]
fn definition_with_all_defaults_stores_default_half_thickness() {
    let pf = default_playfield();
    let def = test_wall_definition(); // half_thickness: 90.0, color_rgb: None, effects: []

    let builder = Wall::builder().left(&pf).definition(&def);

    // definition_half_thickness should be Some(90.0)
    assert!(
        builder.optional.definition_half_thickness.is_some(),
        "definition_half_thickness should be Some"
    );
    assert!(
        (builder.optional.definition_half_thickness.unwrap() - 90.0).abs() < f32::EPSILON,
        "definition_half_thickness should be 90.0"
    );
    // color_rgb: None in definition -> definition_color_rgb should be None
    assert!(
        builder.optional.definition_color_rgb.is_none(),
        "definition_color_rgb should be None for default definition"
    );
    // effects: [] -> definition_effects should be None (empty vec treated as absent)
    assert!(
        builder.optional.definition_effects.is_none(),
        "definition_effects should be None for empty effects vec"
    );
}

#[test]
fn definition_stores_color_rgb_from_custom_definition() {
    let pf = default_playfield();
    let def = custom_wall_definition(); // color_rgb: Some([0.2, 2.0, 3.0])

    let builder = Wall::builder().left(&pf).definition(&def);
    assert_eq!(
        builder.optional.definition_color_rgb,
        Some([0.2, 2.0, 3.0]),
        "definition_color_rgb should match custom definition"
    );
}

#[test]
fn definition_stores_effects_from_custom_definition() {
    let pf = default_playfield();
    let def = custom_wall_definition(); // effects: [On { target: Wall, ... }]

    let builder = Wall::builder().left(&pf).definition(&def);
    assert!(
        builder.optional.definition_effects.is_some(),
        "definition_effects should be Some for non-empty effects"
    );
    assert_eq!(
        builder.optional.definition_effects.as_ref().unwrap().len(),
        1,
        "definition_effects should have 1 root effect"
    );
}

// ── Behavior 7: .definition() with default half_thickness produces same position as no-definition ──

#[test]
fn definition_with_default_ht_produces_same_position_as_no_definition() {
    let pf = default_playfield();
    let def = WallDefinition::default(); // half_thickness: 90.0

    let mut world = World::new();

    // With definition
    let entity_with = Wall::builder()
        .left(&pf)
        .definition(&def)
        .spawn(&mut world.commands());
    world.flush();

    // Without definition
    let entity_without = Wall::builder().left(&pf).spawn(&mut world.commands());
    world.flush();

    let pos_with = world.get::<Position2D>(entity_with).unwrap();
    let pos_without = world.get::<Position2D>(entity_without).unwrap();

    assert!(
        (pos_with.0.x - (-490.0)).abs() < f32::EPSILON,
        "Position with default definition should be (-490.0, 0.0), got ({}, {})",
        pos_with.0.x,
        pos_with.0.y
    );
    assert!(
        (pos_with.0.x - pos_without.0.x).abs() < f32::EPSILON,
        "Position with default definition should match no-definition position"
    );
}

// ── Behavior 12: Resolution priority: override > definition > default (90.0) ──

#[test]
fn resolution_priority_no_definition_no_override_uses_default() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = Wall::builder().left(&pf).spawn(&mut world.commands());
    world.flush();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-490.0)).abs() < f32::EPSILON,
        "Position A (no def, no override) should use default 90.0: x = -490.0, got {}",
        pos.0.x
    );
}

#[test]
fn resolution_priority_definition_without_override() {
    let pf = default_playfield();
    let mut world = World::new();

    let def = WallDefinition {
        half_thickness: 45.0,
        ..Default::default()
    };
    let entity = Wall::builder()
        .left(&pf)
        .definition(&def)
        .spawn(&mut world.commands());
    world.flush();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-445.0)).abs() < f32::EPSILON,
        "Position B (def 45.0, no override) should be -445.0, got {}",
        pos.0.x
    );
}

#[test]
fn resolution_priority_override_beats_definition() {
    let pf = default_playfield();
    let mut world = World::new();

    let def = WallDefinition {
        half_thickness: 45.0,
        ..Default::default()
    };
    let entity = Wall::builder()
        .left(&pf)
        .definition(&def)
        .with_half_thickness(60.0)
        .spawn(&mut world.commands());
    world.flush();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-460.0)).abs() < f32::EPSILON,
        "Position C (def 45.0, override 60.0) should be -460.0, got {}",
        pos.0.x
    );
}

#[test]
fn resolution_priority_definition_with_default_ht_same_as_no_definition() {
    let pf = default_playfield();
    let mut world = World::new();

    let def = WallDefinition {
        half_thickness: 90.0,
        ..Default::default()
    };
    let entity = Wall::builder()
        .left(&pf)
        .definition(&def)
        .spawn(&mut world.commands());
    world.flush();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-490.0)).abs() < f32::EPSILON,
        "Definition with ht=90.0 should produce same as no definition: x = -490.0, got {}",
        pos.0.x
    );
}
