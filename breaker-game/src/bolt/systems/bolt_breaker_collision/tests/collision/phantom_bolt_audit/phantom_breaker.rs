use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::{
        components::{ImpactSide, LastImpact, PiercingRemaining},
        test_utils::piercing_stack,
    },
    breaker::components::{BreakerReflectionSpread, BreakerTilt},
    prelude::*,
};

// ── T10 — Phantom bolt bounces off phantom breaker; phantom-breaker gates apply ─
//
// The phantom-breaker gates are on With<PhantomBreaker> (breaker side).
// A phantom bolt (With<PhantomBolt>) hits a phantom breaker the SAME WAY a
// normal bolt does — tilt-skip, spread-skip, LastImpact-skip, and
// piercing-reset-skip are all per-BREAKER, not per-BOLT.

fn breaker_y() -> f32 {
    -250.0
}

fn start_y_above_phantom_breaker() -> f32 {
    let hh = default_breaker_height();
    breaker_y() + hh.half_height() + default_bolt_radius().0 + 3.0
}

fn assert_phantom_breaker_outcomes(app_a: &App, app_b: &App, bolt_a: Entity, bolt_b: Entity) {
    // Both emit exactly one `BoltImpactBreaker`
    let pairs_a = app_a.world().resource::<CapturedHitPairs>();
    assert_eq!(
        pairs_a.0.len(),
        1,
        "app_a: expected 1 BoltImpactBreaker on phantom-breaker hit, got {}",
        pairs_a.0.len()
    );
    assert_eq!(pairs_a.0[0].0, bolt_a, "app_a: msg.bolt should be bolt_a");

    let pairs_b = app_b.world().resource::<CapturedHitPairs>();
    assert_eq!(
        pairs_b.0.len(),
        1,
        "app_b: expected 1 BoltImpactBreaker on phantom-breaker hit, got {}",
        pairs_b.0.len()
    );
    assert_eq!(pairs_b.0[0].0, bolt_b, "app_b: msg.bolt should be bolt_b");

    let vel_a = app_a.world().get::<Velocity2D>(bolt_a).unwrap().0;
    let vel_b = app_b.world().get::<Velocity2D>(bolt_b).unwrap().0;

    // Tilt-skip gate is on With<PhantomBreaker>, NOT With<PhantomBolt> —
    // both bolts should end with vel.x ≈ 0.0 (tilt not applied)
    assert!(
        vel_a.x.abs() < 1e-3,
        "normal bolt on phantom breaker: tilt-skip should suppress vel.x, got {}",
        vel_a.x
    );
    assert!(
        vel_b.x.abs() < 1e-3,
        "phantom bolt on phantom breaker: tilt-skip should suppress vel.x, got {}",
        vel_b.x
    );
    assert!(
        vel_a.y > 0.0,
        "normal bolt vy should be positive after phantom-breaker reflect, got {}",
        vel_a.y
    );
    assert!(
        vel_b.y > 0.0,
        "phantom bolt vy should be positive after phantom-breaker reflect, got {}",
        vel_b.y
    );
    assert!(
        (vel_a.y - vel_b.y).abs() < 1e-3,
        "vy must match: normal={}, phantom={}",
        vel_a.y,
        vel_b.y
    );

    // `LastImpact` absent on both (phantom-breaker path skips `stamp_last_impact`)
    assert!(
        app_a.world().get::<LastImpact>(bolt_a).is_none(),
        "normal bolt on phantom breaker should NOT have LastImpact"
    );
    assert!(
        app_b.world().get::<LastImpact>(bolt_b).is_none(),
        "phantom bolt on phantom breaker should NOT have LastImpact"
    );

    // `PiercingRemaining`(0) unchanged — phantom-breaker emit_bump returns early
    let pr_a = app_a
        .world()
        .get::<PiercingRemaining>(bolt_a)
        .expect("normal bolt should still have PiercingRemaining");
    let pr_b = app_b
        .world()
        .get::<PiercingRemaining>(bolt_b)
        .expect("phantom bolt should still have PiercingRemaining");
    assert_eq!(
        pr_a.0, 0,
        "normal bolt PiercingRemaining should remain 0 on phantom-breaker hit"
    );
    assert_eq!(
        pr_b.0, 0,
        "phantom bolt PiercingRemaining should remain 0 on phantom-breaker hit"
    );
}

fn make_app_with_collector() -> App {
    let mut app = test_app();
    app.insert_resource(CapturedHitPairs::default());
    app.add_systems(
        FixedUpdate,
        collect_breaker_hit_pairs
            .after(crate::bolt::systems::bolt_breaker_collision::system::bolt_breaker_collision),
    );
    app
}

/// T10 main case: both a normal bolt and a phantom bolt hitting the same phantom
/// breaker produce identical outcomes — `BoltImpactBreaker` emitted, tilt-skip
/// applied, `LastImpact` absent, `PiercingRemaining` unchanged.
#[test]
fn phantom_bolt_on_phantom_breaker_emits_and_matches_normal_bolt() {
    let start_y = start_y_above_phantom_breaker();

    let mut app_a = make_app_with_collector();
    let breaker_a = spawn_phantom_breaker_at(&mut app_a, 0.0, breaker_y());
    app_a.world_mut().entity_mut(breaker_a).insert(BreakerTilt {
        angle:       0.3,
        ease_start:  0.0,
        ease_target: 0.0,
    });
    let bolt_a = spawn_bolt(&mut app_a, 0.0, start_y, 0.0, -400.0);
    app_a
        .world_mut()
        .entity_mut(bolt_a)
        .insert((piercing_stack(&[3]), PiercingRemaining(0)));

    let mut app_b = make_app_with_collector();
    let breaker_b = spawn_phantom_breaker_at(&mut app_b, 0.0, breaker_y());
    app_b.world_mut().entity_mut(breaker_b).insert(BreakerTilt {
        angle:       0.3,
        ease_start:  0.0,
        ease_target: 0.0,
    });
    let bolt_b = spawn_phantom_bolt(&mut app_b, 0.0, start_y, 0.0, -400.0);
    app_b
        .world_mut()
        .entity_mut(bolt_b)
        .insert((piercing_stack(&[3]), PiercingRemaining(0)));

    tick(&mut app_a);
    tick(&mut app_b);

    assert_phantom_breaker_outcomes(&app_a, &app_b, bolt_a, bolt_b);
}

/// T10 edge case: spread-skip on phantom breaker applies to BOTH bolt types.
/// `BreakerReflectionSpread(2.0)` + off-center bolt → `vel.x.abs() < 1e-3` for both.
#[test]
fn phantom_bolt_spread_skip_on_phantom_breaker_matches_normal_bolt() {
    let start_y = start_y_above_phantom_breaker();
    // Off-center at x=-50; spread would normally produce vel.x != 0 on a real
    // breaker, but phantom-breaker skips spread for both bolt types.

    let mut app_a = make_app_with_collector();
    let breaker_a = spawn_phantom_breaker_at(&mut app_a, 0.0, breaker_y());
    app_a
        .world_mut()
        .entity_mut(breaker_a)
        .insert(BreakerReflectionSpread(2.0));
    let bolt_a = spawn_bolt(&mut app_a, -50.0, start_y, 0.0, -400.0);

    let mut app_b = make_app_with_collector();
    let breaker_b = spawn_phantom_breaker_at(&mut app_b, 0.0, breaker_y());
    app_b
        .world_mut()
        .entity_mut(breaker_b)
        .insert(BreakerReflectionSpread(2.0));
    let bolt_b = spawn_phantom_bolt(&mut app_b, -50.0, start_y, 0.0, -400.0);

    tick(&mut app_a);
    tick(&mut app_b);

    let vel_a = app_a.world().get::<Velocity2D>(bolt_a).unwrap().0;
    let vel_b = app_b.world().get::<Velocity2D>(bolt_b).unwrap().0;

    assert!(
        vel_a.x.abs() < 1e-3,
        "normal bolt: spread-skip on phantom-breaker should suppress vel.x, got {}",
        vel_a.x
    );
    assert!(
        vel_b.x.abs() < 1e-3,
        "phantom bolt: spread-skip on phantom-breaker should suppress vel.x, got {}",
        vel_b.x
    );
    assert!(
        (vel_a.y - vel_b.y).abs() < 1e-3,
        "vy must match: normal={}, phantom={}",
        vel_a.y,
        vel_b.y
    );
}

/// T10 edge case: pre-existing `LastImpact` on bolt is preserved unchanged after
/// hitting a phantom breaker (no `stamp_last_impact` call on phantom-breaker path).
#[test]
fn phantom_bolt_pre_existing_last_impact_preserved_on_phantom_breaker_hit() {
    let start_y = start_y_above_phantom_breaker();
    let sentinel = LastImpact {
        position: Vec2::new(999.0, 999.0),
        side:     ImpactSide::Left,
    };

    let mut app_a = make_app_with_collector();
    spawn_phantom_breaker_at(&mut app_a, 0.0, breaker_y());
    let bolt_a = spawn_bolt(&mut app_a, 0.0, start_y, 0.0, -400.0);
    app_a
        .world_mut()
        .entity_mut(bolt_a)
        .insert(sentinel.clone());

    let mut app_b = make_app_with_collector();
    spawn_phantom_breaker_at(&mut app_b, 0.0, breaker_y());
    let bolt_b = spawn_phantom_bolt(&mut app_b, 0.0, start_y, 0.0, -400.0);
    app_b.world_mut().entity_mut(bolt_b).insert(sentinel);

    tick(&mut app_a);
    tick(&mut app_b);

    // Both should retain the sentinel LastImpact unchanged
    let li_a = app_a
        .world()
        .get::<LastImpact>(bolt_a)
        .expect("normal bolt: LastImpact should still be present after phantom-breaker hit");
    let li_b = app_b
        .world()
        .get::<LastImpact>(bolt_b)
        .expect("phantom bolt: LastImpact should still be present after phantom-breaker hit");

    assert!(
        (li_a.position.x - 999.0).abs() < 1e-3 && (li_a.position.y - 999.0).abs() < 1e-3,
        "normal bolt: LastImpact.position should remain sentinel (999,999), got {:?}",
        li_a.position
    );
    assert_eq!(
        li_a.side,
        ImpactSide::Left,
        "normal bolt: LastImpact.side should remain Left"
    );

    assert!(
        (li_b.position.x - 999.0).abs() < 1e-3 && (li_b.position.y - 999.0).abs() < 1e-3,
        "phantom bolt: LastImpact.position should remain sentinel (999,999), got {:?}",
        li_b.position
    );
    assert_eq!(
        li_b.side,
        ImpactSide::Left,
        "phantom bolt: LastImpact.side should remain Left"
    );
}

/// T10 edge case: pre-existing `PiercingRemaining(5)` (no active piercings) is
/// preserved on both bolt types after hitting a phantom breaker.
#[test]
fn phantom_bolt_pre_existing_piercing_remaining_preserved_on_phantom_breaker_hit() {
    let start_y = start_y_above_phantom_breaker();

    let mut app_a = make_app_with_collector();
    spawn_phantom_breaker_at(&mut app_a, 0.0, breaker_y());
    let bolt_a = spawn_bolt(&mut app_a, 0.0, start_y, 0.0, -400.0);
    app_a
        .world_mut()
        .entity_mut(bolt_a)
        .insert(PiercingRemaining(5));

    let mut app_b = make_app_with_collector();
    spawn_phantom_breaker_at(&mut app_b, 0.0, breaker_y());
    let bolt_b = spawn_phantom_bolt(&mut app_b, 0.0, start_y, 0.0, -400.0);
    app_b
        .world_mut()
        .entity_mut(bolt_b)
        .insert(PiercingRemaining(5));

    tick(&mut app_a);
    tick(&mut app_b);

    let pr_a = app_a
        .world()
        .get::<PiercingRemaining>(bolt_a)
        .expect("normal bolt: PiercingRemaining should still be present");
    let pr_b = app_b
        .world()
        .get::<PiercingRemaining>(bolt_b)
        .expect("phantom bolt: PiercingRemaining should still be present");

    assert_eq!(
        pr_a.0, 5,
        "normal bolt: PiercingRemaining should remain 5 on phantom-breaker hit, got {}",
        pr_a.0
    );
    assert_eq!(
        pr_b.0, 5,
        "phantom bolt: PiercingRemaining should remain 5 on phantom-breaker hit, got {}",
        pr_b.0
    );
}
