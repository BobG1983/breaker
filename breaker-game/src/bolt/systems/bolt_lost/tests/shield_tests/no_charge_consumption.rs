//! Behaviors 13-15: Bolt above floor, boundary threshold, barely-below-floor.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::{
    super::{super::system::bolt_lost, helpers::*},
    helpers::spawn_shielded_breaker,
};
use crate::{bolt::components::Bolt, effect::effects::shield::ShieldActive};

// ── Behavior 13: Bolt above floor does not consume shield charges ──

#[test]
fn bolt_above_floor_does_not_consume_charges() {
    // Given: Breaker with ShieldActive { charges: 3 }. Bolt at (0.0, 100.0) above floor.
    // When: bolt_lost runs
    // Then: Bolt velocity unchanged. Bolt position unchanged. charges remain 3.
    let mut app = test_app();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 3);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(100.0, -200.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, 100.0)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (vel.0.x - 100.0).abs() < f32::EPSILON,
        "bolt above floor should keep vx=100.0, got {:.1}",
        vel.0.x
    );
    assert!(
        (vel.0.y - (-200.0)).abs() < f32::EPSILON,
        "bolt above floor should keep vy=-200.0, got {:.1}",
        vel.0.y
    );

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.y - 100.0).abs() < f32::EPSILON,
        "bolt above floor should keep y=100.0, got {:.1}",
        pos.0.y
    );

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(count.0, 0, "bolt above floor should not send BoltLost");

    let shield = app.world().get::<ShieldActive>(breaker).unwrap();
    assert_eq!(
        shield.charges, 3,
        "charges should remain 3 when no bolt is lost, got {}",
        shield.charges
    );
}

// ── Behavior 14: Bolt at exactly bottom()-radius is NOT lost (boundary) ──

#[test]
fn bolt_at_exactly_threshold_is_not_lost() {
    // Given: Bolt at (0.0, -308.0) which is exactly bottom() - radius = -300.0 - 8.0 = -308.0.
    //        Condition is strict `<`, so -308.0 is NOT below threshold.
    // Then: Bolt NOT considered lost, velocity unchanged, charges remain 3.
    let mut app = test_app();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 3);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, -308.0)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (vel.0.y - (-400.0)).abs() < f32::EPSILON,
        "bolt at exact threshold should keep vy=-400.0, got {:.1}",
        vel.0.y
    );

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(count.0, 0, "bolt at exact threshold should NOT be lost");

    let shield = app.world().get::<ShieldActive>(breaker).unwrap();
    assert_eq!(
        shield.charges, 3,
        "charges should remain 3 (bolt was not lost), got {}",
        shield.charges
    );
}

#[test]
fn bolt_barely_below_threshold_is_absorbed_by_shield() {
    // Edge case: Bolt at (0.0, -308.001) — IS below threshold, shield absorbs, charges 3→2.
    let mut app = test_app();
    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 3);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, -308.001)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt barely below threshold should be reflected, got vy={:.1}",
        vel.0.y
    );

    let shield = app.world().get::<ShieldActive>(breaker).unwrap();
    assert_eq!(
        shield.charges, 2,
        "charges should decrement from 3 to 2, got {}",
        shield.charges
    );
}

// ── Behavior 15: Shield with barely-below-floor bolt still absorbs and decrements ──

#[test]
fn shield_absorbs_barely_below_floor_bolt_and_removes_on_last_charge() {
    // Given: Breaker with ShieldActive { charges: 1 }. Bolt at (0.0, -308.5).
    //        Floor threshold = -308.0. Bolt Y below threshold.
    // Then: Bolt reflected. ShieldActive removed (charges was 1, now 0).
    let mut app = test_app();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 1);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, -308.5)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "shield should reflect bolt barely below threshold, got vy={:.1}",
        vel.0.y
    );

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(count.0, 0, "shield should prevent BoltLost");

    assert!(
        app.world().get::<ShieldActive>(breaker).is_none(),
        "ShieldActive should be removed when charges reach 0"
    );
}
