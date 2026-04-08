use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    BaseSpeed, InterpolateTransform2D, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed,
    Position2D, PreviousScale, Scale2D, Spatial, Spatial2D, Velocity2D,
};

use crate::{
    bolt::{
        components::{Bolt, BoltRadius, BoltServing, BoltSpawnOffsetY, ExtraBolt, PrimaryBolt},
        definition::BoltDefinition,
    },
    shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, GameDrawLayer, WALL_LAYER},
};

fn spawn_in_world(world: &mut World, f: impl FnOnce(&mut Commands) -> Entity) -> Entity {
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        f(&mut commands)
    };
    queue.apply(world);
    entity
}

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

// ── Section H: Method Ordering Independence ─────────────────────────

// Behavior 34: Dimensions can be satisfied in any order
#[test]
fn dimensions_any_order_extra_velocity() {
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Bolt::builder()
            .extra()
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .with_angle(0.087, 0.087)
            .at_position(Vec2::new(50.0, 50.0))
            .with_speed(400.0, 200.0, 800.0)
            .headless()
            .spawn(commands)
    });

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON && (pos.0.y - 50.0).abs() < f32::EPSILON,
        "Position2D should be (50.0, 50.0)"
    );
    let vel = world.get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.0.x - 0.0).abs() < f32::EPSILON && (vel.0.y - 400.0).abs() < f32::EPSILON,
        "Velocity2D should be (0.0, 400.0)"
    );
    assert!(
        world.get::<ExtraBolt>(entity).is_some(),
        "should have ExtraBolt"
    );
    let base = world.get::<BaseSpeed>(entity).unwrap();
    assert!(
        (base.0 - 400.0).abs() < f32::EPSILON,
        "BaseSpeed should be 400.0"
    );
}

#[test]
fn from_config_in_middle_of_chain() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Bolt::builder()
            .primary()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<PrimaryBolt>(entity).is_some());
    assert!(world.get::<BoltServing>(entity).is_some());
    let base = world.get::<BaseSpeed>(entity).unwrap();
    assert!((base.0 - 400.0).abs() < f32::EPSILON);
}

// Behavior 35: Optional methods interleaved with dimension methods
#[test]
fn optional_interleaved_with_dimension_methods() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Bolt::builder()
            .spawned_by("test")
            .at_position(Vec2::ZERO)
            .with_lifespan(2.0)
            .definition(&def)
            .with_radius(10.0)
            .serving()
            .extra()
            .headless()
            .spawn(commands)
    });

    // Bolt params from config should be present
    assert!(world.get::<BoltRadius>(entity).is_some());
    assert!(world.get::<BaseSpeed>(entity).is_some());
    assert!(world.get::<BoltServing>(entity).is_some());
    assert!(world.get::<ExtraBolt>(entity).is_some());
}

// ── Section J: Default Collision Layers and Draw Layer ───────────────

// Behavior 38: All built bolts have correct CollisionLayers
#[test]
fn collision_layers_primary_bolt() {
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&test_bolt_definition())
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });

    let layers = world.get::<CollisionLayers>(entity).unwrap();
    assert_eq!(layers.membership, BOLT_LAYER);
    assert_eq!(layers.mask, CELL_LAYER | WALL_LAYER | BREAKER_LAYER);
}

#[test]
fn collision_layers_extra_bolt_same_primary() {
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&test_bolt_definition())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .headless()
            .spawn(commands)
    });

    let layers = world.get::<CollisionLayers>(entity).unwrap();
    assert_eq!(
        layers.membership, BOLT_LAYER,
        "extra bolt membership should be BOLT_LAYER"
    );
    assert_eq!(
        layers.mask,
        CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
        "extra bolt mask should be CELL|WALL|BREAKER"
    );
}

// Behavior 39: Headless bolts do NOT have GameDrawLayer::Bolt
#[test]
fn headless_primary_has_no_game_draw_layer() {
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&test_bolt_definition())
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<GameDrawLayer>(entity).is_none(),
        "headless bolt should NOT have GameDrawLayer"
    );
}

#[test]
fn headless_extra_has_no_game_draw_layer() {
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&test_bolt_definition())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<GameDrawLayer>(entity).is_none(),
        "headless bolt should NOT have GameDrawLayer"
    );
}

// ── Section K: Manual Path Round-Trip ────────────────────────────────

// Behavior 40: Manual speed + angle path produces correct spatial components
#[test]
fn manual_path_produces_correct_spatial_components() {
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(500.0, 250.0, 750.0)
            .with_angle(0.1, 0.15)
            .at_position(Vec2::new(10.0, 20.0))
            .with_velocity(Velocity2D(Vec2::new(200.0, 300.0)))
            .extra()
            .headless()
            .spawn(commands)
    });

    let base = world.get::<BaseSpeed>(entity).unwrap();
    assert!(
        (base.0 - 500.0).abs() < f32::EPSILON,
        "BaseSpeed should be 500.0"
    );
    let min = world.get::<MinSpeed>(entity).unwrap();
    assert!(
        (min.0 - 250.0).abs() < f32::EPSILON,
        "MinSpeed should be 250.0"
    );
    let max = world.get::<MaxSpeed>(entity).unwrap();
    assert!(
        (max.0 - 750.0).abs() < f32::EPSILON,
        "MaxSpeed should be 750.0"
    );
    let h = world.get::<MinAngleHorizontal>(entity).unwrap();
    assert!(
        (h.0 - 0.1).abs() < f32::EPSILON,
        "MinAngleHorizontal should be 0.1"
    );
    let v = world.get::<MinAngleVertical>(entity).unwrap();
    assert!(
        (v.0 - 0.15).abs() < f32::EPSILON,
        "MinAngleVertical should be 0.15"
    );
    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - 10.0).abs() < f32::EPSILON && (pos.0.y - 20.0).abs() < f32::EPSILON,
        "Position2D should be (10.0, 20.0)"
    );
    let vel = world.get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.0.x - 200.0).abs() < f32::EPSILON && (vel.0.y - 300.0).abs() < f32::EPSILON,
        "Velocity2D should be (200.0, 300.0)"
    );
    assert!(world.get::<Spatial>(entity).is_some());
    assert!(world.get::<Spatial2D>(entity).is_some());
    assert!(world.get::<InterpolateTransform2D>(entity).is_some());
}

#[test]
fn manual_path_has_no_config_bolt_params() {
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(500.0, 250.0, 750.0)
            .with_angle(0.1, 0.15)
            .at_position(Vec2::new(10.0, 20.0))
            .with_velocity(Velocity2D(Vec2::new(200.0, 300.0)))
            .extra()
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<BoltSpawnOffsetY>(entity).is_none());

    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 8.0).abs() < f32::EPSILON,
        "BoltRadius should default to 8.0"
    );
}

// Behavior 41: Manual path with .with_radius() sets physical dimensions
#[test]
fn manual_path_with_radius_sets_physical_dimensions() {
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_radius(15.0)
            .headless()
            .spawn(commands)
    });

    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 15.0).abs() < f32::EPSILON,
        "BoltRadius should be 15.0"
    );
    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 15.0).abs() < f32::EPSILON && (scale.y - 15.0).abs() < f32::EPSILON,
        "Scale2D should be (15.0, 15.0)"
    );
    let prev_scale = world.get::<PreviousScale>(entity).unwrap();
    assert!(
        (prev_scale.x - 15.0).abs() < f32::EPSILON && (prev_scale.y - 15.0).abs() < f32::EPSILON,
        "PreviousScale should be (15.0, 15.0)"
    );
    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        (aabb.half_extents.x - 15.0).abs() < f32::EPSILON
            && (aabb.half_extents.y - 15.0).abs() < f32::EPSILON,
        "Aabb2D half_extents should be (15.0, 15.0)"
    );
}

#[test]
fn manual_path_without_radius_uses_default() {
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });

    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 8.0).abs() < f32::EPSILON,
        "default BoltRadius should be 8.0"
    );
    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON && (scale.y - 8.0).abs() < f32::EPSILON,
        "Scale2D should be (8.0, 8.0)"
    );
    let prev_scale = world.get::<PreviousScale>(entity).unwrap();
    assert!(
        (prev_scale.x - 8.0).abs() < f32::EPSILON && (prev_scale.y - 8.0).abs() < f32::EPSILON,
        "PreviousScale should be (8.0, 8.0)"
    );
    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        (aabb.half_extents.x - 8.0).abs() < f32::EPSILON
            && (aabb.half_extents.y - 8.0).abs() < f32::EPSILON,
        "Aabb2D half_extents should be (8.0, 8.0)"
    );
}
