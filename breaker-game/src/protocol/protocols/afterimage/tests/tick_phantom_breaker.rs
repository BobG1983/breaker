//! Group D — `afterimage_tick_phantom_breaker` (Behaviors D1–D5).
//!
//! Pins the per-tick `PhantomBreakerLifetime.0 -= delta_secs` decrement,
//! despawn on `remaining <= 0.0`, multi-phantom independence, harness-safe
//! behaviour when `AfterimageConfig` is absent, and quiet-tick safety.

use bevy::prelude::*;

use super::{
    super::system::{PhantomBreaker, PhantomBreakerLifetime},
    helpers::{
        build_afterimage_app, build_afterimage_app_no_config, phantom_breaker_count,
        seed_active_protocols_with_afterimage, spawn_breaker_with_dash, spawn_phantom_breaker_at,
        tick_n,
    },
};
use crate::{breaker::components::DashState, prelude::*};

// ── D1 — PhantomBreakerLifetime.0 decrements by delta_secs each tick ───────

#[test]
fn phantom_breaker_lifetime_decrements_by_delta_secs_each_tick() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 2.0);

    tick(&mut app);

    let lifetime = app
        .world()
        .get::<PhantomBreakerLifetime>(phantom)
        .expect("PhantomBreakerLifetime must still be present after one tick");
    let expected = 2.0 - 1.0 / 64.0;
    assert!(
        (lifetime.0 - expected).abs() < 1e-5,
        "lifetime expected ~{expected}, got {}",
        lifetime.0
    );
}

// ── D1 (edge case) — 32 ticks = 0.5s → lifetime ≈ 1.5 ─────────────────────

#[test]
fn phantom_breaker_lifetime_after_32_ticks_is_about_1_5() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 2.0);

    tick_n(&mut app, 32);

    let lifetime = app
        .world()
        .get::<PhantomBreakerLifetime>(phantom)
        .expect("PhantomBreakerLifetime must still be present after 32 ticks (2.0 - 0.5 = 1.5)");
    assert!(
        (lifetime.0 - 1.5).abs() < 1e-3,
        "lifetime expected ~1.5 after 32 ticks, got {}",
        lifetime.0
    );
}

// ── D2 — PhantomBreaker despawns when lifetime ≤ 0.0 ──────────────────────

#[test]
fn phantom_breaker_despawns_when_lifetime_reaches_zero() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 0.01);

    tick_n(&mut app, 2);

    assert!(
        app.world().get::<PhantomBreaker>(phantom).is_none(),
        "phantom with lifetime 0.01 must despawn after two ticks (~0.031s elapsed)"
    );
}

// ── D2 (edge case) — exactly at threshold despawns on the same tick ────────

#[test]
fn phantom_breaker_exactly_at_threshold_despawns_on_same_tick() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.0 / 64.0);

    tick(&mut app);

    assert!(
        app.world().get::<PhantomBreaker>(phantom).is_none(),
        "lifetime == 1/64 must despawn on the first tick — despawn predicate \
         MUST use `remaining <= 0.0`, not `< 0.0`"
    );
}

// ── D3 — multiple PhantomBreakers tick independently ──────────────────────

#[test]
fn multiple_phantom_breakers_tick_independently() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom_a = spawn_phantom_breaker_at(&mut app, Vec2::new(-20.0, 0.0), 2.0);
    let phantom_b = spawn_phantom_breaker_at(&mut app, Vec2::new(20.0, 0.0), 0.5);

    tick_n(&mut app, 32);

    let a_lifetime = app
        .world()
        .get::<PhantomBreakerLifetime>(phantom_a)
        .expect("phantom_a with initial lifetime 2.0 must still be alive after 32 ticks");
    assert!(
        (a_lifetime.0 - 1.5).abs() < 1e-3,
        "phantom_a lifetime expected ~1.5 after 32 ticks, got {}",
        a_lifetime.0
    );

    assert!(
        app.world().get::<PhantomBreaker>(phantom_b).is_none(),
        "phantom_b with initial lifetime 0.5 must be despawned after 32 ticks (~0.5s)"
    );
}

// ── D4 — harness-safe: AfterimageConfig absent → lifetime unchanged ───────

#[test]
fn absent_afterimage_config_does_not_tick_phantom_breaker_lifetime() {
    let mut app = build_afterimage_app_no_config();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 2.0);

    tick_n(&mut app, 3);

    let lifetime = app
        .world()
        .get::<PhantomBreakerLifetime>(phantom)
        .expect("phantom must still exist — tick system early-returns with no config");
    assert!(
        (lifetime.0 - 2.0).abs() < f32::EPSILON,
        "lifetime must be UNCHANGED when AfterimageConfig is absent — got {} (expected 2.0)",
        lifetime.0
    );
}

// ── D6 — tick-BEFORE-spawn ordering pin (parallel to phantom-bolt F1) ─────

/// Dedicated regression pin for the `tick → spawn → check → bolt` chain
/// order. If `afterimage_spawn_phantom_breaker` ran BEFORE
/// `afterimage_tick_phantom_breaker`, the freshly-written `PhantomBreakerLifetime`
/// would be decremented on the spawn tick and this test's equality-to-full
/// assertion would fail. Complements C1, which verifies the spawn side from
/// `spawn_phantom_breaker.rs` — this one names the ordering invariant
/// directly so a future swap cannot hide in the spawn-group coverage.
#[test]
fn freshly_spawned_phantom_breaker_keeps_full_lifetime_on_spawn_tick() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::ZERO);

    // Drive the Idle → Dashing rising edge so the real spawn system fires.
    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    let world = app.world_mut();
    let mut q = world.query_filtered::<&PhantomBreakerLifetime, With<PhantomBreaker>>();
    let rows: Vec<f32> = q.iter(world).map(|l| l.0).collect();
    assert_eq!(
        rows.len(),
        1,
        "expected exactly one PhantomBreaker after Idle → Dashing"
    );
    assert!(
        (rows[0] - 2.0).abs() < 1e-6,
        "PhantomBreakerLifetime on spawn tick must equal FULL phantom_duration \
         (2.0) — if this is ~{:.4} (= 2.0 - 1/64), the tick-BEFORE-spawn order \
         in `register` was inverted. Got {}",
        2.0_f32 - 1.0 / 64.0,
        rows[0]
    );
}

// ── D5 — quiet tick safety: no phantoms, no panic ─────────────────────────

#[test]
fn quiet_tick_safety_no_phantoms_no_panic() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);

    tick_n(&mut app, 5);

    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "quiet ticks with zero PhantomBreakers must not create any"
    );
}
