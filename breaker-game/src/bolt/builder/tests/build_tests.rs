use bevy::prelude::*;
use rantzsoft_lifecycle::CleanupOnExit;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    BaseSpeed, InterpolateTransform2D, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed,
    Position2D, PreviousPosition, PreviousScale, Scale2D, Spatial, Spatial2D, Velocity2D,
};

use crate::{
    bolt::{
        components::{Bolt, BoltRadius, BoltServing, BoltSpawnOffsetY, ExtraBolt, PrimaryBolt},
        definition::BoltDefinition,
    },
    shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, GameDrawLayer, WALL_LAYER},
    state::types::{NodeState, RunState},
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

// ── Section E: build() — Component Tuple Output ─────────────────────

// Behavior 19: build() on a primary serving bolt produces correct components
#[test]
fn build_primary_serving_has_bolt_marker() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<Bolt>(entity).is_some(),
        "entity should have Bolt marker"
    );
    assert!(
        world.get::<PrimaryBolt>(entity).is_some(),
        "entity should have PrimaryBolt marker"
    );
    assert!(
        world.get::<BoltServing>(entity).is_some(),
        "entity should have BoltServing marker"
    );
}

#[test]
fn build_primary_serving_has_spatial_markers() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<Spatial>(entity).is_some(),
        "entity should have Spatial marker"
    );
    assert!(
        world.get::<Spatial2D>(entity).is_some(),
        "entity should have Spatial2D (via Spatial #[require])"
    );
    assert!(
        world.get::<InterpolateTransform2D>(entity).is_some(),
        "entity should have InterpolateTransform2D (via Spatial #[require])"
    );
}

#[test]
fn build_primary_serving_has_position() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    let pos = world
        .get::<Position2D>(entity)
        .expect("entity should have Position2D");
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON && (pos.0.y - 50.0).abs() < f32::EPSILON,
        "Position2D should be (0.0, 50.0), got {:?}",
        pos.0
    );
    let prev_pos = world
        .get::<PreviousPosition>(entity)
        .expect("entity should have PreviousPosition");
    assert!(
        (prev_pos.0.x - 0.0).abs() < f32::EPSILON && (prev_pos.0.y - 50.0).abs() < f32::EPSILON,
        "PreviousPosition should be (0.0, 50.0), got {:?}",
        prev_pos.0
    );
}

#[test]
fn build_primary_serving_has_zero_velocity() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    // Guard against false pass from stub — check a non-#[require] component
    assert!(
        world.get::<PrimaryBolt>(entity).is_some(),
        "entity should have PrimaryBolt marker from builder"
    );
    let vel = world
        .get::<Velocity2D>(entity)
        .expect("entity should have Velocity2D");
    assert_eq!(
        vel.0,
        Vec2::ZERO,
        "Serving bolt should have Velocity2D(Vec2::ZERO)"
    );
}

#[test]
fn build_primary_serving_has_speed_components() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    let base = world.get::<BaseSpeed>(entity).unwrap();
    assert!(
        (base.0 - 400.0).abs() < f32::EPSILON,
        "BaseSpeed should be 400.0"
    );
    let min = world.get::<MinSpeed>(entity).unwrap();
    assert!(
        (min.0 - 200.0).abs() < f32::EPSILON,
        "MinSpeed should be 200.0"
    );
    let max = world.get::<MaxSpeed>(entity).unwrap();
    assert!(
        (max.0 - 800.0).abs() < f32::EPSILON,
        "MaxSpeed should be 800.0"
    );
}

#[test]
fn build_primary_serving_has_angle_components() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    let h = world.get::<MinAngleHorizontal>(entity).unwrap();
    let expected_h = 5.0_f32.to_radians();
    assert!(
        (h.0 - expected_h).abs() < 1e-5,
        "MinAngleHorizontal should be {expected_h}"
    );
    let v = world.get::<MinAngleVertical>(entity).unwrap();
    let expected_v = 5.0_f32.to_radians();
    assert!(
        (v.0 - expected_v).abs() < 1e-5,
        "MinAngleVertical should be {expected_v}"
    );
}

#[test]
fn build_primary_serving_has_radius_components() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 8.0).abs() < f32::EPSILON,
        "BoltRadius should be 8.0"
    );
    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON && (scale.y - 8.0).abs() < f32::EPSILON,
        "Scale2D should be (8.0, 8.0), got ({}, {})",
        scale.x,
        scale.y
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

#[test]
fn build_primary_serving_has_cleanup_on_run_end() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<CleanupOnExit<RunState>>(entity).is_some(),
        "Primary bolt should have CleanupOnExit<RunState>"
    );
}

#[test]
fn build_primary_serving_has_collision_layers() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    let layers = world
        .get::<CollisionLayers>(entity)
        .expect("entity should have CollisionLayers");
    assert_eq!(
        layers.membership, BOLT_LAYER,
        "membership should be BOLT_LAYER"
    );
    assert_eq!(
        layers.mask,
        CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
        "mask should be CELL|WALL|BREAKER"
    );
}

#[test]
fn build_primary_serving_headless_has_no_game_draw_layer() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<GameDrawLayer>(entity).is_none(),
        "headless bolt should NOT have GameDrawLayer"
    );
}

// Behavior 20: build() on an extra bolt with velocity produces correct components
#[test]
fn build_extra_velocity_has_extra_bolt_marker() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(200.0, 300.0))
        .with_velocity(Velocity2D(Vec2::new(102.9, 385.5)))
        .extra()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<Bolt>(entity).is_some(),
        "entity should have Bolt"
    );
    assert!(
        world.get::<ExtraBolt>(entity).is_some(),
        "entity should have ExtraBolt"
    );
    assert!(
        world.get::<PrimaryBolt>(entity).is_none(),
        "extra bolt should NOT have PrimaryBolt"
    );
    assert!(
        world.get::<BoltServing>(entity).is_none(),
        "velocity bolt should NOT have BoltServing"
    );
}

#[test]
fn build_extra_velocity_has_explicit_velocity() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(200.0, 300.0))
        .with_velocity(Velocity2D(Vec2::new(102.9, 385.5)))
        .extra()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    let vel = world.get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.0.x - 102.9).abs() < f32::EPSILON && (vel.0.y - 385.5).abs() < f32::EPSILON,
        "Velocity2D should be (102.9, 385.5), got {:?}",
        vel.0
    );
}

#[test]
fn build_extra_velocity_has_cleanup_on_node_exit() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(200.0, 300.0))
        .with_velocity(Velocity2D(Vec2::new(102.9, 385.5)))
        .extra()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<CleanupOnExit<NodeState>>(entity).is_some(),
        "Extra bolt should have CleanupOnExit<NodeState>"
    );
    assert!(
        world.get::<CleanupOnExit<RunState>>(entity).is_none(),
        "Extra bolt should NOT have CleanupOnExit<RunState>"
    );
}

#[test]
fn build_extra_velocity_has_spatial_markers() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(200.0, 300.0))
        .with_velocity(Velocity2D(Vec2::new(102.9, 385.5)))
        .extra()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<Spatial>(entity).is_some(),
        "entity should have Spatial"
    );
    assert!(
        world.get::<Spatial2D>(entity).is_some(),
        "entity should have Spatial2D"
    );
    assert!(
        world.get::<InterpolateTransform2D>(entity).is_some(),
        "entity should have InterpolateTransform2D"
    );
}

#[test]
fn build_extra_bolt_at_zero_pos_straight_up() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert_eq!(pos.0, Vec2::ZERO);
    let vel = world.get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.0.x - 0.0).abs() < f32::EPSILON && (vel.0.y - 400.0).abs() < f32::EPSILON,
        "Velocity should be (0.0, 400.0)"
    );
}

// Behavior 21: Serving bolt always has Velocity2D(Vec2::ZERO)
#[test]
fn serving_bolt_always_zero_velocity() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .with_speed(999.0, 100.0, 2000.0)
        .with_angle(0.0, 0.0)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    // Guard against false pass from stub — check non-#[require] component
    assert!(
        world.get::<BoltServing>(entity).is_some(),
        "entity should have BoltServing marker from builder"
    );
    let vel = world.get::<Velocity2D>(entity).unwrap();
    assert_eq!(
        vel.0,
        Vec2::ZERO,
        "Serving bolt Velocity2D should be Vec2::ZERO regardless of speed config"
    );
}

// Behavior 22: Primary bolt gets CleanupOnExit<RunState>, not CleanupOnExit<NodeState>
#[test]
fn primary_bolt_has_cleanup_on_run_end_not_node_exit() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<CleanupOnExit<RunState>>(entity).is_some(),
        "Primary bolt should have CleanupOnExit<RunState>"
    );
    assert!(
        world.get::<CleanupOnExit<NodeState>>(entity).is_none(),
        "Primary bolt should NOT have CleanupOnExit<NodeState>"
    );
}

// Behavior 23: Extra bolt gets CleanupOnExit<NodeState>, not CleanupOnExit<RunState>
#[test]
fn extra_bolt_has_cleanup_on_node_exit_not_run_end() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<CleanupOnExit<NodeState>>(entity).is_some(),
        "Extra bolt should have CleanupOnExit<NodeState>"
    );
    assert!(
        world.get::<CleanupOnExit<RunState>>(entity).is_none(),
        "Extra bolt should NOT have CleanupOnExit<RunState>"
    );
}

// Behavior 24: build() uses spatial builder internally
#[test]
fn build_uses_spatial_builder_for_velocity_constraints() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .with_speed(500.0, 100.0, 900.0)
        .with_angle(0.1, 0.2)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<Spatial>(entity).is_some(),
        "entity should have Spatial marker"
    );
    let base = world.get::<BaseSpeed>(entity).unwrap();
    assert!(
        (base.0 - 500.0).abs() < f32::EPSILON,
        "BaseSpeed should be 500.0"
    );
    let min = world.get::<MinSpeed>(entity).unwrap();
    assert!(
        (min.0 - 100.0).abs() < f32::EPSILON,
        "MinSpeed should be 100.0"
    );
    let max = world.get::<MaxSpeed>(entity).unwrap();
    assert!(
        (max.0 - 900.0).abs() < f32::EPSILON,
        "MaxSpeed should be 900.0"
    );
    let h = world.get::<MinAngleHorizontal>(entity).unwrap();
    assert!(
        (h.0 - 0.1).abs() < f32::EPSILON,
        "MinAngleHorizontal should be 0.1"
    );
    let v = world.get::<MinAngleVertical>(entity).unwrap();
    assert!(
        (v.0 - 0.2).abs() < f32::EPSILON,
        "MinAngleVertical should be 0.2"
    );
}

// Behavior 25: build() without config() still includes BoltRadius default
#[test]
fn build_without_from_config_has_default_radius() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 8.0).abs() < f32::EPSILON,
        "BoltRadius should default to 8.0 without config(), got {}",
        radius.0
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

#[test]
fn build_without_from_config_has_no_bolt_params() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<BoltSpawnOffsetY>(entity).is_none(),
        "Should NOT have BoltSpawnOffsetY without definition()"
    );
}

// Behavior 26: build() with .with_radius() override
#[test]
fn build_with_radius_override_sets_physical_dimensions() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_radius(20.0)
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 20.0).abs() < f32::EPSILON,
        "BoltRadius should be 20.0"
    );
    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 20.0).abs() < f32::EPSILON && (scale.y - 20.0).abs() < f32::EPSILON,
        "Scale2D should be (20.0, 20.0)"
    );
    let prev_scale = world.get::<PreviousScale>(entity).unwrap();
    assert!(
        (prev_scale.x - 20.0).abs() < f32::EPSILON && (prev_scale.y - 20.0).abs() < f32::EPSILON,
        "PreviousScale should be (20.0, 20.0)"
    );
    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        (aabb.half_extents.x - 20.0).abs() < f32::EPSILON
            && (aabb.half_extents.y - 20.0).abs() < f32::EPSILON,
        "Aabb2D half_extents should be (20.0, 20.0)"
    );
}

#[test]
fn build_with_radius_zero_no_panic() {
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_radius(0.0)
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        radius.0.abs() < f32::EPSILON,
        "BaseRadius(0.0) should be accepted"
    );
    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        scale.x.abs() < f32::EPSILON && scale.y.abs() < f32::EPSILON,
        "Scale2D should be (0.0, 0.0)"
    );
    let prev_scale = world.get::<PreviousScale>(entity).unwrap();
    assert!(
        prev_scale.x.abs() < f32::EPSILON && prev_scale.y.abs() < f32::EPSILON,
        "PreviousScale should be (0.0, 0.0)"
    );
    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        aabb.half_extents.x.abs() < f32::EPSILON && aabb.half_extents.y.abs() < f32::EPSILON,
        "Aabb2D half_extents should be (0.0, 0.0)"
    );
}
