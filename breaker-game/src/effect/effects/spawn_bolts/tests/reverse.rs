//! Tests for `reverse()` no-op behavior.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, ExtraBolt},
        resources::BoltConfig,
    },
    shared::rng::GameRng,
};

fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    world.insert_resource(GameRng::default());
    world
}

#[test]
fn reverse_does_not_despawn_previously_spawned_bolts() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 2, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count_before = query.iter(&world).count();

    reverse(entity, 2, None, false, "", &mut world);

    let count_after = query.iter(&world).count();
    assert_eq!(
        count_before, count_after,
        "reverse should not despawn spawned bolts"
    );
}

#[test]
fn reverse_with_no_prior_spawned_bolts_does_not_panic() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    // Should not panic
    reverse(entity, 2, None, false, "", &mut world);
}
