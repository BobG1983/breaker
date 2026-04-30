use bevy::prelude::*;

use super::{
    super::helpers::{
        build_afterimage_app, phantom_bolts_owned_by, phantom_breaker_count,
        seed_active_protocols_with_afterimage, spawn_breaker_with_bump_state,
        spawn_phantom_breaker_at, spawn_real_bolt, write_bump_performed,
    },
    helpers::perfect_bump_state,
};
use crate::{
    breaker::{components::DashState, messages::BumpGrade, sets::BreakerSystems},
    prelude::*,
};

// ── I7 — check_phantom_bounce runs BEFORE GradeBump (same-tick consume) ───

#[test]
fn check_phantom_bounce_runs_before_grade_bump_same_tick_consume() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    // Bolt overlapping the phantom AABB top face, moving DOWN — CB1 geometry.
    let real_bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    // Phantom-bolt spawned in the SAME tick because:
    // 1. check_phantom_bounce (.before GradeBump) emitted BumpPerformed
    //    with breaker == phantom_entity.
    // 2. spawn_phantom_bolt (.after GradeBump) consumed it.
    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(
        owned.len(),
        1,
        "in-tick chain: check → GradeBump → spawn must produce one phantom bolt"
    );
}

// ── I8 — spawn_phantom_bolt runs AFTER GradeBump (filters real breaker) ───

#[test]
fn spawn_phantom_bolt_runs_after_grade_bump_filters_real_breaker_bumps() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_breaker = app.world_mut().spawn(Breaker).id();
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    // Ghost producer in BreakerSystems::GradeBump writes BumpPerformed with
    // breaker == real_breaker (NOT a PhantomBreaker).
    app.add_systems(
        FixedUpdate,
        (move |mut w: MessageWriter<BumpPerformed>, mut done: Local<bool>| {
            if *done {
                return;
            }
            w.write(BumpPerformed {
                grade:   BumpGrade::Perfect,
                bolt:    Some(real_bolt),
                breaker: real_breaker,
            });
            *done = true;
        })
        .in_set(BreakerSystems::GradeBump)
        .run_if(in_state(NodeState::Playing)),
    );

    tick(&mut app);

    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(
        owned.len(),
        0,
        "PhantomBreaker marker filter: a Perfect bump attributed to a REAL \
         breaker must NOT spawn a phantom bolt even though spawn runs after \
         GradeBump (which produced the message)"
    );
}

// ── I13 — full in-tick chain: spawn_phantom_breaker → check → spawn_bolt ──

#[test]
fn single_tick_spawns_phantom_breaker_and_phantom_bolt_via_full_chain() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);

    // Real bolt above the origin, moving DOWN into the AABB of the phantom
    // breaker that is about to be spawned at the breaker's position.
    let _breaker_bump = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    // The real breaker with bump state is at position (0,0) by helper default.
    let real_bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0), // overlapping top face
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    // The helper `spawn_breaker_with_bump_state` already spawns the real
    // breaker with `DashState::Idle`. Flip it to `Dashing` to trigger
    // `afterimage_spawn_phantom_breaker` on the Idle → Dashing edge.
    let breaker_entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("real breaker must be spawned by spawn_breaker_with_bump_state");
    app.world_mut()
        .entity_mut(breaker_entity)
        .insert(DashState::Dashing);

    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        1,
        "spawn_phantom_breaker must have fired on the Idle → Dashing edge"
    );
    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(
        owned.len(),
        1,
        "spawn_phantom_bolt must have consumed the Perfect BumpPerformed from check_phantom_bounce"
    );
}

// ── I14 — spawn_phantom_bolt runs BEFORE tick_phantom_lifetime ────────────

#[test]
fn spawn_phantom_bolt_runs_before_tick_phantom_lifetime_in_same_tick() {
    use crate::effect_v3::effects::phantom_bolt::components::PhantomLifetime;

    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(&mut app, phantom, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);

    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(owned.len(), 1);
    let lifetime = app.world().get::<PhantomLifetime>(owned[0]).unwrap().0;
    let expected = 3.0 - 1.0 / 64.0;
    assert!(
        (lifetime - expected).abs() < 1e-4,
        "spawn runs BEFORE tick_phantom_lifetime → lifetime must be ~{expected} \
         (one tick decrement), got {lifetime}"
    );
}
