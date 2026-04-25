//! Group I — `wire` wiring, schedule, gates (Behaviors I1–I14).
//!
//! Pins that `wire`-wired systems run under the correct schedules,
//! gated by `protocol_active(Afterimage)` + `in_state(NodeState::Playing)`,
//! and that the full in-tick chain
//! `spawn_phantom_breaker → check_phantom_bounce → GradeBump →
//! spawn_phantom_bolt → tick_phantom_lifetime` fires in order.

use bevy::prelude::*;

use super::{
    super::system::{AfterimageConfig, PhantomBreaker, PhantomBreakerLifetime},
    helpers::{
        build_afterimage_app, build_afterimage_app_in_chip_selecting,
        build_afterimage_app_no_config, captured_bump_performed, phantom_bolt_count,
        phantom_bolts_owned_by, phantom_breaker_count, seed_active_protocols_with_afterimage,
        spawn_breaker_with_bump_state, spawn_breaker_with_dash, spawn_phantom_breaker_at,
        spawn_real_bolt, tick_n, write_bump_performed,
    },
};
use crate::{
    breaker::{
        components::{BumpState, DashState},
        messages::BumpGrade,
        sets::BreakerSystems,
    },
    prelude::*,
};

// ── Canonical helpers ──────────────────────────────────────────────────────

fn perfect_bump_state() -> BumpState {
    BumpState {
        active:         true,
        timer:          0.1,
        post_hit_timer: 0.0,
        cooldown:       0.0,
        last_hit_bolt:  None,
    }
}

// ── I1 — wire wires spawn_phantom_breaker in FixedUpdate ──────────────

#[test]
fn register_wires_spawn_phantom_breaker_in_fixed_update() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::ZERO);

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        1,
        "afterimage_spawn_phantom_breaker must run via wire"
    );
}

// ── I2 — spawn_phantom_breaker gated OFF when Afterimage not active ───────

#[test]
fn spawn_phantom_breaker_gated_off_when_afterimage_not_active() {
    let mut app = build_afterimage_app();
    // Do NOT seed ActiveProtocols.
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::ZERO);

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "run_if(protocol_active) must gate off afterimage_spawn_phantom_breaker"
    );
}

// ── I3 — spawn_phantom_breaker gated OFF when NodeState != Playing ────────

#[test]
fn spawn_phantom_breaker_gated_off_in_chip_selecting() {
    let mut app = build_afterimage_app_in_chip_selecting();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::ZERO);

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "run_if(in_state(Playing)) must gate off afterimage_spawn_phantom_breaker"
    );
}

// ── I4 — tick_phantom_breaker wired + gated ───────────────────────────────

#[test]
fn tick_phantom_breaker_wired_and_gated_on_active_and_playing() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 0.01);

    tick_n(&mut app, 2);

    assert!(
        app.world().get::<PhantomBreaker>(phantom).is_none(),
        "tick_phantom_breaker must despawn the short-lived phantom"
    );
}

// ── I4 (edge case) — ChipSelecting: tick_phantom_breaker does NOT tick ────

#[test]
fn tick_phantom_breaker_does_not_tick_in_chip_selecting() {
    let mut app = build_afterimage_app_in_chip_selecting();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 2.0);

    tick_n(&mut app, 2);

    let lifetime = app
        .world()
        .get::<PhantomBreakerLifetime>(phantom)
        .expect("phantom must persist — tick system is gated off");
    assert!(
        (lifetime.0 - 2.0).abs() < f32::EPSILON,
        "lifetime must be UNCHANGED in ChipSelecting, got {}",
        lifetime.0
    );
}

// ── I5 — check_phantom_bounce wired + gated on active + Playing ───────────

#[test]
fn check_phantom_bounce_wired_and_gated_on_active_and_playing() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0), // overlapping top face
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "check_phantom_bounce must reflect the bolt via wire, got y={}",
        velocity.0.y
    );
    let bumps = captured_bump_performed(&app);
    assert_eq!(bumps.len(), 1);
    assert_eq!(bumps[0].breaker, phantom);
}

// ── I5 (edge case) — inactive → no reflection ─────────────────────────────

#[test]
fn check_phantom_bounce_gated_off_when_inactive() {
    let mut app = build_afterimage_app();
    // Do NOT seed ActiveProtocols.
    let _phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0), // overlapping top face
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.y - (-400.0)).abs() < 1.0,
        "inactive → no reflection, got velocity.y={}",
        velocity.0.y
    );
    assert!(captured_bump_performed(&app).is_empty());
}

// ── I5 (edge case) — ChipSelecting → no reflection ────────────────────────

#[test]
fn check_phantom_bounce_gated_off_in_chip_selecting() {
    let mut app = build_afterimage_app_in_chip_selecting();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0), // overlapping top face
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.y - (-400.0)).abs() < 1.0,
        "ChipSelecting → no reflection"
    );
    assert!(captured_bump_performed(&app).is_empty());
}

// ── I6 — spawn_phantom_bolt wired + gated on active + Playing ─────────────

#[test]
fn spawn_phantom_bolt_wired_and_gated_on_active_and_playing() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(&mut app, phantom, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);

    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(owned.len(), 1);
}

// ── I6 (edge case) — inactive → no spawn ──────────────────────────────────

#[test]
fn spawn_phantom_bolt_gated_off_when_inactive() {
    let mut app = build_afterimage_app();
    // Do NOT seed ActiveProtocols.
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(&mut app, phantom, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(phantom_bolts_owned_by(&mut app, real_bolt).len(), 0);
}

// ── I6 (edge case) — ChipSelecting → no spawn ─────────────────────────────

#[test]
fn spawn_phantom_bolt_gated_off_in_chip_selecting() {
    let mut app = build_afterimage_app_in_chip_selecting();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(&mut app, phantom, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(phantom_bolts_owned_by(&mut app, real_bolt).len(), 0);
}

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

// ── I9 — tick_phantom_lifetime is live + despawns afterimage phantoms ─────

#[test]
fn tick_phantom_lifetime_is_live_and_despawns_afterimage_phantoms() {
    use crate::effect_v3::effects::phantom_bolt::components::PhantomLifetime;

    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(&mut app, phantom, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);
    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(owned.len(), 1, "phantom bolt spawned");

    // Despawn by manipulating the PhantomLifetime so tick_phantom_lifetime
    // terminates the entity quickly — instead of waiting 3.0 seconds.
    let phantom_entity = owned[0];
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(PhantomLifetime(0.01));

    tick_n(&mut app, 2);

    assert!(
        app.world().get_entity(phantom_entity).is_err(),
        "tick_phantom_lifetime must despawn the afterimage-spawned phantom"
    );
}

// ── I10 — all four afterimage systems gated by active AND Playing ────────-
//
// Covered already by I2/I3/I5/I5-edge/I6-edge. This aggregate test double-
// checks the four gates are all in place by combining both scopes.

#[test]
fn all_four_systems_gated_when_only_in_state_playing_but_not_active() {
    let mut app = build_afterimage_app();
    // ActiveProtocols empty — only in_state(Playing) passes.
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::ZERO);
    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);
    assert_eq!(phantom_breaker_count(&mut app), 0);
}

#[test]
fn all_four_systems_gated_when_only_active_but_not_playing() {
    let mut app = build_afterimage_app_in_chip_selecting();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::ZERO);
    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);
    assert_eq!(phantom_breaker_count(&mut app), 0);
}

// ── I11 — quiet tick safety under the full schedule ───────────────────────

#[test]
fn quiet_tick_safety_under_full_schedule() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);

    tick_n(&mut app, 3);

    assert_eq!(
        *app.world().resource::<AfterimageConfig>(),
        AfterimageConfig {
            phantom_duration:      2.0,
            phantom_bolt_duration: 3.0,
        },
        "AfterimageConfig must remain canonical across quiet ticks"
    );
    assert_eq!(phantom_breaker_count(&mut app), 0);
    assert_eq!(phantom_bolt_count(&mut app), 0);
}

// ── I12 — wire does not panic when AfterimageConfig absent ────────────

#[test]
fn register_does_not_panic_when_afterimage_config_absent() {
    let mut app = build_afterimage_app_no_config();
    seed_active_protocols_with_afterimage(&mut app);

    tick_n(&mut app, 3);

    assert!(
        app.world().get_resource::<AfterimageConfig>().is_none(),
        "wire must not side-effect-insert AfterimageConfig"
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
