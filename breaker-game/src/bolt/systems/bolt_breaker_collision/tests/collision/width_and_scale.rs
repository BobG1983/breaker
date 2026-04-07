//! Tests for `ActiveSizeBoosts` (width boost) and `NodeScalingFactor` collision
//! behavior on the breaker.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Spatial2D, Velocity2D};

use crate::{
    bolt::systems::bolt_breaker_collision::tests::helpers::*,
    breaker::components::{Breaker, BreakerTilt},
    effect::effects::size_boost::ActiveSizeBoosts,
    shared::{GameDrawLayer, NodeScalingFactor},
};

// --- WidthBoost tests ---

#[test]
fn active_size_boosts_widens_breaker_collision_width() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;

    app.world_mut().spawn((
        Breaker,
        BreakerTilt::default(),
        default_breaker_width(),
        default_breaker_height(),
        default_reflection_spread(),
        ActiveSizeBoosts(vec![4.0_f32 / 3.0]),
        Position2D(Vec2::new(0.0, y_pos)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 75.0, start_y, 0.0, -400.0);

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt at x=75.0 (inside effective half_w=80 from 60*4/3) should reflect upward, got vy={}",
        vel.0.y
    );
}

// --- NodeScalingFactor collision tests ---

#[test]
fn scaled_breaker_has_smaller_collision_hitbox() {
    let mut app = test_app();
    let y_pos = -250.0;
    spawn_scaled_breaker_at(&mut app, 0.0, y_pos, 0.7);

    let bolt_entity = spawn_bolt(&mut app, 0.0, -234.0, 0.0, -1.0);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y < 0.0,
        "bolt at y=-234 should NOT be inside scaled breaker (scaled expanded top=-235), \
         got vy={:.1} (if positive, overlap resolution fired with unscaled dimensions)",
        vel.0.y
    );
}

#[test]
fn bolt_outside_scaled_breaker_width_misses() {
    let mut app = test_app();
    let y_pos = -250.0;
    spawn_scaled_breaker_at(&mut app, 0.0, y_pos, 0.7);

    let scaled_half_h = 10.0 * 0.7;
    let bolt_r = default_bolt_radius().0;
    let start_y = y_pos + scaled_half_h + bolt_r + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 55.0, start_y, 0.0, -400.0);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y < 0.0,
        "bolt at x=55 should miss scaled breaker (expanded half_w=50), got vy={:.1}",
        vel.0.y
    );
}

#[test]
fn active_size_boosts_stacks_with_entity_scale_in_collision() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;

    app.world_mut().spawn((
        Breaker,
        BreakerTilt::default(),
        default_breaker_width(),
        default_breaker_height(),
        default_reflection_spread(),
        ActiveSizeBoosts(vec![4.0_f32 / 3.0]),
        NodeScalingFactor(0.7),
        Position2D(Vec2::new(0.0, y_pos)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 70.0, start_y, 0.0, -400.0);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y < 0.0,
        "bolt at x=70 should miss scaled breaker (effective half_w = 60 * 4/3 * 0.7 = 56), got vy={:.1}",
        vel.0.y
    );
}

#[test]
fn entity_scale_1_0_is_backward_compatible_with_breaker_collision() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_scaled_breaker_at(&mut app, 0.0, y_pos, 1.0);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    spawn_scaled_bolt(&mut app, 0.0, start_y, 0.0, -400.0, 1.0);
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "NodeScalingFactor(1.0) should produce identical behavior to no scale, got vy={:.1}",
        vel.0.y
    );
}
