use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Velocity2D};

use super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, BoltLifespan, ExtraBolt, PiercingRemaining},
        resources::BoltConfig,
    },
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, CleanupOnRunEnd, WALL_LAYER,
        rng::GameRng,
    },
};

fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    world.insert_resource(GameRng::default());
    world
}

// -- fire tests ──────────────────────────────────────────────────

#[test]
fn fire_spawns_phantom_with_bolt_marker_and_physics_components() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(50.0, 100.0))).id();

    fire(entity, 5.0, 3, "", &mut world);

    // Query for the spawned phantom entity
    let mut query = world.query_filtered::<Entity, With<PhantomBoltMarker>>();
    let phantoms: Vec<Entity> = query.iter(&world).collect();
    assert_eq!(phantoms.len(), 1, "expected exactly one phantom");
    let phantom = phantoms[0];

    // Bolt marker
    assert!(
        world.get::<Bolt>(phantom).is_some(),
        "phantom should have Bolt component"
    );

    // ExtraBolt marker
    assert!(
        world.get::<ExtraBolt>(phantom).is_some(),
        "phantom should have ExtraBolt component"
    );

    // Position2D from owner
    let pos = world
        .get::<Position2D>(phantom)
        .expect("phantom should have Position2D");
    assert_eq!(
        pos.0,
        Vec2::new(50.0, 100.0),
        "phantom Position2D should match owner position"
    );

    // Velocity2D — magnitude at base_speed (direction is random via GameRng)
    let vel = world
        .get::<Velocity2D>(phantom)
        .expect("phantom should have Velocity2D");
    assert!(
        (vel.0.length() - 400.0).abs() < 1.0,
        "phantom velocity magnitude should be base_speed (400.0), got {}",
        vel.0.length()
    );

    // Scale2D
    let scale = world
        .get::<Scale2D>(phantom)
        .expect("phantom should have Scale2D");
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON,
        "phantom Scale2D.x should be radius (8.0), got {}",
        scale.x
    );
    assert!(
        (scale.y - 8.0).abs() < f32::EPSILON,
        "phantom Scale2D.y should be radius (8.0), got {}",
        scale.y
    );

    // Aabb2D
    let aabb = world
        .get::<Aabb2D>(phantom)
        .expect("phantom should have Aabb2D");
    assert_eq!(
        aabb.center,
        Vec2::ZERO,
        "phantom Aabb2D center should be Vec2::ZERO"
    );
    assert_eq!(
        aabb.half_extents,
        Vec2::new(8.0, 8.0),
        "phantom Aabb2D half_extents should be (8.0, 8.0)"
    );

    // CollisionLayers
    let layers = world
        .get::<CollisionLayers>(phantom)
        .expect("phantom should have CollisionLayers");
    assert_eq!(
        layers.membership, BOLT_LAYER,
        "phantom membership should be BOLT_LAYER"
    );
    assert_eq!(
        layers.mask,
        CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
        "phantom mask should be CELL|WALL|BREAKER"
    );
}

#[test]
fn fire_spawns_phantom_with_extra_bolt_marker() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 5.0, 3, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<PhantomBoltMarker>>();
    let phantom = query.iter(&world).next().expect("phantom should exist");
    assert!(
        world.get::<ExtraBolt>(phantom).is_some(),
        "phantom should have ExtraBolt component"
    );
}

#[test]
fn fire_spawns_phantom_with_bolt_lifespan() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 5.0, 3, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<PhantomBoltMarker>>();
    let phantom = query.iter(&world).next().expect("phantom should exist");

    // Should have BoltLifespan
    let lifespan = world.get::<BoltLifespan>(phantom);
    assert!(
        lifespan.is_some(),
        "phantom should have BoltLifespan component"
    );
    if let Some(lifespan) = lifespan {
        assert!(
            (lifespan.0.duration().as_secs_f32() - 5.0).abs() < 0.01,
            "BoltLifespan duration should be 5.0s, got {}",
            lifespan.0.duration().as_secs_f32()
        );
    }
}

#[test]
fn fire_spawns_phantom_with_infinite_piercing() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 5.0, 3, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<PhantomBoltMarker>>();
    let phantom = query.iter(&world).next().expect("phantom should exist");

    let piercing = world
        .get::<PiercingRemaining>(phantom)
        .expect("phantom should have PiercingRemaining");
    assert_eq!(
        piercing.0,
        u32::MAX,
        "phantom should have infinite piercing (u32::MAX), got {}",
        piercing.0
    );
}

#[test]
fn fire_spawns_phantom_with_cleanup_on_node_exit_not_run_end() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 5.0, 3, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<PhantomBoltMarker>>();
    let phantom = query.iter(&world).next().expect("phantom should exist");

    assert!(
        world.get::<CleanupOnNodeExit>(phantom).is_some(),
        "phantom should have CleanupOnNodeExit"
    );
    assert!(
        world.get::<CleanupOnRunEnd>(phantom).is_none(),
        "phantom should NOT have CleanupOnRunEnd"
    );
}

#[test]
fn fire_spawns_phantom_with_marker_and_owner() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(30.0, 40.0))).id();

    fire(entity, 5.0, 3, "", &mut world);

    let mut query = world.query::<(&PhantomBoltMarker, &PhantomOwner)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected exactly one phantom");

    let (_marker, owner) = results[0];
    assert_eq!(
        owner.0, entity,
        "PhantomOwner should reference the owner entity"
    );
}

#[test]
fn fire_spawns_phantom_with_velocity_magnitude_at_base_speed() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 5.0, 3, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<PhantomBoltMarker>>();
    let phantom = query.iter(&world).next().expect("phantom should exist");

    let vel = world
        .get::<Velocity2D>(phantom)
        .expect("phantom should have Velocity2D");
    assert!(
        (vel.0.length() - 400.0).abs() < 1.0,
        "phantom velocity magnitude should be base_speed (400.0), got {}",
        vel.0.length()
    );
}

#[test]
fn fire_spawns_phantom_with_custom_base_speed() {
    let mut world = World::new();
    world.insert_resource(BoltConfig {
        base_speed: 600.0,
        ..BoltConfig::default()
    });
    world.insert_resource(GameRng::default());

    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 5.0, 3, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<PhantomBoltMarker>>();
    let phantom = query.iter(&world).next().expect("phantom should exist");

    let vel = world
        .get::<Velocity2D>(phantom)
        .expect("phantom should have Velocity2D");
    assert!(
        (vel.0.length() - 600.0).abs() < 1.0,
        "phantom velocity magnitude should use custom base_speed (600.0), got {}",
        vel.0.length()
    );
}

#[test]
fn fire_enforces_max_active_cap() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    // Spawn 4 phantoms with max_active=2
    fire(entity, 5.0, 2, "", &mut world);
    fire(entity, 5.0, 2, "", &mut world);
    fire(entity, 5.0, 2, "", &mut world);
    fire(entity, 5.0, 2, "", &mut world);

    let mut query = world.query::<&PhantomBoltMarker>();
    let count = query.iter(&world).count();
    assert_eq!(
        count, 2,
        "should enforce max of 2 active phantoms, got {count}"
    );

    // Each surviving phantom should have full bolt physics components
    let mut bolt_query = world.query_filtered::<Entity, (With<PhantomBoltMarker>, With<Bolt>)>();
    let bolt_count = bolt_query.iter(&world).count();
    assert_eq!(
        bolt_count, 2,
        "surviving phantoms should have Bolt component, got {bolt_count}"
    );
}

#[test]
fn fire_max_active_one_replaces_previous() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 5.0, 1, "", &mut world);
    fire(entity, 5.0, 1, "", &mut world);

    let mut query = world.query::<&PhantomBoltMarker>();
    let count = query.iter(&world).count();
    assert_eq!(
        count, 1,
        "max_active=1 should despawn previous phantom, got {count}"
    );
}

#[test]
fn reverse_is_noop_phantoms_self_despawn() {
    let mut world = world_with_bolt_config();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(owner, 5.0, 10, "", &mut world);
    fire(owner, 5.0, 10, "", &mut world);

    reverse(owner, "", &mut world);

    // Phantoms should still exist — they self-despawn via BoltLifespan
    let mut query = world.query::<&PhantomOwner>();
    let remaining = query.iter(&world).count();
    assert_eq!(
        remaining, 2,
        "reverse is no-op, phantoms persist until lifespan expires"
    );
}

#[test]
fn fire_reads_position_from_position2d_not_transform() {
    let mut world = world_with_bolt_config();
    let entity = world
        .spawn((
            Position2D(Vec2::new(50.0, 75.0)),
            Transform::from_xyz(999.0, 999.0, 0.0),
        ))
        .id();

    fire(entity, 5.0, 3, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<PhantomBoltMarker>>();
    let phantom = query.iter(&world).next().expect("phantom should exist");

    let pos = world
        .get::<Position2D>(phantom)
        .expect("phantom should have Position2D");
    assert_eq!(
        pos.0,
        Vec2::new(50.0, 75.0),
        "phantom should use Position2D (50, 75) not Transform (999, 999), got {:?}",
        pos.0
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
