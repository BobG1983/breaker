//! Group C — `reckless_dash_on_bump` risky-catch detection (Behaviors 7–20).
//!
//! Pins the `BumpPerformed` consumer:
//! - Per-message `breakers.get(msg.breaker)` lookup.
//! - Late-dash (progress > `risky_zone_start`) risky catches insert
//!   `RiskyDamageBoost { multiplier: config.damage_multiplier }`.
//! - Mid-dash / non-Dashing / stationary catches do NOT insert.
//! - Strict inequality on the progress threshold — exact equality is NOT
//!   risky.
//! - `BumpGrade` does not affect detection.
//! - `bolt: None`, despawned breaker, and despawned bolt are all tolerated.
//! - Multi-bolt isolation — the bump targets only the bolt named in the
//!   message.
//! - Re-insert on an already-boosted bolt is last-write-wins (Bevy insert
//!   semantics).
//! - `DashDuration(0.0)` is treated as non-risky (no division by zero).
//! - Harness-safe: early return + reader drain when `RecklessDashConfig`
//!   absent.

use bevy::prelude::*;

use super::{
    super::system::RiskyDamageBoost,
    helpers::{
        build_reckless_dash_app, build_reckless_dash_app_no_config, risky_boost,
        seed_active_protocols_with_reckless_dash, spawn_bolt_with_base_damage,
        spawn_breaker_dashing, spawn_breaker_in_state, write_bump_performed,
    },
};
use crate::{
    breaker::{components::DashState, messages::BumpGrade},
    prelude::*,
};

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_reckless_dash(app, 0.7, 4.0, true);
}

// ── Behavior 7 — late-dash risky catch inserts RiskyDamageBoost ────────────-

#[test]
fn late_dash_risky_catch_inserts_risky_damage_boost() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.2); // progress 0.8
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        Some(4.0),
        "late-dash risky catch must insert RiskyDamageBoost with multiplier 4.0"
    );
}

// ── Behavior 7 (edge case) — progress 0.99 still triggers ──────────────────-

#[test]
fn very_late_dash_catch_at_progress_0_99_still_triggers() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.01); // progress 0.99
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        Some(4.0),
        "progress 0.99 must still be risky and insert boost"
    );
}

// ── Behavior 8 — mid-dash (progress 0.5) does NOT insert boost ─────────────-

#[test]
fn mid_dash_non_risky_catch_does_not_insert_boost() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5); // progress 0.5
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "mid-dash catch (progress 0.5) must NOT insert RiskyDamageBoost"
    );
}

// ── Behavior 8 (edge case) — early-dash (progress 0.3) does NOT insert ─────-

#[test]
fn early_dash_non_risky_catch_does_not_insert_boost() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.7); // progress 0.3
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "early-dash catch (progress 0.3) must NOT insert boost"
    );
}

// ── Behavior 9 — Idle (stationary) breaker does NOT insert boost ───────────-

#[test]
fn idle_breaker_catch_does_not_insert_boost() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_in_state(&mut app, DashState::Idle, 1.0, 0.0);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "Idle breaker catch must NOT insert boost"
    );
}

// ── Behavior 10 — Braking breaker does NOT insert boost ────────────────────-

#[test]
fn braking_breaker_catch_does_not_insert_boost() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    // progress fraction would be 0.9, but state is Braking not Dashing.
    let breaker = spawn_breaker_in_state(&mut app, DashState::Braking, 1.0, 0.1);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "Braking breaker catch must NOT insert boost (state matters, not progress)"
    );
}

// ── Behavior 11 — Settling breaker does NOT insert boost ───────────────────-

#[test]
fn settling_breaker_catch_does_not_insert_boost() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_in_state(&mut app, DashState::Settling, 1.0, 0.0);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "Settling breaker catch must NOT insert boost"
    );
}

// ── Behavior 12 — progress EXACTLY at risky_zone_start is NOT risky ────────-

#[test]
fn progress_exactly_at_risky_zone_start_is_not_risky() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.3); // progress exactly 0.7
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "progress exactly equal to risky_zone_start is NOT risky (strict inequality)"
    );
}

// ── Behavior 12 (edge case) — progress just above threshold IS risky ───────-

#[test]
fn progress_just_above_risky_zone_start_is_risky() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    // DashDuration(1.0), remaining 0.3 - 1e-4 → progress ≈ 0.7001 > 0.7.
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.3 - 1e-4);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        Some(4.0),
        "progress just above risky_zone_start MUST insert boost (strict inequality)"
    );
}

// ── Behavior 13 — BumpGrade::Early on late-dash still produces boost ───────-

#[test]
fn early_bump_on_late_dash_still_produces_boost() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.2); // progress 0.8
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Early);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        Some(4.0),
        "Early bump on late-dash catch must still produce boost — grade does not matter"
    );
}

// ── Behavior 13 (edge case) — BumpGrade::Late also produces boost ──────────-

#[test]
fn late_bump_on_late_dash_still_produces_boost() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.2);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Late);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        Some(4.0),
        "Late bump on late-dash catch must still produce boost"
    );
}

// ── Behavior 14 — bolt: None bump is a no-op ───────────────────────────────-

#[test]
fn bump_with_bolt_none_is_a_no_op_no_panic() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.1); // progress 0.9
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0); // unrelated to bump

    write_bump_performed(&mut app, breaker, None, BumpGrade::Perfect);
    tick(&mut app); // must not panic

    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "bolt: None bump must not tag an unrelated bolt"
    );
}

// ── Behavior 15 — despawned breaker is tolerated; no boost inserted ────────-

#[test]
fn despawned_breaker_is_tolerated_no_boost() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.2);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    app.world_mut().entity_mut(breaker).despawn();

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app); // must not panic

    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "despawned breaker lookup must fail → no boost inserted"
    );
}

// ── Behavior 16 — despawned bolt is tolerated; no panic ────────────────────-

#[test]
fn despawned_bolt_is_tolerated_no_panic() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.1); // progress 0.9
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    app.world_mut().entity_mut(bolt).despawn();

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app); // must not panic

    // Despawned bolt has no RiskyDamageBoost to inspect — assert it does not
    // exist (the entity is gone, the component query returns None).
    assert!(
        app.world().get::<RiskyDamageBoost>(bolt).is_none(),
        "despawned bolt must not carry RiskyDamageBoost (no resurrection)"
    );
}

// ── Behavior 17 — multi-bolt isolation ─────────────────────────────────────-

#[test]
fn multi_bolt_isolation_bump_targets_only_the_named_bolt() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.1); // progress 0.9
    let bolt_a = spawn_bolt_with_base_damage(&mut app, 10.0);
    let bolt_b = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt_a), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt_a),
        Some(4.0),
        "bolt_a (named in the bump) must receive the boost"
    );
    assert_eq!(
        risky_boost(&app, bolt_b),
        None,
        "bolt_b (unrelated) must NOT receive the boost"
    );
}

// ── Behavior 18 — re-insert is last-write-wins ─────────────────────────────-

#[test]
fn re_insert_on_already_boosted_bolt_is_last_write_wins() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.1);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    // Pre-install a boost with multiplier 2.0.
    app.world_mut()
        .entity_mut(bolt)
        .insert(RiskyDamageBoost { multiplier: 2.0 });

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        Some(4.0),
        "re-insert with active config multiplier 4.0 must overwrite the 2.0 boost"
    );
}

// ── Behavior 19 — DashDuration(0.0) degenerate case → non-risky, no panic ──-

#[test]
fn dash_duration_zero_is_non_risky_no_panic() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_in_state(&mut app, DashState::Dashing, 0.0, 0.0);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app); // must not panic (no division by zero)

    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "DashDuration(0.0) must be treated as non-risky (can't stretch a zero dash)"
    );
}

// ── Behavior 20 — harness-safe: no config → early return + reader drain ────-

#[test]
fn on_bump_early_returns_and_clears_reader_when_config_absent() {
    let mut app = build_reckless_dash_app_no_config();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.2); // progress 0.8
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app); // tick 1 — config absent → early return

    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "no boost inserted when RecklessDashConfig absent"
    );

    // Second quiet tick — the buffered message must NOT be retro-processed.
    tick(&mut app);
    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "buffered BumpPerformed must not retro-apply after a quiet tick"
    );
}
