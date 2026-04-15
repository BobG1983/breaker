//! Basic bolt-breaker reflection tests — center hit, far above, upward skip,
//! and `Position2D`-based breaker position.

use crate::{bolt::systems::bolt_breaker_collision::tests::helpers::*, prelude::*};

#[test]
fn bolt_reflects_upward_on_center_hit() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.y > 0.0, "bolt should reflect upward");
}

#[test]
fn no_collision_when_bolt_above() {
    let mut app = test_app();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    spawn_bolt(&mut app, 0.0, 200.0, 0.0, -400.0);
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.y < 0.0, "bolt should not be reflected when far above");
}

#[test]
fn upward_bolt_not_reflected() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.y > 0.0, "upward-moving bolt should not be reflected");
}

/// Breaker position is read from `Position2D` (not `Transform`).
/// This test verifies that the system reads breaker from `Position2D.0`.
#[test]
fn breaker_position_read_from_position2d() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let breaker_x = 50.0;
    let breaker_y = -250.0;
    // Breaker at non-zero X via Position2D
    spawn_breaker_at(&mut app, breaker_x, breaker_y);

    let start_y = breaker_y + hh.half_height() + default_bolt_radius().0 + 3.0;
    // Bolt at breaker_x so it hits the center
    spawn_bolt(&mut app, breaker_x, start_y, 0.0, -400.0);
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt should reflect off breaker at Position2D (50, -250), got vy={:.1}",
        vel.0.y
    );
}
