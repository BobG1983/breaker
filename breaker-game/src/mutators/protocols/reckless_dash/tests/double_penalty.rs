//! Group E — `reckless_dash_double_penalty` bolt-lost amplification
//! (Behaviors 32–40).
//!
//! Pins the `BoltLost` consumer:
//! - Reads `BoltLost`. On a Dashing breaker with `config.double_penalty ==
//!   true`, emits a second `BoltLost` message carrying the original bolt +
//!   breaker.
//! - Non-Dashing breaker or `double_penalty == false` → no duplicate.
//! - Infinite-loop guard: a single `BoltLost` produces exactly one extra,
//!   even across many ticks.
//! - Multi-bolt isolation: two different originals produce two independent
//!   duplications.
//! - Despawned breaker tolerated; no duplicate.
//! - Harness-safe: early return + reader drain when `RecklessDashConfig`
//!   absent.

use bevy::prelude::*;

use super::{
    super::system::RecklessDashConfig,
    helpers::{
        build_reckless_dash_app, build_reckless_dash_app_no_config, captured_bolt_lost,
        install_reckless_dash_config, seed_active_protocols_with_reckless_dash,
        spawn_bolt_with_base_damage, spawn_breaker_dashing, spawn_breaker_in_state,
        write_bolt_lost,
    },
};
use crate::{breaker::components::DashState, prelude::*};

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_reckless_dash(app, 0.7, 4.0, true);
}

// ── Behavior 32 — Dashing + double_penalty:true + BoltLost → 1 extra ───────-

#[test]
fn dashing_breaker_with_double_penalty_true_produces_one_extra_bolt_lost() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let captured = captured_bolt_lost(&app);
    assert_eq!(
        captured.len(),
        2,
        "expected 2 captured BoltLost (original + 1 extra), got {}",
        captured.len()
    );
    for msg in &captured {
        assert_eq!(
            msg.bolt, bolt,
            "every captured BoltLost must carry the bolt"
        );
        assert_eq!(
            msg.breaker, breaker,
            "every captured BoltLost must carry the breaker"
        );
    }
}

// ── Behavior 32 (edge case) — progress 0.99 still duplicates ───────────────-

#[test]
fn dashing_breaker_progress_0_99_still_duplicates() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.01); // progress 0.99
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    assert_eq!(
        captured_bolt_lost(&app).len(),
        2,
        "Dashing state alone determines duplication; progress doesn't matter"
    );
}

// ── Behavior 32 (edge case) — progress 0.01 still duplicates ───────────────-

#[test]
fn dashing_breaker_progress_0_01_still_duplicates() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.99); // progress 0.01
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    assert_eq!(
        captured_bolt_lost(&app).len(),
        2,
        "low progress while Dashing still triggers duplicate"
    );
}

// ── Behavior 33 — Idle breaker + BoltLost → no duplicate ───────────────────-

#[test]
fn idle_breaker_bolt_lost_produces_no_duplicate() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_in_state(&mut app, DashState::Idle, 1.0, 0.0);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    assert_eq!(
        captured_bolt_lost(&app).len(),
        1,
        "Idle breaker must not duplicate BoltLost"
    );
}

// ── Behavior 34 — Settling breaker + BoltLost → no duplicate ───────────────-

#[test]
fn settling_breaker_bolt_lost_produces_no_duplicate() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_in_state(&mut app, DashState::Settling, 1.0, 0.0);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    assert_eq!(
        captured_bolt_lost(&app).len(),
        1,
        "Settling breaker must not duplicate BoltLost"
    );
}

// ── Behavior 35 — Braking breaker + BoltLost → no duplicate ────────────────-

#[test]
fn braking_breaker_bolt_lost_produces_no_duplicate() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_in_state(&mut app, DashState::Braking, 1.0, 0.1);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    assert_eq!(
        captured_bolt_lost(&app).len(),
        1,
        "Braking breaker must not duplicate (state != Dashing)"
    );
}

// ── Behavior 36 — double_penalty:false + Dashing → no duplicate ────────────-

#[test]
fn double_penalty_false_with_dashing_breaker_produces_no_duplicate() {
    let mut app = build_reckless_dash_app();
    install_reckless_dash_config(
        &mut app,
        RecklessDashConfig {
            risky_zone_start:  0.7,
            damage_multiplier: 4.0,
            double_penalty:    false,
        },
    );
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, false);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    assert_eq!(
        captured_bolt_lost(&app).len(),
        1,
        "double_penalty:false must disable the duplicate even while Dashing"
    );
}

// ── Behavior 37 — infinite-loop guard: never more than 1 extra ─────────────-

#[test]
fn single_bolt_lost_produces_exactly_one_extra_across_many_ticks() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    // Tick 1: original + one duplicate captured this frame.
    let tick1 = captured_bolt_lost(&app);
    assert_eq!(
        tick1.len(),
        2,
        "tick 1 must capture original + exactly one duplicate; got {}",
        tick1.len()
    );

    // Ticks 2-5: no new BoltLost messages should be written by the protocol.
    // MessageCollector clears each frame, so a clean frame means len == 0.
    // This is the anti-feedback-loop invariant — the doubled-bolts set
    // prevents re-amplification when the cursor re-reads a duplicate.
    for n in 2..=5 {
        tick(&mut app);
        let captured = captured_bolt_lost(&app);
        assert_eq!(
            captured.len(),
            0,
            "tick {n} must capture zero new BoltLost messages (no re-amplification); got {}",
            captured.len()
        );
    }
}

// ── Behavior 38 — multi-bolt isolation ─────────────────────────────────────-

#[test]
fn two_different_bolt_losts_each_produce_independent_duplicates() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker_a = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let breaker_b = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt_a = spawn_bolt_with_base_damage(&mut app, 10.0);
    let bolt_b = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt_a, breaker_a);
    write_bolt_lost(&mut app, bolt_b, breaker_b);
    tick(&mut app);

    let captured = captured_bolt_lost(&app);
    assert_eq!(captured.len(), 4, "two originals + two duplicates = 4");

    let count_a = captured
        .iter()
        .filter(|m| m.bolt == bolt_a && m.breaker == breaker_a)
        .count();
    let count_b = captured
        .iter()
        .filter(|m| m.bolt == bolt_b && m.breaker == breaker_b)
        .count();
    assert_eq!(
        count_a, 2,
        "exactly 2 with (bolt_a, breaker_a), got {count_a}"
    );
    assert_eq!(
        count_b, 2,
        "exactly 2 with (bolt_b, breaker_b), got {count_b}"
    );
}

// ── Behavior 39 — despawned breaker is tolerated; no duplicate ─────────────-

#[test]
fn despawned_breaker_bolt_lost_produces_no_duplicate() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    app.world_mut().entity_mut(breaker).despawn();

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app); // must not panic

    assert_eq!(
        captured_bolt_lost(&app).len(),
        1,
        "despawned breaker lookup fails → no duplicate (non-Dashing treatment)"
    );
}

// ── Behavior 40 — harness-safe: early return + reader drain when config absent

#[test]
fn double_penalty_early_returns_and_clears_reader_when_config_absent() {
    let mut app = build_reckless_dash_app_no_config();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    assert_eq!(
        captured_bolt_lost(&app).len(),
        1,
        "no duplicate when config absent — only the original is captured"
    );

    // Second quiet tick — buffered BoltLost must NOT retro-duplicate.
    tick(&mut app);
    assert_eq!(
        captured_bolt_lost(&app).len(),
        0,
        "MessageCollector clears each frame; no new messages this tick"
    );
}
