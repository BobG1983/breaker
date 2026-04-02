//! Tests for `fire()` effect inheritance (`BoundEffects` copying).

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, ExtraBolt, ImpactSide, LastImpact},
        definition::BoltDefinition,
        registry::BoltRegistry,
    },
    effect::{BoundEffects, EffectKind, EffectNode},
    shared::rng::GameRng,
};

fn world_with_bolt_registry() -> World {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Bolt".to_string(),
        BoltDefinition {
            name: "Bolt".to_owned(),
            base_speed: 400.0,
            min_speed: 200.0,
            max_speed: 800.0,
            radius: 8.0,
            base_damage: 10.0,
            effects: vec![],
            color_rgb: [6.0, 5.0, 0.5],
            min_angle_horizontal: 5.0,
            min_angle_vertical: 5.0,
            min_radius: None,
            max_radius: None,
        },
    );
    world.insert_resource(registry);
    world.insert_resource(GameRng::default());
    world
}

fn bolt_bundle() -> (Bolt, Position2D, Velocity2D, LastImpact) {
    (
        Bolt,
        Position2D(Vec2::new(60.0, 250.0)),
        Velocity2D(Vec2::new(100.0, 400.0)),
        LastImpact {
            position: Vec2::new(50.0, 200.0),
            side: ImpactSide::Top,
        },
    )
}

// -- Behavior 9: inherit=true copies BoundEffects from the bolt entity --

#[test]
fn fire_with_inherit_true_copies_bound_effects() {
    let mut world = world_with_bolt_registry();
    let bound = BoundEffects(vec![
        (
            "piercing_chip".to_string(),
            EffectNode::Do(EffectKind::Piercing(3)),
        ),
        (
            "speed_chip".to_string(),
            EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
        ),
    ]);
    let bolt_entity = world.spawn((bolt_bundle(), bound)).id();

    fire(bolt_entity, true, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolts: Vec<Entity> = query.iter(&world).collect();
    assert_eq!(bolts.len(), 1, "should spawn 1 bolt");

    let bolt = bolts[0];
    let effects = world
        .get::<BoundEffects>(bolt)
        .expect("spawned bolt should have BoundEffects when inherit=true");
    assert_eq!(
        effects.0.len(),
        2,
        "BoundEffects should have 2 entries, got {}",
        effects.0.len()
    );
    assert_eq!(effects.0[0].0, "piercing_chip");
    assert_eq!(effects.0[1].0, "speed_chip");

    // Source bolt's BoundEffects should be unchanged
    let source_effects = world
        .get::<BoundEffects>(bolt_entity)
        .expect("source bolt should still have BoundEffects");
    assert_eq!(
        source_effects.0.len(),
        2,
        "source bolt's BoundEffects should be unchanged"
    );
}

// -- Behavior 9 edge case: Bolt has BoundEffects with empty vec --

#[test]
fn fire_with_inherit_true_and_empty_bound_effects_spawns_bolt() {
    let mut world = world_with_bolt_registry();
    let bolt_entity = world.spawn((bolt_bundle(), BoundEffects::default())).id();

    fire(bolt_entity, true, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let effects = world.get::<BoundEffects>(bolt);
    // Either has BoundEffects with empty vec, or no BoundEffects at all -- both acceptable
    if let Some(effects) = effects {
        assert!(
            effects.0.is_empty(),
            "empty BoundEffects should produce empty BoundEffects on spawned bolt"
        );
    }
}

// -- Behavior 10: inherit=false does not copy BoundEffects --

#[test]
fn fire_with_inherit_false_does_not_copy_bound_effects() {
    let mut world = world_with_bolt_registry();
    let bound = BoundEffects(vec![(
        "chip".to_string(),
        EffectNode::Do(EffectKind::DamageBoost(2.0)),
    )]);
    let bolt_entity = world.spawn((bolt_bundle(), bound)).id();

    fire(bolt_entity, false, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let effects = world.get::<BoundEffects>(bolt);
    // Either no BoundEffects at all, or empty BoundEffects
    if let Some(effects) = effects {
        assert!(
            effects.0.is_empty(),
            "inherit=false should not copy BoundEffects to spawned bolt"
        );
    }
}

// -- Behavior 10 edge case: inherit=false with bolt having no BoundEffects component --

#[test]
fn fire_with_inherit_false_and_no_bound_effects_still_spawns_bolt() {
    let mut world = world_with_bolt_registry();
    let bolt_entity = world.spawn(bolt_bundle()).id();

    fire(bolt_entity, false, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(
        count, 1,
        "bolt should still be spawned with inherit=false and no BoundEffects"
    );
}

// -- Behavior 21: inherit=true with bolt having no BoundEffects component does not panic --

#[test]
fn fire_with_inherit_true_and_no_bound_effects_component_does_not_panic() {
    let mut world = world_with_bolt_registry();
    let bolt_entity = world.spawn(bolt_bundle()).id();

    // Should not panic
    fire(bolt_entity, true, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query
        .iter(&world)
        .next()
        .expect("bolt should still be spawned");

    // Should not have BoundEffects since source doesn't have them
    let effects = world.get::<BoundEffects>(bolt);
    assert!(
        effects.is_none() || effects.unwrap().0.is_empty(),
        "spawned bolt should not have BoundEffects when source bolt has none"
    );

    // Verify position is correct mirror position
    let pos = world
        .get::<Position2D>(bolt)
        .expect("bolt should have Position2D");
    assert_eq!(
        pos.0,
        Vec2::new(40.0, 250.0),
        "bolt should be at mirror position despite no BoundEffects"
    );
}
