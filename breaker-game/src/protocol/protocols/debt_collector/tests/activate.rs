//! Group A — `activate` lifecycle (Behaviors 1–4).
//!
//! Pins that matching `ProtocolTuning::DebtCollector` inserts
//! `DebtCollectorConfig` with fields passed through verbatim; mismatched
//! tuning is a no-op; repeat activates are last-write-wins; and mismatched
//! activate after a matched activate preserves the earlier config.

use bevy::prelude::{Commands, Update};

use super::{
    super::system::DebtCollectorConfig,
    helpers::{activate_now, canonical_debt_collector_config, install_debt_collector_config},
};
use crate::{prelude::*, protocol::definition::ProtocolTuning};

// ── Behavior 1 — matching tuning inserts exact value ────────────────────────
//
// Ported verbatim from the pre-split single-file scaffold.

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        super::super::system::activate(
            &ProtocolTuning::DebtCollector {
                stack_per_bump: 0.1,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<DebtCollectorConfig>();
    assert!((cfg.stack_per_bump - 0.1).abs() < f32::EPSILON);
}

// ── Behavior 1 (edge case) — non-trivial values pass through verbatim ──────-

#[test]
fn activate_passes_non_trivial_values_through_verbatim() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::DebtCollector {
            stack_per_bump: 0.75,
        },
    );

    let cfg = app
        .world()
        .get_resource::<DebtCollectorConfig>()
        .expect("DebtCollectorConfig should be inserted after matching activate");
    assert!(
        (cfg.stack_per_bump - 0.75).abs() < 1e-4,
        "stack_per_bump expected 0.75 (verbatim), got {}",
        cfg.stack_per_bump
    );

    // Edge case: larger authored values round-trip unchanged too.
    activate_now(
        &mut app,
        &ProtocolTuning::DebtCollector {
            stack_per_bump: 2.5,
        },
    );
    let cfg = app
        .world()
        .get_resource::<DebtCollectorConfig>()
        .expect("DebtCollectorConfig must remain present after overwrite");
    assert!(
        (cfg.stack_per_bump - 2.5).abs() < 1e-4,
        "stack_per_bump expected 2.5 (verbatim), got {}",
        cfg.stack_per_bump
    );
}

// ── Behavior 2 — mismatched tuning does not insert config ───────────────────
//
// Ported verbatim from the pre-split single-file scaffold.

#[test]
fn activate_with_mismatched_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        super::super::system::activate(
            &ProtocolTuning::Greed {
                rarity_boost_per_skip: 0.05,
            },
            &mut commands,
        );
    });
    app.update();

    assert!(app.world().get_resource::<DebtCollectorConfig>().is_none());
}

// ── Behavior 2 (edge case) — prior config preserved across mismatched call ─-

#[test]
fn mismatched_activate_preserves_existing_config() {
    let mut app = TestAppBuilder::new().build();
    install_debt_collector_config(&mut app, canonical_debt_collector_config());

    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    let cfg = app
        .world()
        .get_resource::<DebtCollectorConfig>()
        .expect("prior DebtCollectorConfig must remain after mismatched activate");
    assert_eq!(*cfg, canonical_debt_collector_config());
}

// ── Behavior 3 — last-write-wins on matching re-activate ────────────────────

#[test]
fn second_activate_with_debt_collector_tuning_overwrites_prior_config() {
    let mut app = TestAppBuilder::new().build();
    install_debt_collector_config(
        &mut app,
        DebtCollectorConfig {
            stack_per_bump: 0.5,
        },
    );

    activate_now(
        &mut app,
        &ProtocolTuning::DebtCollector {
            stack_per_bump: 1.0,
        },
    );

    let cfg = app
        .world()
        .get_resource::<DebtCollectorConfig>()
        .expect("DebtCollectorConfig should still be present after re-activate");
    assert!(
        (cfg.stack_per_bump - 1.0).abs() < f32::EPSILON,
        "stack_per_bump expected 1.0 after overwrite, got {}",
        cfg.stack_per_bump
    );
}

// ── Behavior 4 — mismatched activate after matched activate preserves cfg ──-

#[test]
fn mismatched_activate_after_matched_activate_preserves_config() {
    let mut app = TestAppBuilder::new().build();

    // First mismatched call — no insert.
    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );
    assert!(app.world().get_resource::<DebtCollectorConfig>().is_none());

    // Matched call — insert canonical.
    activate_now(
        &mut app,
        &ProtocolTuning::DebtCollector {
            stack_per_bump: 0.5,
        },
    );

    // Second mismatched call — must preserve.
    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    let cfg = app
        .world()
        .get_resource::<DebtCollectorConfig>()
        .expect("prior DebtCollectorConfig must be preserved after second mismatched activate");
    assert!(
        (cfg.stack_per_bump - 0.5).abs() < f32::EPSILON,
        "stack_per_bump must remain 0.5 after mismatched activate, got {}",
        cfg.stack_per_bump
    );
}
