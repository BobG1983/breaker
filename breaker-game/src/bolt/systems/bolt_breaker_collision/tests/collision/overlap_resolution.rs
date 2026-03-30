//! Overlap resolution and side-hit tests for bolt-breaker collision.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::super::helpers::*;

#[test]
fn overlap_resolved_writes_position2d_y() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    app.insert_resource(HitBreakers::default()).add_systems(
        FixedUpdate,
        collect_breaker_hits.after(super::super::super::system::bolt_breaker_collision),
    );

    let animated_y = y_pos + 10.0;
    spawn_breaker_at(&mut app, 0.0, animated_y);

    let bolt_entity = spawn_bolt(&mut app, 0.0, y_pos, 0.0, -400.0);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "overlap should reflect bolt upward, got vy={:.1}",
        vel.0.y
    );

    let pos = app.world().get::<Position2D>(bolt_entity).unwrap();
    let expected_y = animated_y + hh.half_height() + default_bolt_radius().0;
    assert!(
        (pos.0.y - expected_y).abs() < 1.0,
        "bolt Position2D.0.y should be pushed above breaker, y={:.1} expected={expected_y:.1}",
        pos.0.y
    );

    let hits = app.world().resource::<HitBreakers>();
    assert_eq!(
        hits.0, 1,
        "overlap with downward bolt should send BoltImpactBreaker"
    );
}

#[test]
fn upward_bolt_inside_breaker_pushed_out_no_message() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    app.insert_resource(HitBreakers::default()).add_systems(
        FixedUpdate,
        collect_breaker_hits.after(super::super::super::system::bolt_breaker_collision),
    );

    let animated_y = y_pos + 10.0;
    spawn_breaker_at(&mut app, 0.0, animated_y);

    let bolt_entity = spawn_bolt(&mut app, 0.0, animated_y, 50.0, 400.0);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "upward bolt should keep moving up, got vy={:.1}",
        vel.0.y
    );
    assert!(
        (vel.0.x - 50.0).abs() < f32::EPSILON,
        "velocity should be unchanged, got vx={:.1}",
        vel.0.x
    );

    let pos = app.world().get::<Position2D>(bolt_entity).unwrap();
    let min_y = animated_y + hh.half_height() + default_bolt_radius().0;
    assert!(
        pos.0.y >= min_y - 0.01,
        "bolt Position2D.0.y should be pushed above breaker, y={:.3} min={min_y:.3}",
        pos.0.y
    );

    let hits = app.world().resource::<HitBreakers>();
    assert!(
        hits.0 == 0,
        "upward bolt overlap should NOT send BoltImpactBreaker"
    );
}

#[test]
fn upward_bolt_side_hit_is_not_reflected() {
    let mut app = test_app();
    let breaker_y = -250.0;
    app.insert_resource(HitBreakers::default()).add_systems(
        FixedUpdate,
        collect_breaker_hits.after(super::super::super::system::bolt_breaker_collision),
    );
    spawn_breaker_at(&mut app, 0.0, breaker_y);

    let bolt_entity = spawn_bolt(&mut app, -70.0, breaker_y, 200.0, 300.0);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.x > 0.0,
        "upward side hit should NOT flip X velocity (guard should skip), got vx={:.1}",
        vel.0.x
    );
    assert!(
        vel.0.y > 0.0,
        "upward side hit should NOT flip Y velocity, got vy={:.1}",
        vel.0.y
    );

    let hits = app.world().resource::<HitBreakers>();
    assert!(
        hits.0 == 0,
        "upward side hit should NOT send BoltImpactBreaker, got {} messages",
        hits.0
    );
}

#[test]
fn downward_bolt_side_hit_is_reflected() {
    let mut app = test_app();
    let breaker_y = -250.0;
    app.insert_resource(HitBreakers::default()).add_systems(
        FixedUpdate,
        collect_breaker_hits.after(super::super::super::system::bolt_breaker_collision),
    );
    spawn_breaker_at(&mut app, 0.0, breaker_y);

    let bolt_entity = spawn_bolt(&mut app, -70.0, breaker_y, 200.0, -300.0);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.x < 0.0,
        "downward side hit SHOULD flip X velocity, got vx={:.1}",
        vel.0.x
    );
}
