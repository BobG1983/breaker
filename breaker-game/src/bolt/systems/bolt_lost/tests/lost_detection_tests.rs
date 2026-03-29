use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, Spatial2D, Velocity2D};

use super::helpers::*;
use crate::{
    bolt::{
        components::{Bolt, BoltBaseSpeed, BoltRadius, BoltRespawnAngleSpread, BoltRespawnOffsetY},
        resources::BoltConfig,
    },
    breaker::components::Breaker,
    shared::{EntityScale, GameDrawLayer, PlayfieldConfig},
};

#[test]
fn bolt_below_floor_detected_via_position2d() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));
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
    let bolt_config = BoltConfig::default();
    let playfield = PlayfieldConfig::default();
    let breaker_x = 42.0;
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(breaker_x, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(100.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(200.0, playfield.bottom() - 100.0)),
    ));
    tick(&mut app);

    let (vel, pos) = app
        .world_mut()
        .query::<(&Velocity2D, &Position2D)>()
        .iter(app.world())
        .next()
        .unwrap();

    let speed = vel.0.length();
    assert!(
        (speed - bolt_config.base_speed).abs() < 1.0,
        "respawn speed should equal base_speed {:.0}, got {:.1}",
        bolt_config.base_speed,
        speed,
    );

    let angle = vel.0.x.atan2(vel.0.y).abs();
    assert!(
        angle <= bolt_config.respawn_angle_spread + f32::EPSILON,
        "respawn angle {angle:.3} rad should be within spread {:.3} rad",
        bolt_config.respawn_angle_spread,
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
    let bolt_config = BoltConfig::default();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(100.0, -400.0)),
        (
            BoltBaseSpeed(bolt_config.base_speed),
            BoltRadius(bolt_config.radius),
            BoltRespawnOffsetY(bolt_config.respawn_offset_y),
            BoltRespawnAngleSpread(0.0),
        ),
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();

    assert!(
        vel.0.x.abs() < f32::EPSILON,
        "zero spread should launch straight up, got vx={:.3}",
        vel.0.x,
    );
}

#[test]
fn respawn_position2d_y_uses_respawn_offset() {
    let mut app = test_app();
    let bolt_config = BoltConfig::default();
    let playfield = PlayfieldConfig::default();
    let breaker_y = -250.0;
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, breaker_y)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));
    tick(&mut app);

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();

    let expected_y = breaker_y + bolt_config.respawn_offset_y;
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "respawn Position2D.0.y should be breaker_y + respawn_offset_y ({expected_y}), got {}",
        pos.0.y,
    );
}

#[test]
fn respawn_inserts_previous_position_matching_position2d() {
    let mut app = test_app();
    let bolt_config = BoltConfig::default();
    let playfield = PlayfieldConfig::default();
    let breaker_x = 42.0;
    let breaker_y = -250.0;
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(breaker_x, breaker_y)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));
    tick(&mut app);

    let (pos, prev_pos) = app
        .world_mut()
        .query_filtered::<(&Position2D, &PreviousPosition), With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();

    let expected = Vec2::new(breaker_x, breaker_y + bolt_config.respawn_offset_y);
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
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(100.0, -200.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, 100.0)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.y < 0.0, "bolt above floor should keep going down");
}

// --- EntityScale lost detection tests ---

#[test]
fn scaled_bolt_uses_effective_radius_for_lost_detection() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let bolt_y = playfield.bottom() - 4.0 - 1.0; // -305.0
    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        EntityScale(0.5),
        Position2D(Vec2::new(0.0, bolt_y)),
    ));
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
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        // No EntityScale
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt without EntityScale should be respawned normally, got vy={:.1}",
        vel.0.y
    );
}
