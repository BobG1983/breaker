//! Group A — `activate` lifecycle (Behaviors 1–6).
//!
//! Pins that matching `ProtocolTuning::Siphon` inserts `SiphonConfig` with
//! fields passed through verbatim (no percent translation); mismatched
//! tuning is a no-op; repeat activates last-write-wins; and mismatched
//! activate after a matched activate preserves the earlier config.

use super::{
    super::system::SiphonConfig,
    helpers::{activate_now, canonical_siphon_config, install_siphon_config},
};
use crate::{mutators::protocols::definition::ProtocolTuning, prelude::*};

// ── Behavior 1 — matching Siphon tuning inserts exact values ────────────────

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    let cfg = app.world().resource::<SiphonConfig>();
    assert!((cfg.streak_window - 2.0).abs() < f32::EPSILON);
    assert!((cfg.time_per_kill - 0.25).abs() < f32::EPSILON);
}

// ── Behavior 3 (extra witness) — Greed tuning does not insert SiphonConfig ──

#[test]
fn activate_with_mismatched_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.05,
        },
    );
    assert!(app.world().get_resource::<SiphonConfig>().is_none());
}

// ── Behavior 2b — non-trivial authored values pass through verbatim ─────────
// (Spec Behavior 2 — "Non-trivial authored values pass through verbatim")

#[test]
fn activate_passes_non_trivial_values_through_verbatim() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 3.5,
            time_per_kill: 0.75,
        },
    );

    let cfg = app
        .world()
        .get_resource::<SiphonConfig>()
        .expect("SiphonConfig should be inserted after matching activate");
    assert!(
        (cfg.streak_window - 3.5).abs() < 1e-4,
        "streak_window expected 3.5 (verbatim), got {}",
        cfg.streak_window
    );
    assert!(
        (cfg.time_per_kill - 0.75).abs() < 1e-4,
        "time_per_kill expected 0.75 (verbatim), got {}",
        cfg.time_per_kill
    );

    // Edge case: larger values also round-trip unchanged (no percent coercion).
    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 10.0,
            time_per_kill: 2.0,
        },
    );
    let cfg = app
        .world()
        .get_resource::<SiphonConfig>()
        .expect("SiphonConfig must remain present after overwrite");
    assert!(
        (cfg.streak_window - 10.0).abs() < 1e-4,
        "streak_window expected 10.0 (verbatim), got {}",
        cfg.streak_window
    );
    assert!(
        (cfg.time_per_kill - 2.0).abs() < 1e-4,
        "time_per_kill expected 2.0 (verbatim), got {}",
        cfg.time_per_kill
    );
}

// ── Behavior 3 — Greed tuning does not insert SiphonConfig ──────────────────

#[test]
fn activate_with_greed_tuning_does_not_insert_siphon_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.05,
        },
    );

    assert!(
        app.world().get_resource::<SiphonConfig>().is_none(),
        "non-Siphon tuning must not insert SiphonConfig"
    );

    // Edge case: prior SiphonConfig must be preserved.
    install_siphon_config(&mut app, canonical_siphon_config());
    activate_now(
        &mut app,
        &ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.05,
        },
    );
    let cfg = app
        .world()
        .get_resource::<SiphonConfig>()
        .expect("SiphonConfig inserted prior must remain after mismatched activate");
    assert_eq!(*cfg, canonical_siphon_config());
}

// ── Behavior 4 — DebtCollector tuning does not insert SiphonConfig ─────────-

#[test]
fn activate_with_debt_collector_tuning_does_not_insert_siphon_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::DebtCollector {
            stack_per_bump: 0.1,
        },
    );

    assert!(
        app.world().get_resource::<SiphonConfig>().is_none(),
        "DebtCollector tuning must not bind to Siphon's activate branch"
    );
}

// ── Behavior 5 — second activate overwrites prior config (last-write-wins) ──

#[test]
fn second_activate_with_siphon_tuning_overwrites_prior_config() {
    let mut app = TestAppBuilder::new().build();
    install_siphon_config(
        &mut app,
        SiphonConfig {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 4.0,
            time_per_kill: 0.50,
        },
    );

    let cfg = app
        .world()
        .get_resource::<SiphonConfig>()
        .expect("SiphonConfig should still be present after re-activate");
    assert!(
        (cfg.streak_window - 4.0).abs() < f32::EPSILON,
        "streak_window expected 4.0 after overwrite, got {}",
        cfg.streak_window
    );
    assert!(
        (cfg.time_per_kill - 0.50).abs() < f32::EPSILON,
        "time_per_kill expected 0.50 after overwrite, got {}",
        cfg.time_per_kill
    );
}

// ── Behavior 6 — mismatch after match preserves earlier config ──────────────

#[test]
fn mismatched_activate_after_matched_activate_preserves_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );
    activate_now(
        &mut app,
        &ProtocolTuning::DebtCollector {
            stack_per_bump: 0.1,
        },
    );

    let cfg = app
        .world()
        .get_resource::<SiphonConfig>()
        .expect("prior SiphonConfig must be preserved when mismatched tuning applied");
    assert!(
        (cfg.streak_window - 2.0).abs() < f32::EPSILON,
        "streak_window must remain 2.0 after mismatched activate, got {}",
        cfg.streak_window
    );
    assert!(
        (cfg.time_per_kill - 0.25).abs() < f32::EPSILON,
        "time_per_kill must remain 0.25 after mismatched activate, got {}",
        cfg.time_per_kill
    );
}
