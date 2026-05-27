use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::{
        components::{ImpactSide, LastImpact, PiercingRemaining},
        test_utils::piercing_stack,
    },
    breaker::components::BreakerTilt,
    prelude::*,
};

// ── T9 — Phantom bolt bounces off real breaker identically to normal bolt ────

// start_y: breaker center y + half_h + bolt_radius + 3 overlap margin
fn breaker_y() -> f32 {
    -250.0
}

fn start_y_above_real_breaker() -> f32 {
    let hh = default_breaker_height();
    breaker_y() + hh.half_height() + default_bolt_radius().0 + 3.0
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

fn assert_real_breaker_center_hit(app_a: &App, app_b: &App, bolt_a: Entity, bolt_b: Entity) {
    let pairs_a = app_a.world().resource::<CapturedHitPairs>();
    assert_eq!(
        pairs_a.0.len(),
        1,
        "app_a: expected 1 BoltImpactBreaker, got {}",
        pairs_a.0.len()
    );
    assert_eq!(pairs_a.0[0].0, bolt_a, "app_a: msg.bolt should be bolt_a");

    let pairs_b = app_b.world().resource::<CapturedHitPairs>();
    assert_eq!(
        pairs_b.0.len(),
        1,
        "app_b: expected 1 BoltImpactBreaker, got {}",
        pairs_b.0.len()
    );
    assert_eq!(pairs_b.0[0].0, bolt_b, "app_b: msg.bolt should be bolt_b");

    let vel_a = app_a.world().get::<Velocity2D>(bolt_a).unwrap().0;
    let vel_b = app_b.world().get::<Velocity2D>(bolt_b).unwrap().0;
    assert!(
        vel_a.y > 0.0,
        "normal bolt vy should be positive after real breaker reflect, got {}",
        vel_a.y
    );
    assert!(
        vel_b.y > 0.0,
        "phantom bolt vy should be positive after real breaker reflect, got {}",
        vel_b.y
    );
    assert!(
        (vel_a.x - vel_b.x).abs() < 1e-3,
        "vx must be equal: normal={}, phantom={}",
        vel_a.x,
        vel_b.x
    );
    assert!(
        (vel_a.y - vel_b.y).abs() < 1e-3,
        "vy must be equal: normal={}, phantom={}",
        vel_a.y,
        vel_b.y
    );

    // Top surface: side = ImpactSide::Top, position.y = breaker_y + half_h = -250 + 10 = -240
    let li_a = app_a
        .world()
        .get::<LastImpact>(bolt_a)
        .expect("normal bolt should have LastImpact");
    let li_b = app_b
        .world()
        .get::<LastImpact>(bolt_b)
        .expect("phantom bolt should have LastImpact");
    assert_eq!(
        li_a.side,
        ImpactSide::Top,
        "normal bolt LastImpact.side should be Top"
    );
    assert_eq!(
        li_b.side,
        ImpactSide::Top,
        "phantom bolt LastImpact.side should be Top"
    );
    assert!(
        (li_a.position.x - 0.0).abs() < 1e-3 && (li_a.position.y - (-240.0)).abs() < 1e-3,
        "normal bolt LastImpact.position should be (0.0, -240.0), got {:?}",
        li_a.position
    );
    assert!(
        (li_b.position.x - 0.0).abs() < 1e-3 && (li_b.position.y - (-240.0)).abs() < 1e-3,
        "phantom bolt LastImpact.position should be (0.0, -240.0), got {:?}",
        li_b.position
    );

    let pr_a = app_a
        .world()
        .get::<PiercingRemaining>(bolt_a)
        .expect("normal bolt should have PiercingRemaining");
    let pr_b = app_b
        .world()
        .get::<PiercingRemaining>(bolt_b)
        .expect("phantom bolt should have PiercingRemaining");
    assert_eq!(
        pr_a.0, 3,
        "normal bolt PiercingRemaining should be reset to 3"
    );
    assert_eq!(
        pr_b.0, 3,
        "phantom bolt PiercingRemaining should be reset to 3"
    );
}

#[test]
fn phantom_bolt_emits_bolt_impact_breaker_on_real_breaker_identically_to_normal_bolt() {
    let mut app_a = make_app_with_collector();
    let _breaker_a = spawn_breaker_at(&mut app_a, 0.0, breaker_y());
    let bolt_a = spawn_bolt(&mut app_a, 0.0, start_y_above_real_breaker(), 0.0, -400.0);
    app_a
        .world_mut()
        .entity_mut(bolt_a)
        .insert((piercing_stack(&[3]), PiercingRemaining(0)));

    let mut app_b = make_app_with_collector();
    let _breaker_b = spawn_breaker_at(&mut app_b, 0.0, breaker_y());
    let bolt_b = spawn_phantom_bolt(&mut app_b, 0.0, start_y_above_real_breaker(), 0.0, -400.0);
    app_b
        .world_mut()
        .entity_mut(bolt_b)
        .insert((piercing_stack(&[3]), PiercingRemaining(0)));

    tick(&mut app_a);
    tick(&mut app_b);

    assert_real_breaker_center_hit(&app_a, &app_b, bolt_a, bolt_b);
}

// Edge case: off-center hit — both bolts deflect identically
#[test]
fn phantom_bolt_off_center_real_breaker_hit_deflects_identically_to_normal_bolt() {
    let start_y = start_y_above_real_breaker();
    // Hit at x=-50, breaker at x=0, half_w=60 → fraction = -50/60 ≈ -0.833 → vel.x < 0

    let mut app_a = make_app_with_collector();
    let _breaker_a = spawn_breaker_at(&mut app_a, 0.0, breaker_y());
    let bolt_a = spawn_bolt(&mut app_a, -50.0, start_y, 0.0, -400.0);

    let mut app_b = make_app_with_collector();
    let _breaker_b = spawn_breaker_at(&mut app_b, 0.0, breaker_y());
    let bolt_b = spawn_phantom_bolt(&mut app_b, -50.0, start_y, 0.0, -400.0);

    tick(&mut app_a);
    tick(&mut app_b);

    let pairs_a = app_a.world().resource::<CapturedHitPairs>();
    assert_eq!(
        pairs_a.0.len(),
        1,
        "app_a: expected BoltImpactBreaker on off-center hit"
    );
    let pairs_b = app_b.world().resource::<CapturedHitPairs>();
    assert_eq!(
        pairs_b.0.len(),
        1,
        "app_b: expected BoltImpactBreaker on off-center hit"
    );

    let vel_a = app_a.world().get::<Velocity2D>(bolt_a).unwrap().0;
    let vel_b = app_b.world().get::<Velocity2D>(bolt_b).unwrap().0;

    assert!(
        vel_a.x < 0.0,
        "normal bolt vx should be negative for left-of-center hit, got {}",
        vel_a.x
    );
    assert!(
        vel_b.x < 0.0,
        "phantom bolt vx should be negative for left-of-center hit, got {}",
        vel_b.x
    );
    assert!(
        (vel_a.x - vel_b.x).abs() < 1e-3,
        "vx must be equal: normal={}, phantom={}",
        vel_a.x,
        vel_b.x
    );
    assert!(
        (vel_a.y - vel_b.y).abs() < 1e-3,
        "vy must be equal: normal={}, phantom={}",
        vel_a.y,
        vel_b.y
    );

    // LastImpact position for off-center hit
    let li_a = app_a
        .world()
        .get::<LastImpact>(bolt_a)
        .expect("normal bolt should have LastImpact");
    let li_b = app_b
        .world()
        .get::<LastImpact>(bolt_b)
        .expect("phantom bolt should have LastImpact");
    assert!(
        (li_a.position.x - (-50.0)).abs() < 1e-3 && (li_a.position.y - (-240.0)).abs() < 1e-3,
        "normal bolt LastImpact.position should be (-50.0, -240.0), got {:?}",
        li_a.position
    );
    assert!(
        (li_b.position.x - (-50.0)).abs() < 1e-3 && (li_b.position.y - (-240.0)).abs() < 1e-3,
        "phantom bolt LastImpact.position should be (-50.0, -240.0), got {:?}",
        li_b.position
    );
}

// Edge case: no piercing components — both still emit BoltImpactBreaker; neither has PiercingRemaining
#[test]
fn phantom_bolt_no_piercing_real_breaker_hit_same_as_normal_bolt() {
    let start_y = start_y_above_real_breaker();

    let mut app_a = make_app_with_collector();
    spawn_breaker_at(&mut app_a, 0.0, breaker_y());
    let bolt_a = spawn_bolt(&mut app_a, 0.0, start_y, 0.0, -400.0);

    let mut app_b = make_app_with_collector();
    spawn_breaker_at(&mut app_b, 0.0, breaker_y());
    let bolt_b = spawn_phantom_bolt(&mut app_b, 0.0, start_y, 0.0, -400.0);

    tick(&mut app_a);
    tick(&mut app_b);

    let pairs_a = app_a.world().resource::<CapturedHitPairs>();
    assert_eq!(
        pairs_a.0.len(),
        1,
        "normal bolt: expected BoltImpactBreaker without piercing"
    );
    let pairs_b = app_b.world().resource::<CapturedHitPairs>();
    assert_eq!(
        pairs_b.0.len(),
        1,
        "phantom bolt: expected BoltImpactBreaker without piercing"
    );

    assert!(
        app_a.world().get::<LastImpact>(bolt_a).is_some(),
        "normal bolt should have LastImpact"
    );
    assert!(
        app_b.world().get::<LastImpact>(bolt_b).is_some(),
        "phantom bolt should have LastImpact"
    );

    assert!(
        app_a.world().get::<PiercingRemaining>(bolt_a).is_none(),
        "normal bolt should NOT have PiercingRemaining without piercing stack"
    );
    assert!(
        app_b.world().get::<PiercingRemaining>(bolt_b).is_none(),
        "phantom bolt should NOT have PiercingRemaining without piercing stack"
    );
}

// Edge case: tilt on real breaker applies to both phantom and non-phantom bolt equally
#[test]
fn phantom_bolt_real_breaker_with_tilt_deflects_identically_to_normal_bolt() {
    let start_y = start_y_above_real_breaker();

    let mut app_a = make_app_with_collector();
    let breaker_a = spawn_breaker_at(&mut app_a, 0.0, breaker_y());
    app_a.world_mut().entity_mut(breaker_a).insert(BreakerTilt {
        angle:       0.3,
        ease_start:  0.0,
        ease_target: 0.0,
    });
    let bolt_a = spawn_bolt(&mut app_a, 0.0, start_y, 0.0, -400.0);

    let mut app_b = make_app_with_collector();
    let breaker_b = spawn_breaker_at(&mut app_b, 0.0, breaker_y());
    app_b.world_mut().entity_mut(breaker_b).insert(BreakerTilt {
        angle:       0.3,
        ease_start:  0.0,
        ease_target: 0.0,
    });
    let bolt_b = spawn_phantom_bolt(&mut app_b, 0.0, start_y, 0.0, -400.0);

    tick(&mut app_a);
    tick(&mut app_b);

    let pairs_a = app_a.world().resource::<CapturedHitPairs>();
    assert_eq!(
        pairs_a.0.len(),
        1,
        "normal bolt tilt test: expected BoltImpactBreaker"
    );
    let pairs_b = app_b.world().resource::<CapturedHitPairs>();
    assert_eq!(
        pairs_b.0.len(),
        1,
        "phantom bolt tilt test: expected BoltImpactBreaker"
    );

    let vel_a = app_a.world().get::<Velocity2D>(bolt_a).unwrap().0;
    let vel_b = app_b.world().get::<Velocity2D>(bolt_b).unwrap().0;

    // Tilt is NOT gated on bolt side — both get the same non-zero vx from tilt
    assert!(
        (vel_a.x - vel_b.x).abs() < 1e-3,
        "tilt effect must be identical for normal and phantom bolt on real breaker: vx_a={}, vx_b={}",
        vel_a.x,
        vel_b.x
    );
    assert!(
        (vel_a.y - vel_b.y).abs() < 1e-3,
        "vy must match: normal={}, phantom={}",
        vel_a.y,
        vel_b.y
    );
}
