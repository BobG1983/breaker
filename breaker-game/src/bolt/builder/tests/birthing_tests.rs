use std::time::Duration;

use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::{PreviousScale, Scale2D, Velocity2D};

use crate::{
    bolt::{
        components::{Bolt, BoltServing, ExtraBolt, PrimaryBolt},
        definition::BoltDefinition,
    },
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, WALL_LAYER,
        birthing::{BIRTHING_DURATION, Birthing},
    },
};

/// Creates a `BoltDefinition` matching the values previously provided by
/// `BoltConfig::default()`, so existing assertions remain valid.
fn test_bolt_definition() -> BoltDefinition {
    BoltDefinition {
        name: "Bolt".to_string(),
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
    }
}

/// Spawns a bolt via Commands backed by a `CommandQueue`, then applies the queue.
/// Returns the Entity.
fn spawn_bolt_in_world(
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

// Behavior 10: .birthed() causes spawn to insert Birthing component
#[test]
fn birthed_inserts_birthing_component() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .birthed()
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<Birthing>(entity).is_some(),
        "Entity should have Birthing component after .birthed().spawn()"
    );
}

// Edge case: .birthed() called multiple times -- idempotent
#[test]
fn birthed_called_multiple_times_is_idempotent() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .birthed()
            .birthed()
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<Birthing>(entity).is_some(),
        "Entity should have exactly one Birthing component"
    );
}

// Behavior 11: .birthed() inserts Birthing with correct timer, target_scale, stashed_layers
#[test]
fn birthed_sets_correct_birthing_fields() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .birthed()
            .headless()
            .spawn(commands)
    });

    let birthing = world
        .get::<Birthing>(entity)
        .expect("entity should have Birthing");
    assert_eq!(
        birthing.timer.duration(),
        Duration::from_secs_f32(BIRTHING_DURATION),
        "Birthing timer duration should be {BIRTHING_DURATION}s"
    );
    assert!(
        (birthing.target_scale.x - 8.0).abs() < f32::EPSILON,
        "target_scale.x should be 8.0 (bolt radius), got {}",
        birthing.target_scale.x
    );
    assert!(
        (birthing.target_scale.y - 8.0).abs() < f32::EPSILON,
        "target_scale.y should be 8.0 (bolt radius), got {}",
        birthing.target_scale.y
    );
    assert_eq!(
        birthing.stashed_layers,
        CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
        "stashed_layers should be the bolt's normal collision layers"
    );
}

// Edge case: .birthed() with .with_radius(16.0) -- target_scale should be 16.0
#[test]
fn birthed_with_custom_radius_uses_custom_target_scale() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .with_radius(16.0)
            .birthed()
            .headless()
            .spawn(commands)
    });

    let birthing = world
        .get::<Birthing>(entity)
        .expect("entity should have Birthing");
    assert!(
        (birthing.target_scale.x - 16.0).abs() < f32::EPSILON,
        "target_scale.x should be 16.0 with custom radius, got {}",
        birthing.target_scale.x
    );
    assert!(
        (birthing.target_scale.y - 16.0).abs() < f32::EPSILON,
        "target_scale.y should be 16.0 with custom radius, got {}",
        birthing.target_scale.y
    );
}

// Behavior 12: .birthed() zeroes CollisionLayers on the spawned entity
#[test]
fn birthed_zeroes_collision_layers() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .birthed()
            .headless()
            .spawn(commands)
    });

    let layers = world
        .get::<CollisionLayers>(entity)
        .expect("entity should have CollisionLayers");
    assert_eq!(
        *layers,
        CollisionLayers::default(),
        "CollisionLayers should be zeroed (membership: 0, mask: 0)"
    );

    // Original layers are stashed in Birthing
    let birthing = world.get::<Birthing>(entity).unwrap();
    assert_eq!(
        birthing.stashed_layers,
        CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
        "stashed_layers should contain the original bolt layers"
    );
}

// Behavior 13: .birthed() sets Scale2D and PreviousScale to zero
#[test]
fn birthed_zeroes_scale_and_previous_scale() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .birthed()
            .headless()
            .spawn(commands)
    });

    let scale = world.get::<Scale2D>(entity).expect("should have Scale2D");
    assert!(
        scale.x.abs() < f32::EPSILON && scale.y.abs() < f32::EPSILON,
        "Scale2D should be (0.0, 0.0), got ({}, {})",
        scale.x,
        scale.y
    );

    let prev_scale = world
        .get::<PreviousScale>(entity)
        .expect("should have PreviousScale");
    assert!(
        prev_scale.x.abs() < f32::EPSILON && prev_scale.y.abs() < f32::EPSILON,
        "PreviousScale should be (0.0, 0.0), got ({}, {})",
        prev_scale.x,
        prev_scale.y
    );

    // Target scale is stored in Birthing
    let birthing = world.get::<Birthing>(entity).unwrap();
    assert!(
        (birthing.target_scale.x - 8.0).abs() < f32::EPSILON,
        "target_scale should store the original radius 8.0"
    );
}

// Behavior 14: Without .birthed(), no Birthing component is inserted
#[test]
fn without_birthed_no_birthing_component() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<Birthing>(entity).is_none(),
        "Entity should NOT have Birthing when .birthed() is not called"
    );

    let layers = world.get::<CollisionLayers>(entity).unwrap();
    assert_eq!(
        *layers,
        CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
        "CollisionLayers should be normal without .birthed()"
    );

    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON && (scale.y - 8.0).abs() < f32::EPSILON,
        "Scale2D should be (8.0, 8.0) without .birthed()"
    );
}

// Behavior 15: .birthed() is available in any typestate
#[test]
fn birthed_available_in_any_typestate() {
    // .birthed() called first -- before definition, position, etc.
    let _builder = Bolt::builder()
        .birthed()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .headless();

    // .birthed() called after position
    let _builder = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .birthed()
        .serving()
        .primary()
        .headless();

    // .birthed() called last (before terminal)
    let _builder = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .birthed()
        .headless();
}

// Behavior 16: .birthed() works with both primary and extra role bolts
#[test]
fn birthed_works_with_primary_and_extra_roles() {
    let mut world = World::new();
    let def = test_bolt_definition();

    // Primary bolt with .birthed()
    let primary = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .birthed()
            .headless()
            .spawn(commands)
    });

    // Extra bolt with .birthed()
    let extra = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .birthed()
            .headless()
            .spawn(commands)
    });

    // Both should have Birthing
    assert!(
        world.get::<Birthing>(primary).is_some(),
        "Primary bolt should have Birthing"
    );
    assert!(
        world.get::<Birthing>(extra).is_some(),
        "Extra bolt should have Birthing"
    );

    // Primary has PrimaryBolt marker
    assert!(
        world.get::<PrimaryBolt>(primary).is_some(),
        "Primary bolt should have PrimaryBolt marker"
    );

    // Extra has ExtraBolt marker
    assert!(
        world.get::<ExtraBolt>(extra).is_some(),
        "Extra bolt should have ExtraBolt marker"
    );

    // Both have zeroed CollisionLayers and Scale2D
    let layers_p = world.get::<CollisionLayers>(primary).unwrap();
    assert_eq!(*layers_p, CollisionLayers::default());
    let scale_p = world.get::<Scale2D>(primary).unwrap();
    assert!(scale_p.x.abs() < f32::EPSILON && scale_p.y.abs() < f32::EPSILON);

    let layers_e = world.get::<CollisionLayers>(extra).unwrap();
    assert_eq!(*layers_e, CollisionLayers::default());
    let scale_e = world.get::<Scale2D>(extra).unwrap();
    assert!(scale_e.x.abs() < f32::EPSILON && scale_e.y.abs() < f32::EPSILON);
}

// Behavior 17: .birthed() works with both serving and velocity motion modes
#[test]
fn birthed_works_with_serving_and_velocity_modes() {
    let mut world = World::new();
    let def = test_bolt_definition();

    // Serving bolt with .birthed()
    let serving = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .birthed()
            .headless()
            .spawn(commands)
    });

    // Velocity bolt with .birthed()
    let velocity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .birthed()
            .headless()
            .spawn(commands)
    });

    // Both should have Birthing
    assert!(
        world.get::<Birthing>(serving).is_some(),
        "Serving bolt should have Birthing"
    );
    assert!(
        world.get::<Birthing>(velocity).is_some(),
        "Velocity bolt should have Birthing"
    );

    // Serving bolt should have BoltServing
    assert!(
        world.get::<BoltServing>(serving).is_some(),
        "Serving bolt should have BoltServing marker"
    );

    // Velocity bolt should have Velocity2D
    let vel = world.get::<Velocity2D>(velocity).unwrap();
    assert!(
        (vel.0.x - 0.0).abs() < f32::EPSILON && (vel.0.y - 400.0).abs() < f32::EPSILON,
        "Velocity bolt should have Velocity2D(0.0, 400.0)"
    );

    // Both have zeroed CollisionLayers and Scale2D
    let layers_s = world.get::<CollisionLayers>(serving).unwrap();
    assert_eq!(*layers_s, CollisionLayers::default());
    let scale_s = world.get::<Scale2D>(serving).unwrap();
    assert!(scale_s.x.abs() < f32::EPSILON && scale_s.y.abs() < f32::EPSILON);

    let layers_v = world.get::<CollisionLayers>(velocity).unwrap();
    assert_eq!(*layers_v, CollisionLayers::default());
    let scale_v = world.get::<Scale2D>(velocity).unwrap();
    assert!(scale_v.x.abs() < f32::EPSILON && scale_v.y.abs() < f32::EPSILON);
}
