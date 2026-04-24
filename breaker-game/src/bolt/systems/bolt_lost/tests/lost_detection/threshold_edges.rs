use bevy::prelude::*;
use rantzsoft_spatial2d::components::Spatial2D;

use super::super::helpers::*;
use crate::{
    bolt::{
        components::{BoltAngleSpread, BoltSpawnOffsetY},
        resources::DEFAULT_BOLT_SPAWN_OFFSET_Y,
        test_utils::speed_stack,
    },
    prelude::*,
    shared::GameDrawLayer,
};

#[test]
fn respawn_with_zero_spread_launches_straight_up() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let def = crate::bolt::definition::BoltDefinition {
        min_angle_horizontal: 0.0,
        min_angle_vertical: 0.0,
        ..make_default_bolt_definition()
    };
    let entity = spawn_bolt_with_definition(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(100.0, -400.0),
        &def,
    );
    // Override angle spread to 0.0
    app.world_mut()
        .entity_mut(entity)
        .insert(BoltAngleSpread(0.0));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();

    assert!(
        vel.0.x.abs() < 0.01,
        "zero spread should launch straight up, got vx={:.3}",
        vel.0.x,
    );
}

#[test]
fn bolt_lost_zero_spawn_offset_respawns_at_breaker_y() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let entity = spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    // Override spawn offset to 0.0
    app.world_mut()
        .entity_mut(entity)
        .insert(BoltSpawnOffsetY(0.0));
    tick(&mut app);

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();

    assert!(
        (pos.0.y - (-250.0)).abs() < f32::EPSILON,
        "zero offset should respawn at breaker Y exactly (-250.0), got {}",
        pos.0.y
    );
}

// Behavior 35: LostBoltData query uses BoltAngleSpread and BoltSpawnOffsetY
// (This is implicitly tested by behaviors 33-34, but we also test the
// config-path fallback.)
#[test]
fn bolt_lost_definition_built_bolt_has_required_query_components() {
    // Verify that .definition()-built bolts have both BoltAngleSpread and BoltSpawnOffsetY
    let mut app = test_app();
    let entity = spawn_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    let world = app.world();
    assert!(
        world.get::<BoltAngleSpread>(entity).is_some(),
        "definition-built bolt should have BoltAngleSpread"
    );
    assert!(
        world.get::<BoltSpawnOffsetY>(entity).is_some(),
        "definition-built bolt should have BoltSpawnOffsetY"
    );
}

// Behavior 36: extra bolt is despawned (not respawned)
// Already tested in extra_bolt_tests.rs with .definition() migration

#[test]
fn bolt_lost_respawn_velocity_with_speed_boost() {
    // Edge case: EffectStack<SpeedBoostConfig> with 1.2 -> 720.0 * 1.2 = 864.0
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let entity = spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    app.world_mut()
        .entity_mut(entity)
        .insert(speed_stack(&[1.2]));
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    let speed = vel.speed();
    assert!(
        (speed - 864.0).abs() < 2.0,
        "respawn speed should be approximately 720.0 * 1.2 = 864.0, got {speed:.1}"
    );
}

// Behavior 39: bolt_lost respawns correctly using BoltAngleSpread and BoltSpawnOffsetY
// (BoltRespawnAngleSpread and BoltRespawnOffsetY were deleted in Wave 6)
#[test]
fn bolt_lost_works_without_old_respawn_components() {
    // Given: Bolt built via .definition(). Bolt is below floor.
    // Then: System runs without error. Bolt respawns using BoltAngleSpread and BoltSpawnOffsetY.
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let entity = spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );

    // System should run without error and respawn the bolt
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt should be respawned upward even without old respawn components"
    );

    let pos = app.world().get::<Position2D>(entity).unwrap();
    let expected_y = -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y;
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "respawn y should be {expected_y} (from BoltSpawnOffsetY), got {}",
        pos.0.y
    );
}
