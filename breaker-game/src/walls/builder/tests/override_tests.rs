use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use super::helpers::{custom_wall_definition, default_playfield};
use crate::walls::{components::Wall, definition::WallDefinition};

// ── Behavior 8: .with_half_thickness() stores override ──

#[test]
fn with_half_thickness_overrides_position() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = Wall::builder()
        .left(&pf)
        .with_half_thickness(60.0)
        .spawn(&mut world.commands());
    world.flush();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-460.0)).abs() < f32::EPSILON,
        "Position x should be -460.0 (playfield_left - 60.0), got {}",
        pos.0.x
    );

    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 60.0).abs() < f32::EPSILON,
        "Scale2D.x should be 60.0, got {}",
        scale.x
    );
    assert!(
        (scale.y - 300.0).abs() < f32::EPSILON,
        "Scale2D.y should be 300.0, got {}",
        scale.y
    );

    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        (aabb.half_extents.x - 60.0).abs() < f32::EPSILON,
        "Aabb2D.half_extents.x should be 60.0, got {}",
        aabb.half_extents.x
    );
    assert!(
        (aabb.half_extents.y - 300.0).abs() < f32::EPSILON,
        "Aabb2D.half_extents.y should be 300.0, got {}",
        aabb.half_extents.y
    );
}

#[test]
fn with_half_thickness_zero_produces_edge_position() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = Wall::builder()
        .left(&pf)
        .with_half_thickness(0.0)
        .spawn(&mut world.commands());
    world.flush();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-400.0)).abs() < f32::EPSILON,
        "Position x should be -400.0 with ht=0.0, got {}",
        pos.0.x
    );

    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        aabb.half_extents.x.abs() < f32::EPSILON,
        "Aabb2D.half_extents.x should be 0.0 with ht=0.0, got {}",
        aabb.half_extents.x
    );
}

// ── Behavior 9: .with_color() stores override ──

#[test]
fn with_color_stores_override_color() {
    let pf = default_playfield();
    let builder = Wall::builder().left(&pf).with_color([1.0, 0.5, 0.0]);
    assert_eq!(
        builder.optional.override_color_rgb,
        Some([1.0, 0.5, 0.0]),
        "override_color_rgb should be Some([1.0, 0.5, 0.0])"
    );
}

#[test]
fn with_color_hdr_stored_without_clamping() {
    let pf = default_playfield();
    let builder = Wall::builder().left(&pf).with_color([0.2, 2.0, 3.0]);
    assert_eq!(
        builder.optional.override_color_rgb,
        Some([0.2, 2.0, 3.0]),
        "HDR color should be stored without clamping"
    );
}

// ── Behavior 10: .with_effects() stores override ──

#[test]
fn with_effects_stores_override() {
    use ordered_float::OrderedFloat;

    use crate::effect_v3::{
        effects::SpeedBoostConfig,
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    };

    let pf = default_playfield();
    let root_node = RootNode::Stamp(
        StampTarget::ActiveWalls,
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        ),
    );

    let builder = Wall::builder().left(&pf).with_effects(vec![root_node]);
    assert!(
        builder.optional.override_effects.is_some(),
        "override_effects should be Some"
    );
    assert_eq!(builder.optional.override_effects.as_ref().unwrap().len(), 1);
}

#[test]
fn with_effects_empty_vec_stores_some_empty() {
    let pf = default_playfield();
    let builder = Wall::builder().left(&pf).with_effects(vec![]);
    assert_eq!(
        builder.optional.override_effects,
        Some(vec![]),
        ".with_effects(vec![]) should store Some(vec![]), not None"
    );
}

// ── Behavior 11: Override beats definition regardless of call order ──

#[test]
fn override_beats_definition_when_definition_first() {
    let pf = default_playfield();
    let def = WallDefinition {
        half_thickness: 45.0,
        ..Default::default()
    };
    let mut world = World::new();

    let entity = Wall::builder()
        .left(&pf)
        .definition(&def)
        .with_half_thickness(60.0)
        .spawn(&mut world.commands());
    world.flush();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-460.0)).abs() < f32::EPSILON,
        "Override 60.0 should win over definition 45.0: x = -460.0, got {}",
        pos.0.x
    );
}

#[test]
fn override_beats_definition_when_override_first() {
    let pf = default_playfield();
    let def = WallDefinition {
        half_thickness: 45.0,
        ..Default::default()
    };
    let mut world = World::new();

    let entity = Wall::builder()
        .left(&pf)
        .with_half_thickness(60.0)
        .definition(&def)
        .spawn(&mut world.commands());
    world.flush();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-460.0)).abs() < f32::EPSILON,
        "Override 60.0 should win even when definition called after: x = -460.0, got {}",
        pos.0.x
    );
}

#[test]
fn override_color_beats_definition_color() {
    let pf = default_playfield();
    let def = custom_wall_definition(); // color_rgb: Some([0.2, 2.0, 3.0])

    let builder = Wall::builder()
        .left(&pf)
        .with_color([1.0, 0.0, 0.0])
        .definition(&def);

    // Override should still be present
    assert_eq!(
        builder.optional.override_color_rgb,
        Some([1.0, 0.0, 0.0]),
        "Override color should persist after definition call"
    );
}
