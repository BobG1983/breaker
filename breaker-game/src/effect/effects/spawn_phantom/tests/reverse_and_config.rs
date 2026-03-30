//! Tests for `SpawnPhantom` `reverse()` no-op behavior and config edge cases.

use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use super::{super::effect::*, helpers::*};
use crate::{
    bolt::{components::BoltLifespan, resources::BoltConfig},
    shared::rng::GameRng,
};

#[test]
fn reverse_is_noop_phantoms_self_despawn() {
    let mut world = world_with_bolt_config();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(owner, 5.0, 10, "", &mut world);
    fire(owner, 5.0, 10, "", &mut world);

    reverse(owner, "", &mut world);

    // Phantoms should still exist -- they self-despawn via BoltLifespan
    let mut query = world.query::<&PhantomOwner>();
    let remaining = query.iter(&world).count();
    assert_eq!(
        remaining, 2,
        "reverse is no-op, phantoms persist until lifespan expires"
    );
}

#[test]
fn fire_short_duration_creates_valid_lifespan() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 0.01, 3, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<PhantomBoltMarker>>();
    let phantom = query.iter(&world).next().expect("phantom should exist");

    let lifespan = world
        .get::<BoltLifespan>(phantom)
        .expect("phantom should have BoltLifespan");
    assert!(
        (lifespan.0.duration().as_secs_f32() - 0.01).abs() < 0.001,
        "BoltLifespan should work with short duration 0.01, got {}",
        lifespan.0.duration().as_secs_f32()
    );
}

#[test]
fn fire_custom_radius_in_bolt_config() {
    let mut world = World::new();
    world.insert_resource(BoltConfig {
        radius: 6.0,
        ..BoltConfig::default()
    });
    world.insert_resource(GameRng::default());

    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 5.0, 3, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<PhantomBoltMarker>>();
    let phantom = query.iter(&world).next().expect("phantom should exist");

    let scale = world
        .get::<Scale2D>(phantom)
        .expect("phantom should have Scale2D");
    assert!(
        (scale.x - 6.0).abs() < f32::EPSILON,
        "Scale2D.x should use custom radius (6.0), got {}",
        scale.x
    );

    let aabb = world
        .get::<Aabb2D>(phantom)
        .expect("phantom should have Aabb2D");
    assert_eq!(
        aabb.half_extents,
        Vec2::new(6.0, 6.0),
        "Aabb2D half_extents should use custom radius (6.0), got {:?}",
        aabb.half_extents
    );
}
