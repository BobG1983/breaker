//! Phantom-breaker collision gating — regression guard for single real breaker.
//!
//! Group H: Single real breaker baseline tests pass today and act as the
//! "do not break the real breaker" guard.

use bevy::prelude::*;

use super::helpers::{breaker_y, start_y_above};
use crate::{
    bolt::{
        components::{ImpactSide, LastImpact, PiercingRemaining},
        systems::bolt_breaker_collision::tests::helpers::*,
        test_utils::piercing_stack,
    },
    breaker::components::BreakerTilt,
    prelude::*,
};

// ── Group H: Regression guard — single real breaker baseline ─────────────────
//
// This test PASSES today. It is the "do not break the real breaker" guard.
// If any of the above gating work accidentally breaks single-real-breaker
// behavior, this test will catch it.

/// Behavior #10 (regression guard): single real breaker (no phantom) still
/// applies all effects — reflects upward, stamps `LastImpact::Top`, resets
/// `PiercingRemaining`, emits `BoltImpactBreaker`.
#[test]
fn single_real_breaker_baseline_is_unchanged() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();

    app.insert_resource(CapturedHitPairs::default())
        .add_systems(
            FixedUpdate,
            collect_breaker_hit_pairs.after(
                crate::bolt::systems::bolt_breaker_collision::system::bolt_breaker_collision,
            ),
        );

    let real_entity = spawn_breaker_at(&mut app, 0.0, by);

    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((piercing_stack(&[3]), PiercingRemaining(0)));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "single real breaker: bolt should reflect upward (vy={:.1})",
        vel.0.y
    );

    let li = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("single real breaker: LastImpact should be stamped");
    assert_eq!(
        li.side,
        ImpactSide::Top,
        "single real breaker: ImpactSide::Top expected (got {:?})",
        li.side
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 3,
        "single real breaker: PiercingRemaining should be reset to 3 (got {})",
        pr.0
    );

    let captured = app.world().resource::<CapturedHitPairs>();
    assert_eq!(
        captured.0.len(),
        1,
        "single real breaker: exactly one BoltImpactBreaker message (got {})",
        captured.0.len()
    );
    assert_eq!(
        captured.0[0].1, real_entity,
        "BoltImpactBreaker.breaker should be real_entity"
    );
}

/// Behavior #10 edge case: real breaker with tilt still steers when no phantom present.
#[test]
fn single_real_breaker_with_tilt_still_steers() {
    let mut app = test_app();
    let by = breaker_y();

    let real_entity = spawn_breaker_at(&mut app, 0.0, by);
    app.world_mut().entity_mut(real_entity).insert(BreakerTilt {
        angle:       0.3,
        ease_start:  0.0,
        ease_target: 0.0,
    });

    let start_y = start_y_above(by);
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.x > 0.0,
        "single real breaker tilt should steer bolt rightward (vel.x={:.4})",
        vel.0.x
    );
}
