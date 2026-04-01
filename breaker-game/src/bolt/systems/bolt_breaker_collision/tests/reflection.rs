use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::helpers::*;
use crate::{
    bolt::components::Bolt,
    breaker::components::{Breaker, BreakerTilt},
    effect::effects::speed_boost::ActiveSpeedBoosts,
    shared::GameDrawLayer,
};

#[test]
fn left_hit_reflects_leftward() {
    let mut app = test_app();
    let hw = default_breaker_width();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let hit_x = -hw.half_width() + 5.0;
    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    spawn_bolt(&mut app, hit_x, start_y, 0.0, -400.0);
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.x < 0.0, "left hit should angle bolt leftward");
    assert!(vel.0.y > 0.0, "bolt should still go upward");
}

#[test]
fn right_hit_reflects_rightward() {
    let mut app = test_app();
    let hw = default_breaker_width();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let hit_x = hw.half_width() - 5.0;
    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    spawn_bolt(&mut app, hit_x, start_y, 0.0, -400.0);
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.x > 0.0, "right hit should angle bolt rightward");
}

#[test]
fn tilt_affects_reflection() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;

    app.world_mut().spawn((
        Breaker,
        BreakerTilt {
            angle: 0.3,
            ease_start: 0.0,
            ease_target: 0.0,
        },
        default_breaker_width(),
        default_breaker_height(),
        default_reflection_spread(),
        Position2D(Vec2::new(0.0, y_pos)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.x > 0.0,
        "right tilt should push bolt rightward even on center hit"
    );
}

/// Behavior 7: `reflect_top_hit` uses `base_speed * ActiveSpeedBoosts.multiplier()` as speed floor.
///
/// Given: Bolt with `BaseSpeed(400.0)`, `ActiveSpeedBoosts(vec![2.0])`,
///        velocity (0.0, 300.0) (speed=300, below boosted base of 800), hitting breaker center.
/// When: bolt hits breaker top surface.
/// Then: post-reflection speed >= 800.0 (400.0 * 2.0).
#[test]
fn reflect_top_hit_uses_active_speed_boosts_as_speed_floor() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = Bolt::builder()
        .at_position(Vec2::new(0.0, start_y))
        .with_speed(400.0, 0.0, f32::MAX)
        .with_angle(0.0, 0.0)
        .with_velocity(Velocity2D(Vec2::new(0.0, -300.0)))
        .primary()
        .spawn(app.world_mut());
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(ActiveSpeedBoosts(vec![2.0]));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt should have reflected upward off breaker, got vy={}",
        vel.0.y
    );
    let speed = vel.speed();
    assert!(
        speed >= 800.0 - 1.0,
        "post-reflection speed should be >= 800.0 (base 400 * boost 2.0), got {speed}",
    );
}

/// Behavior 7 edge case: No `ActiveSpeedBoosts` -> speed floor = `base_speed` * 1.0.
#[test]
fn reflect_top_hit_without_speed_boosts_uses_raw_base_speed() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = Bolt::builder()
        .at_position(Vec2::new(0.0, start_y))
        .with_speed(400.0, 0.0, f32::MAX)
        .with_angle(0.0, 0.0)
        .with_velocity(Velocity2D(Vec2::new(0.0, -300.0)))
        .primary()
        .spawn(app.world_mut());
    // No ActiveSpeedBoosts

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt should have reflected upward off breaker, got vy={}",
        vel.0.y
    );
    let speed = vel.speed();
    assert!(
        speed >= 400.0 - 1.0,
        "post-reflection speed should be >= 400.0 (base_speed * 1.0 default), got {speed}",
    );
}

/// Behavior 8: `reflect_top_hit` ignores `BaseSpeed` alone when boost active.
///
/// Given: Bolt with `BaseSpeed(400.0)`, `ActiveSpeedBoosts(vec![2.0])`,
///        velocity (0.0, 500.0) (speed=500, above raw base but below boosted base of 800).
/// When: bolt hits breaker top surface.
/// Then: post-reflection speed >= 800.0 (not just 400.0).
#[test]
fn reflect_top_hit_ignores_base_speed_alone_when_boost_active() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = Bolt::builder()
        .at_position(Vec2::new(0.0, start_y))
        .with_speed(400.0, 0.0, f32::MAX)
        .with_angle(0.0, 0.0)
        .with_velocity(Velocity2D(Vec2::new(0.0, -500.0)))
        .primary()
        .spawn(app.world_mut());
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(ActiveSpeedBoosts(vec![2.0]));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt should have reflected upward off breaker, got vy={}",
        vel.0.y
    );
    let speed = vel.speed();
    assert!(
        speed >= 800.0 - 1.0,
        "post-reflection speed should be >= 800.0 (base 400 * boost 2.0), not just 500 from base_speed comparison, got {speed}",
    );
}
