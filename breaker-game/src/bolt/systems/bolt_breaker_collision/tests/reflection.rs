use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Spatial2D, Velocity2D};

use super::helpers::*;
use crate::{
    breaker::components::{Breaker, BreakerTilt},
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
        default_max_reflection_angle(),
        default_min_angle(),
        Position2D(Vec2::new(0.0, y_pos)),
        Spatial2D,
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
