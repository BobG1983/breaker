use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, Velocity2D};

use super::helpers::*;
use crate::{
    bolt::{
        components::{Bolt, BoltAngleSpread, BoltSpawnOffsetY},
        messages::BoltLost,
        resources::{DEFAULT_BOLT_ANGLE_SPREAD, DEFAULT_BOLT_SPAWN_OFFSET_Y},
        systems::bolt_lost::system::bolt_lost,
    },
    breaker::components::Breaker,
    shared::{GameDrawLayer, NodeScalingFactor, PlayfieldConfig, birthing::Birthing},
};

#[test]
fn bolt_below_floor_detected_via_position2d() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.y > 0.0, "bolt should be relaunched upward");
}

#[test]
fn respawn_inserts_position2d_at_breaker_x() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    let breaker_x = 42.0;
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(breaker_x, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(
        &mut app,
        Vec2::new(200.0, playfield.bottom() - 100.0),
        Vec2::new(100.0, -400.0),
    );
    tick(&mut app);

    let (vel, pos) = app
        .world_mut()
        .query::<(&Velocity2D, &Position2D)>()
        .iter(app.world())
        .next()
        .unwrap();

    let speed = vel.0.length();
    assert!(
        (speed - 720.0).abs() < 2.0,
        "respawn speed should equal base_speed 720.0, got {speed:.1}",
    );

    let angle = vel.0.x.atan2(vel.0.y).abs();
    assert!(
        angle <= DEFAULT_BOLT_ANGLE_SPREAD + 0.01,
        "respawn angle {angle:.3} rad should be within spread {DEFAULT_BOLT_ANGLE_SPREAD:.3} rad",
    );

    assert!(vel.0.y > 0.0, "respawn should launch upward");

    assert!(
        (pos.0.x - breaker_x).abs() < f32::EPSILON,
        "respawn Position2D.0.x should match breaker X {breaker_x:.0}, got {:.1}",
        pos.0.x,
    );
}

#[test]
fn respawn_with_zero_spread_launches_straight_up() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
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
fn respawn_position2d_y_uses_spawn_offset() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    let breaker_y = -250.0;
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, breaker_y)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();

    let expected_y = breaker_y + DEFAULT_BOLT_SPAWN_OFFSET_Y;
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "respawn Position2D.0.y should be breaker_y + spawn_offset_y ({expected_y}), got {}",
        pos.0.y,
    );
}

#[test]
fn respawn_inserts_previous_position_matching_position2d() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    let breaker_x = 42.0;
    let breaker_y = -250.0;
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(breaker_x, breaker_y)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let (pos, prev_pos) = app
        .world_mut()
        .query_filtered::<(&Position2D, &PreviousPosition), With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();

    let expected = Vec2::new(breaker_x, breaker_y + DEFAULT_BOLT_SPAWN_OFFSET_Y);
    assert!(
        (pos.0 - expected).length() < f32::EPSILON,
        "respawn Position2D should be ({expected:?}), got {:?}",
        pos.0,
    );
    assert!(
        (prev_pos.0 - expected).length() < f32::EPSILON,
        "respawn PreviousPosition should match Position2D ({expected:?}), got {:?}",
        prev_pos.0,
    );
}

#[test]
fn bolt_above_floor_not_lost() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(&mut app, Vec2::new(0.0, 100.0), Vec2::new(100.0, -200.0));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.y < 0.0, "bolt above floor should keep going down");
}

// --- NodeScalingFactor lost detection tests ---

#[test]
fn scaled_bolt_uses_effective_radius_for_lost_detection() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));

    // Effective radius = 14.0 * 0.5 = 7.0, threshold = -300.0 - 7.0 = -307.0
    // Bolt must be below -307.0 to be detected as lost
    let bolt_y = playfield.bottom() - 7.0 - 1.0; // -308.0
    let entity = spawn_bolt(&mut app, Vec2::new(0.0, bolt_y), Vec2::new(0.0, -400.0));
    app.world_mut()
        .entity_mut(entity)
        .insert(NodeScalingFactor(0.5));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "scaled bolt below effective threshold should be respawned (vy > 0), got vy={:.1}",
        vel.0.y
    );
}

#[test]
fn bolt_without_entity_scale_in_lost_detection_is_backward_compatible() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));

    // No NodeScalingFactor
    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt without NodeScalingFactor should be respawned normally, got vy={:.1}",
        vel.0.y
    );
}

// ── Migration tests: Behaviors 33-39 ──

// Behavior 33: bolt_lost queries BoltAngleSpread instead of BoltRespawnAngleSpread
#[test]
fn bolt_lost_queries_bolt_angle_spread_for_respawn() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let entity = spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );

    // Verify the entity has BoltAngleSpread (from .definition())
    assert!(
        app.world().get::<BoltAngleSpread>(entity).is_some(),
        "definition-built bolt should have BoltAngleSpread"
    );

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(vel.0.y > 0.0, "bolt should be respawned upward");

    let angle = vel.0.x.atan2(vel.0.y).abs();
    assert!(
        angle <= DEFAULT_BOLT_ANGLE_SPREAD + 0.01,
        "respawn angle {angle:.3} should be within BoltAngleSpread ({DEFAULT_BOLT_ANGLE_SPREAD:.3})"
    );
}

// Behavior 34: bolt_lost queries BoltSpawnOffsetY instead of BoltRespawnOffsetY
#[test]
fn bolt_lost_queries_bolt_spawn_offset_y_for_respawn() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(42.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();

    let expected = Vec2::new(42.0, -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y);
    assert!(
        (pos.0.x - expected.x).abs() < f32::EPSILON,
        "respawn x should be {}, got {}",
        expected.x,
        pos.0.x
    );
    assert!(
        (pos.0.y - expected.y).abs() < f32::EPSILON,
        "respawn y should be {} (from BoltSpawnOffsetY {DEFAULT_BOLT_SPAWN_OFFSET_Y}), got {}",
        expected.y,
        pos.0.y
    );
}

#[test]
fn bolt_lost_zero_spawn_offset_respawns_at_breaker_y() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
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

// Behavior 37: bolt_lost respawn velocity uses base_speed via velocity formula
#[test]
fn bolt_lost_respawn_velocity_uses_base_speed() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();

    let speed = vel.speed();
    assert!(
        (speed - 720.0).abs() < 2.0,
        "respawn speed should be approximately 720.0 (BaseSpeed from definition), got {speed:.1}"
    );
}

#[test]
fn bolt_lost_respawn_velocity_with_speed_boost() {
    // Edge case: EffectStack<SpeedBoostConfig> with 1.2 -> 720.0 * 1.2 = 864.0
    use crate::bolt::test_utils::speed_stack;

    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
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

// Behavior 38: bolt_lost respawn inserts PreviousPosition matching new position
#[test]
fn bolt_lost_respawn_previous_position_matches_new_position() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let entity = spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    let prev = app.world().get::<PreviousPosition>(entity).unwrap();
    let expected = Vec2::new(0.0, -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y);

    assert!(
        (pos.0 - expected).length() < f32::EPSILON,
        "Position2D should be {expected:?}, got {:?}",
        pos.0
    );
    assert!(
        (prev.0 - expected).length() < f32::EPSILON,
        "PreviousPosition should match Position2D at {expected:?}, got {:?}",
        prev.0
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
        rantzsoft_spatial2d::components::Spatial2D,
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

// ── Birthing: respawned bolt should enter birthing animation ──

#[test]
fn bolt_lost_respawn_inserts_birthing_component() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let entity = spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    // After respawn, bolt should have Birthing component
    assert!(
        app.world().get::<Birthing>(entity).is_some(),
        "respawned bolt must have Birthing component for scale-up animation"
    );
}

// ── BoltLost message carries entity fields ──

#[test]
fn bolt_lost_sends_correct_bolt_and_breaker_entities_for_baseline() {
    use crate::shared::test_utils::TestAppBuilder;

    let mut app = TestAppBuilder::new()
        .with_playfield()
        .with_resource::<crate::shared::GameRng>()
        .with_message::<BoltLost>()
        .with_system(FixedUpdate, (bolt_lost, capture_bolt_lost.after(bolt_lost)))
        .build();

    app.init_resource::<CapturedBoltLost>();

    let playfield = PlayfieldConfig::default();
    let breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -250.0)),
            rantzsoft_spatial2d::components::Spatial2D,
            GameDrawLayer::Breaker,
        ))
        .id();

    let bolt_entity = spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let captured = app.world().resource::<CapturedBoltLost>();
    assert_eq!(
        captured.0.len(),
        1,
        "exactly one BoltLost message should be captured"
    );
    assert_eq!(
        captured.0[0].bolt, bolt_entity,
        "BoltLost.bolt should equal the bolt entity"
    );
    assert_eq!(
        captured.0[0].breaker, breaker_entity,
        "BoltLost.breaker should equal the breaker entity"
    );

    // Baseline bolt should still be alive (respawned, not despawned)
    assert!(
        app.world().get_entity(bolt_entity).is_ok(),
        "baseline bolt entity should still be alive after respawn"
    );
}

#[test]
fn bolt_lost_sends_correct_entities_when_multiple_bolts_lost_in_same_frame() {
    use crate::shared::test_utils::TestAppBuilder;

    let mut app = TestAppBuilder::new()
        .with_playfield()
        .with_resource::<crate::shared::GameRng>()
        .with_message::<BoltLost>()
        .with_system(FixedUpdate, (bolt_lost, capture_bolt_lost.after(bolt_lost)))
        .build();

    app.init_resource::<CapturedBoltLost>();

    let playfield = PlayfieldConfig::default();
    let breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -250.0)),
            rantzsoft_spatial2d::components::Spatial2D,
            GameDrawLayer::Breaker,
        ))
        .id();

    let bolt_a = spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    let bolt_b = spawn_bolt(
        &mut app,
        Vec2::new(50.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let captured = app.world().resource::<CapturedBoltLost>();
    assert_eq!(
        captured.0.len(),
        2,
        "exactly two BoltLost messages should be captured"
    );

    // Both messages should have breaker == breaker_entity
    for msg in &captured.0 {
        assert_eq!(
            msg.breaker, breaker_entity,
            "each BoltLost.breaker should equal the breaker entity"
        );
    }

    // The bolt values should be distinct and match bolt_a and bolt_b (order may vary)
    let bolt_entities: Vec<Entity> = captured.0.iter().map(|m| m.bolt).collect();
    assert!(
        bolt_entities.contains(&bolt_a),
        "BoltLost bolt entities should contain bolt_a"
    );
    assert!(
        bolt_entities.contains(&bolt_b),
        "BoltLost bolt entities should contain bolt_b"
    );
    assert_ne!(
        bolt_entities[0], bolt_entities[1],
        "BoltLost bolt entities should be distinct"
    );
}
