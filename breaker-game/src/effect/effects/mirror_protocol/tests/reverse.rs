//! Tests for `reverse()` no-op behavior.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, ExtraBolt, ImpactSide, LastImpact},
        definition::BoltDefinition,
        registry::BoltRegistry,
    },
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

// -- Behavior 12: reverse() is a noop --

#[test]
fn reverse_does_not_despawn_previously_spawned_mirrored_bolts() {
    let mut world = world_with_bolt_registry();
    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(60.0, 250.0)),
            Velocity2D(Vec2::new(100.0, 400.0)),
            LastImpact {
                position: Vec2::new(50.0, 200.0),
                side: ImpactSide::Top,
            },
        ))
        .id();

    fire(bolt_entity, true, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count_before = query.iter(&world).count();
    assert_eq!(count_before, 1, "should have 1 bolt before reverse");

    reverse(bolt_entity, true, "mirror_protocol", &mut world);

    let count_after = query.iter(&world).count();
    assert_eq!(
        count_before, count_after,
        "reverse should not despawn mirrored bolts"
    );
}

// -- Behavior 12 edge case: reverse() called without prior fire() does not panic --

#[test]
fn reverse_without_prior_fire_does_not_panic() {
    let mut world = world_with_bolt_registry();
    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(60.0, 250.0)),
            Velocity2D(Vec2::new(100.0, 400.0)),
            LastImpact {
                position: Vec2::new(50.0, 200.0),
                side: ImpactSide::Top,
            },
        ))
        .id();

    // Should not panic
    reverse(bolt_entity, true, "mirror_protocol", &mut world);
}
