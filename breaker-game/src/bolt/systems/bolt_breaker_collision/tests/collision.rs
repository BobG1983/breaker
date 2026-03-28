use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Spatial2D, Velocity2D};

use super::helpers::*;
use crate::{
    bolt::components::Bolt,
    breaker::components::{Breaker, BreakerTilt},
    chips::components::{Piercing, PiercingRemaining, WidthBoost},
    shared::{EntityScale, GameDrawLayer},
};

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

#[test]
fn overlap_resolved_writes_position2d_y() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    app.insert_resource(HitBreakers::default()).add_systems(
        FixedUpdate,
        collect_breaker_hits.after(super::super::system::bolt_breaker_collision),
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
        collect_breaker_hits.after(super::super::system::bolt_breaker_collision),
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
        collect_breaker_hits.after(super::super::system::bolt_breaker_collision),
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
        collect_breaker_hits.after(super::super::system::bolt_breaker_collision),
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

#[test]
fn multiple_bolts_each_reflect_off_breaker() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    app.insert_resource(HitBreakers::default()).add_systems(
        FixedUpdate,
        collect_breaker_hits.after(super::super::system::bolt_breaker_collision),
    );
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;

    let left_bolt = spawn_bolt(&mut app, -30.0, start_y, 0.0, -400.0);
    let right_bolt = spawn_bolt(&mut app, 30.0, start_y, 0.0, -400.0);

    tick(&mut app);

    let velocities: Vec<(Entity, Vec2)> = app
        .world_mut()
        .query::<(Entity, &Velocity2D)>()
        .iter(app.world())
        .map(|(e, v)| (e, v.0))
        .collect();

    for (entity, vel) in &velocities {
        assert!(
            vel.y > 0.0,
            "bolt {entity:?} should reflect upward, got vy={:.1}",
            vel.y
        );
    }

    let hits = app.world().resource::<HitBreakers>();
    assert_eq!(hits.0, 2, "both bolts should trigger hit messages");

    let left_vel = velocities.iter().find(|(e, _)| *e == left_bolt).unwrap().1;
    let right_vel = velocities.iter().find(|(e, _)| *e == right_bolt).unwrap().1;
    assert!(
        left_vel.x < 0.0,
        "left bolt should angle leftward, got vx={:.1}",
        left_vel.x
    );
    assert!(
        right_vel.x > 0.0,
        "right bolt should angle rightward, got vx={:.1}",
        right_vel.x
    );
}

// --- Chip effect reset tests ---

#[test]
fn breaker_hit_resets_piercing_remaining() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_param_bundle(),
            Piercing(3),
            PiercingRemaining(0),
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt should have reflected off breaker, got vy={}",
        vel.0.y
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 3,
        "breaker hit should reset PiercingRemaining to Piercing.0 (3), got {}",
        pr.0
    );
}

#[test]
fn piercing_remaining_without_piercing_does_not_reset_on_breaker_hit() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_param_bundle(),
            PiercingRemaining(5),
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt should have reflected off breaker, got vy={}",
        vel.0.y
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 5,
        "PiercingRemaining without Piercing should not be reset on breaker hit, got {}",
        pr.0
    );
}

// --- WidthBoost tests ---

#[test]
fn width_boost_widens_effective_breaker_collision_width() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;

    app.world_mut().spawn((
        Breaker,
        BreakerTilt::default(),
        default_breaker_width(),
        default_breaker_height(),
        default_max_reflection_angle(),
        default_min_angle(),
        WidthBoost(40.0),
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
        "bolt at x=75.0 (inside boosted width) should reflect upward, got vy={}",
        vel.0.y
    );
}

// --- EntityScale collision tests ---

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
fn width_boost_stacks_with_entity_scale() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;

    app.world_mut().spawn((
        Breaker,
        BreakerTilt::default(),
        default_breaker_width(),
        default_breaker_height(),
        default_max_reflection_angle(),
        default_min_angle(),
        WidthBoost(40.0),
        EntityScale(0.7),
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
        "bolt at x=70 should miss scaled breaker (expanded half_w=64), got vy={:.1}",
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
        "EntityScale(1.0) should produce identical behavior to no scale, got vy={:.1}",
        vel.0.y
    );
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
