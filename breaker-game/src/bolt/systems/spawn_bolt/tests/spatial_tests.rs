use bevy::prelude::*;
use rantzsoft_spatial2d::{
    components::{
        InterpolateTransform2D, Position2D, PreviousPosition, Rotation2D, Scale2D, Spatial2D,
    },
    draw_layer::DrawLayer,
};

use super::{super::*, helpers::*};
use crate::{
    bolt::{components::Bolt, resources::BoltConfig},
    breaker::{BreakerConfig, components::Breaker},
    shared::GameDrawLayer,
};

#[test]
fn spawn_bolt_creates_entity() {
    let mut app = test_app();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
    assert_eq!(count, 1);
}

#[test]
fn spawned_bolt_has_position2d_at_spawn_position() {
    // Given: no existing bolt, breaker at default y_position (-250.0),
    //        BoltConfig default spawn_offset_y = 30.0
    // When: spawn_bolt runs
    // Then: Bolt has Position2D(Vec2::new(0.0, -220.0))
    let mut app = test_app();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let position = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist with Position2D");
    let breaker_y = BreakerConfig::default().y_position; // -250.0
    let spawn_offset_y = BoltConfig::default().spawn_offset_y; // 30.0
    let expected = Vec2::new(0.0, breaker_y + spawn_offset_y); // (0.0, -220.0)
    assert!(
        (position.0.x - expected.x).abs() < f32::EPSILON
            && (position.0.y - expected.y).abs() < f32::EPSILON,
        "bolt Position2D should be {expected:?}, got {:?}",
        position.0,
    );
}

#[test]
fn spawned_bolt_has_position2d_without_breaker_entity() {
    // Edge case: no breaker entity — uses BreakerConfig default y_position
    let mut app = test_app();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let position = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist with Position2D even without breaker");
    let expected_y = BreakerConfig::default().y_position + BoltConfig::default().spawn_offset_y;
    assert!(
        (position.0.y - expected_y).abs() < f32::EPSILON,
        "bolt y should use BreakerConfig default, expected {expected_y}, got {}",
        position.0.y,
    );
}

#[test]
fn spawned_bolt_has_game_draw_layer_bolt() {
    // When: spawn_bolt runs
    // Then: Bolt has GameDrawLayer::Bolt with z() == 1.0
    let mut app = test_app();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    let layer = app
        .world()
        .get::<GameDrawLayer>(entity)
        .expect("bolt should have GameDrawLayer");
    assert!(
        (layer.z() - 1.0).abs() < f32::EPSILON,
        "GameDrawLayer::Bolt.z() should be 1.0, got {}",
        layer.z(),
    );
}

#[test]
fn spawned_bolt_has_spatial2d_and_interpolate_transform2d() {
    // When: spawn_bolt runs
    // Then: Bolt has Spatial2D and InterpolateTransform2D markers, plus
    //       required components Position2D, Rotation2D, Scale2D,
    //       PreviousPosition, PreviousRotation
    let mut app = test_app();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    let world = app.world();
    assert!(
        world.get::<Spatial2D>(entity).is_some(),
        "bolt should have Spatial2D marker"
    );
    assert!(
        world.get::<InterpolateTransform2D>(entity).is_some(),
        "bolt should have InterpolateTransform2D marker"
    );
    assert!(
        world.get::<Position2D>(entity).is_some(),
        "bolt should have Position2D (via Spatial2D #[require])"
    );
    assert!(
        world.get::<Rotation2D>(entity).is_some(),
        "bolt should have Rotation2D (via Spatial2D #[require])"
    );
    assert!(
        world.get::<Scale2D>(entity).is_some(),
        "bolt should have Scale2D (via Spatial2D #[require])"
    );
    assert!(
        world.get::<PreviousPosition>(entity).is_some(),
        "bolt should have PreviousPosition (via Spatial2D #[require])"
    );
}

#[test]
fn spawned_bolt_previous_position_matches_initial_position() {
    // Edge case: PreviousPosition.0 must match Position2D.0 to prevent
    // interpolation teleport on the first frame
    let mut app = test_app();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    let pos = app
        .world()
        .get::<Position2D>(entity)
        .expect("bolt should have Position2D");
    let prev = app
        .world()
        .get::<PreviousPosition>(entity)
        .expect("bolt should have PreviousPosition");
    assert_eq!(
        pos.0, prev.0,
        "PreviousPosition should match initial Position2D to prevent teleport"
    );
}

#[test]
fn spawned_bolt_has_scale2d_matching_radius() {
    // Given: BoltConfig default radius = 8.0 (from BoltDefaults)
    // When: spawn_bolt runs
    // Then: Scale2D { x: 8.0, y: 8.0 }
    let mut app = test_app();
    // Use radius = 6.0 from spec for concreteness
    app.world_mut().resource_mut::<BoltConfig>().radius = 6.0;
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let scale = app
        .world_mut()
        .query_filtered::<&Scale2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should have Scale2D");
    assert!(
        (scale.x - 6.0).abs() < f32::EPSILON && (scale.y - 6.0).abs() < f32::EPSILON,
        "Scale2D should be (6.0, 6.0), got ({}, {})",
        scale.x,
        scale.y,
    );
}

#[test]
fn bolt_spawns_above_moved_breaker() {
    // Given: breaker at (50.0, -100.0, 0.0), spawn_offset_y = 30.0
    // When: spawn_bolt runs
    // Then: Position2D(Vec2::new(50.0, -70.0))
    let moved_y = -100.0;
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(50.0, moved_y)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let config = BoltConfig::default();
    let position = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist with Position2D");
    let expected = Vec2::new(50.0, moved_y + config.spawn_offset_y);
    assert!(
        (position.0.x - expected.x).abs() < f32::EPSILON
            && (position.0.y - expected.y).abs() < f32::EPSILON,
        "bolt Position2D should be {expected:?}, got {:?}",
        position.0,
    );
}

#[test]
fn spawned_bolt_has_aabb2d_with_half_extents_matching_radius() {
    // Given: BoltConfig default radius = 8.0
    // When: spawn_bolt runs
    // Then: bolt entity has Aabb2D { center: Vec2::ZERO, half_extents: Vec2::new(8.0, 8.0) }
    use rantzsoft_physics2d::aabb::Aabb2D;

    let mut app = test_app();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    let aabb = app
        .world()
        .get::<Aabb2D>(entity)
        .expect("bolt should have Aabb2D");
    let config = BoltConfig::default();
    assert_eq!(
        aabb.center,
        Vec2::ZERO,
        "bolt Aabb2D center should be ZERO (local space)"
    );
    assert!(
        (aabb.half_extents.x - config.radius).abs() < f32::EPSILON
            && (aabb.half_extents.y - config.radius).abs() < f32::EPSILON,
        "bolt Aabb2D half_extents should be ({}, {}), got ({}, {})",
        config.radius,
        config.radius,
        aabb.half_extents.x,
        aabb.half_extents.y,
    );
}

#[test]
fn spawned_bolt_aabb2d_uses_configured_radius() {
    // Edge case: BoltConfig.radius = 6.0 → Aabb2D half_extents = (6.0, 6.0)
    use rantzsoft_physics2d::aabb::Aabb2D;

    let mut app = test_app();
    app.world_mut().resource_mut::<BoltConfig>().radius = 6.0;
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    let aabb = app
        .world()
        .get::<Aabb2D>(entity)
        .expect("bolt should have Aabb2D");
    assert_eq!(aabb.center, Vec2::ZERO);
    assert!(
        (aabb.half_extents.x - 6.0).abs() < f32::EPSILON
            && (aabb.half_extents.y - 6.0).abs() < f32::EPSILON,
        "bolt Aabb2D half_extents should be (6.0, 6.0), got ({}, {})",
        aabb.half_extents.x,
        aabb.half_extents.y,
    );
}

#[test]
fn spawned_bolt_has_collision_layers_bolt_membership_cell_wall_breaker_mask() {
    // Given: spawn_bolt runs
    // Then: CollisionLayers { membership: BOLT_LAYER (0x01), mask: CELL|WALL|BREAKER (0x0E) }
    use rantzsoft_physics2d::collision_layers::CollisionLayers;

    use crate::shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, WALL_LAYER};

    let mut app = test_app();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("bolt should have CollisionLayers");
    let expected_mask = CELL_LAYER | WALL_LAYER | BREAKER_LAYER;
    assert_eq!(
        layers.membership, BOLT_LAYER,
        "bolt membership should be BOLT_LAYER (0x{:02X}), got 0x{:02X}",
        BOLT_LAYER, layers.membership,
    );
    assert_eq!(
        layers.mask, expected_mask,
        "bolt mask should be CELL|WALL|BREAKER (0x{:02X}), got 0x{:02X}",
        expected_mask, layers.mask,
    );
}
