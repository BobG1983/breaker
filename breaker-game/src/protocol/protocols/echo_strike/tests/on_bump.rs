//! Group C — `echo_strike_on_bump` (Behaviors 8–15).
//!
//! Pins the `BumpPerformed` consumer:
//! - Perfect bump inserts `EchoPrimed` on the targeted bolt.
//! - Early / Late bumps do NOT prime.
//! - `bolt: None` is ignored (no priming).
//! - Re-priming an already-primed bolt is idempotent.
//! - Multi-bolt priming is independent.
//! - Despawned bolts are tolerated.
//! - Harness-safe: `reader.clear()` + early-return when `EchoStrikeConfig`
//!   absent.

use bevy::prelude::*;

use super::{
    super::system::EchoPrimed,
    helpers::{
        build_echo_strike_app, build_echo_strike_app_no_config, install_echo_primed,
        seed_active_protocols_with_echo_strike, spawn_bolt_with_base_damage, write_bump_performed,
    },
};
use crate::{breaker::messages::BumpGrade, prelude::*};

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_echo_strike(app, 3, 0.5, 0.25, 0.1);
}

// ── Behavior 8 — Perfect bump primes the bolt ───────────────────────────────

#[test]
fn perfect_bump_inserts_echo_primed_on_bolt() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_some(),
        "Perfect bump should insert EchoPrimed on the bolt"
    );
}

// ── Behavior 9 — Early bump does NOT prime ─────────────────────────────────-

#[test]
fn early_bump_does_not_insert_echo_primed() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Early);
    tick(&mut app);

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "Early bump must NOT insert EchoPrimed"
    );
}

// ── Behavior 10 — Late bump does NOT prime ─────────────────────────────────-

#[test]
fn late_bump_does_not_insert_echo_primed() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Late);
    tick(&mut app);

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "Late bump must NOT insert EchoPrimed"
    );
}

// ── Behavior 11 — bump with bolt: None primes nothing ──────────────────────-

#[test]
fn perfect_bump_with_bolt_none_primes_no_bolt() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, None, BumpGrade::Perfect);
    tick(&mut app); // must not panic

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "bolt: None must not prime any existing bolt"
    );
}

// ── Behavior 12 — re-priming an already-primed bolt is idempotent ──────────-

#[test]
fn re_priming_already_primed_bolt_is_idempotent() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_echo_primed(&mut app, bolt);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_some(),
        "already-primed bolt should still have EchoPrimed after re-prime"
    );
}

// ── Behavior 12 edge (12a) — two Perfect bumps same tick ───────────────────-

#[test]
fn two_perfect_bumps_same_tick_is_idempotent() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);
    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);
    tick(&mut app); // must not panic

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_some(),
        "two same-tick Perfect bumps should still leave EchoPrimed present"
    );
}

// ── Behavior 13 — multiple bolts receive EchoPrimed independently ──────────-

#[test]
fn multiple_bolts_receive_echo_primed_independently() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_bolt_with_base_damage(&mut app, 10.0);
    let b = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, Some(a), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        app.world().get::<EchoPrimed>(a).is_some(),
        "bolt A should be primed"
    );
    assert!(
        app.world().get::<EchoPrimed>(b).is_none(),
        "bolt B should NOT be primed"
    );
}

// ── Behavior 13 edge (13a) — one Perfect + one Late same tick ──────────────-

#[test]
fn perfect_for_a_and_late_for_b_same_tick_primes_only_a() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_bolt_with_base_damage(&mut app, 10.0);
    let b = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, Some(a), BumpGrade::Perfect);
    write_bump_performed(&mut app, Some(b), BumpGrade::Late);
    tick(&mut app);

    assert!(
        app.world().get::<EchoPrimed>(a).is_some(),
        "bolt A (Perfect) should be primed"
    );
    assert!(
        app.world().get::<EchoPrimed>(b).is_none(),
        "bolt B (Late) should NOT be primed"
    );
}

// ── Behavior 14 — bump for despawned bolt is tolerated ─────────────────────-

#[test]
fn bump_performed_for_despawned_bolt_is_tolerated() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    app.world_mut().entity_mut(bolt).despawn();

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);
    tick(&mut app); // must not panic

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "despawned bolt should not have EchoPrimed"
    );
}

// ── Behavior 15 — on_bump early-returns + clears reader when config absent ─-

#[test]
fn on_bump_early_returns_and_clears_reader_when_config_absent() {
    let mut app = build_echo_strike_app_no_config();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);
    tick(&mut app); // must not panic

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "bolt must NOT be primed when EchoStrikeConfig absent"
    );

    // Second quiet tick with no new messages — buffered message must NOT be
    // retroactively processed (proves reader.clear() drained it).
    tick(&mut app);
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "second quiet tick must not retroactively prime the bolt"
    );
}
