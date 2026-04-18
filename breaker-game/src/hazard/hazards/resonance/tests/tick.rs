//! Group G — `resonance_tick_slows` expiry via `ResonanceActiveSlows`.

use std::time::Duration;

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{
    super::system::{ResonanceActiveSlows, resonance_tick_slows},
    helpers::{
        count_stack_entries_with_source, seed_active_slow, spawn_breaker_at,
        test_app_playing_with_effects, tick_with_dt,
    },
};

// ── G1 — remaining > 0 after tick → decremented but retained ────────────

#[test]
fn g1_tick_decrements_remaining_when_above_zero() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(FixedUpdate, resonance_tick_slows);
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));
    seed_active_slow(&mut app, breaker, "hazard:resonance:wave:7", 1.5);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.5));
    app.update();

    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    let entry = slows
        .get("hazard:resonance:wave:7")
        .expect("Entry should still be present");
    assert!(
        (entry.remaining - 1.0).abs() < 1e-5,
        "remaining should decrement to ~1.0, got {}",
        entry.remaining
    );
    assert_eq!(
        entry.multiplier,
        OrderedFloat(0.5),
        "multiplier field must not be mutated by ticking"
    );

    let count = count_stack_entries_with_source(&app, breaker, "hazard:resonance:wave:7");
    assert_eq!(count, 1, "Stack entry should remain");
}

// ── G2 — remaining <= 0 → entry reversed + map drops ───────────────────

#[test]
fn g2_tick_expires_and_reverses_entry() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(FixedUpdate, resonance_tick_slows);
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));
    seed_active_slow(&mut app, breaker, "hazard:resonance:wave:9", 0.1);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.2));
    app.update();

    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    assert!(
        !slows.contains_key("hazard:resonance:wave:9"),
        "Expired entry should be removed; got {:?}",
        slows.keys().collect::<Vec<_>>()
    );
    let count = count_stack_entries_with_source(&app, breaker, "hazard:resonance:wave:9");
    assert_eq!(
        count, 0,
        "Stack entry should be reversed (removed), got {count}"
    );
}

// ── G3 — remaining == 0.0 → treated as expired (inclusive) ─────────────

#[test]
fn g3_tick_treats_zero_as_expired() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(FixedUpdate, resonance_tick_slows);
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));
    seed_active_slow(&mut app, breaker, "hazard:resonance:wave:12", 0.2);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.2));
    app.update();

    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    assert!(
        !slows.contains_key("hazard:resonance:wave:12"),
        "Entry at exactly 0.0 remaining should be treated as expired"
    );
    let count = count_stack_entries_with_source(&app, breaker, "hazard:resonance:wave:12");
    assert_eq!(count, 0, "Stack entry should be reversed at remaining==0");
}

// ── G4 — two different sources expire independently ────────────────────

#[test]
fn g4_two_different_sources_expire_independently() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(FixedUpdate, resonance_tick_slows);
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));
    seed_active_slow(&mut app, breaker, "hazard:resonance:wave:11", 0.3);
    seed_active_slow(&mut app, breaker, "hazard:resonance:wave:22", 0.5);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.4));
    app.update();

    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    assert!(
        !slows.contains_key("hazard:resonance:wave:11"),
        "wave:11 should be expired; remaining keys {:?}",
        slows.keys().collect::<Vec<_>>()
    );
    let c_11 = count_stack_entries_with_source(&app, breaker, "hazard:resonance:wave:11");
    assert_eq!(c_11, 0, "wave:11 stack entry reversed");

    let entry = slows
        .get("hazard:resonance:wave:22")
        .expect("wave:22 should still exist");
    assert!((entry.remaining - 0.1).abs() < 1e-5);
    assert_eq!(entry.multiplier, OrderedFloat(0.5));
    let c_22 = count_stack_entries_with_source(&app, breaker, "hazard:resonance:wave:22");
    assert_eq!(c_22, 1, "wave:22 stack entry retained");
}

// ── G5 — expiry without breaker: drain map, no panic ──────────────────

#[test]
fn g5_tick_expiry_without_breaker_drops_entry_without_panic() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(FixedUpdate, resonance_tick_slows);
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));
    seed_active_slow(&mut app, breaker, "hazard:resonance:wave:77", 0.1);

    // Despawn breaker before the tick.
    app.world_mut().entity_mut(breaker).despawn();

    tick_with_dt(&mut app, Duration::from_secs_f32(2.0));
    app.update();

    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    assert!(
        !slows.contains_key("hazard:resonance:wave:77"),
        "Entry should be removed even with no breaker present"
    );
}
