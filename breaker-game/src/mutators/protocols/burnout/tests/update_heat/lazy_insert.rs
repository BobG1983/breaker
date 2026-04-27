//! C14, C14b — `BurnoutHeat::default()` is lazy-inserted on first tick
//! and not re-reset on subsequent ticks.

use bevy::prelude::*;

use super::{
    super::{
        super::system::BurnoutHeat,
        helpers::{build_burnout_app, read_heat, set_breaker_velocity, set_heat_state},
    },
    common::{MOVING, seed_canonical},
};
use crate::prelude::*;

// ── C14 — Lazy-insert BurnoutHeat::default() on first tick ─────────────────-

#[test]
fn update_heat_lazy_inserts_burnout_heat_on_breakers_lacking_it() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    // Spawn a breaker with NO BurnoutHeat (note: zero velocity, stationary).
    let breaker = app
        .world_mut()
        .spawn((Breaker, Position2D(Vec2::ZERO), Velocity2D(Vec2::ZERO)))
        .id();

    tick(&mut app);

    let h = app
        .world()
        .get::<BurnoutHeat>(breaker)
        .expect("BurnoutHeat should have been lazy-inserted after first tick");
    assert!(
        (h.heat - 0.0).abs() < f32::EPSILON,
        "lazy-inserted heat must be 0.0, got {}",
        h.heat
    );
    assert!(!h.mega_bump_charged);
    assert!(
        h.still_timer >= 0.0,
        "still_timer must be ≥ 0.0, got {}",
        h.still_timer
    );
}

// ── C14b — Second tick does NOT re-insert BurnoutHeat::default() ───────────-

#[test]
fn update_heat_does_not_reset_existing_burnout_heat_on_subsequent_ticks() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = app
        .world_mut()
        .spawn((Breaker, Position2D(Vec2::ZERO), Velocity2D(Vec2::ZERO)))
        .id();
    tick(&mut app);

    set_heat_state(&mut app, breaker, 0.42, 0.11, false);
    set_breaker_velocity(&mut app, breaker, MOVING);
    tick(&mut app);

    let h = read_heat(&app, breaker).expect("BurnoutHeat should still be present");
    // Expected: heat advanced by one-tick fill increment on top of 0.42, proving
    // the existing value was preserved (not reset to default).
    let expected = 0.42 + (1.0 / 64.0) / 4.0;
    assert!(
        (h.heat - expected).abs() < 1e-3,
        "second tick must mutate the existing BurnoutHeat — expected heat ≈ {expected}, got {}",
        h.heat
    );
}
