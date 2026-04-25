//! Group B — `activate` lifecycle (Behaviors 7–12).
//!
//! Pins the tuning-to-config boundary: fractional `ProtocolTuning::Greed`
//! authoring field translates by `* 100.0` into a percent-unit `GreedConfig`.
//! Non-matching tuning variants are no-ops and never touch the resource.

use super::{
    super::system::GreedConfig,
    helpers::{activate_now, canonical_greed_config, install_greed_config},
};
use crate::{mutators::protocols::definition::ProtocolTuning, prelude::*};

// ── Behavior 7 — matching tuning inserts percent-translated config ──────────

#[test]
fn activate_with_matching_tuning_inserts_percent_translated_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.05,
        },
    );

    let cfg = app
        .world()
        .get_resource::<GreedConfig>()
        .expect("GreedConfig should be inserted after matching activate");
    assert!(
        (cfg.rarity_boost_per_skip - 5.0).abs() < f32::EPSILON,
        "rarity_boost_per_skip expected 5.0 (0.05 × 100), got {}",
        cfg.rarity_boost_per_skip
    );
}

// ── Behavior 8 — non-trivial fraction translates accurately ─────────────────

#[test]
fn activate_translates_non_trivial_fraction() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.075,
        },
    );

    let cfg = app
        .world()
        .get_resource::<GreedConfig>()
        .expect("GreedConfig should be inserted");
    assert!(
        (cfg.rarity_boost_per_skip - 7.5).abs() < 1e-4,
        "rarity_boost_per_skip expected 7.5 (0.075 × 100), got {}",
        cfg.rarity_boost_per_skip
    );
}

// ── Behavior 9 — DebtCollector tuning is a no-op ────────────────────────────

#[test]
fn activate_with_debt_collector_tuning_does_not_insert_greed_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::DebtCollector {
            stack_per_bump: 0.1,
        },
    );

    assert!(
        app.world().get_resource::<GreedConfig>().is_none(),
        "non-Greed tuning must not insert GreedConfig"
    );
}

// ── Behavior 10 — Siphon tuning is a no-op ──────────────────────────────────

#[test]
fn activate_with_siphon_tuning_does_not_insert_greed_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    assert!(
        app.world().get_resource::<GreedConfig>().is_none(),
        "Siphon tuning must not bind to Greed's activate branch"
    );
}

// ── Behavior 11 — second activate overwrites prior config ───────────────────

#[test]
fn second_activate_overwrites_prior_greed_config() {
    let mut app = TestAppBuilder::new().build();
    install_greed_config(&mut app, canonical_greed_config());

    activate_now(
        &mut app,
        &ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.10,
        },
    );

    let cfg = app
        .world()
        .get_resource::<GreedConfig>()
        .expect("GreedConfig should still be present after re-activate");
    assert!(
        (cfg.rarity_boost_per_skip - 10.0).abs() < f32::EPSILON,
        "second activate should overwrite (last-write-wins); expected 10.0, got {}",
        cfg.rarity_boost_per_skip
    );
}

// ── Behavior 12 — mismatch after match preserves existing config ────────────

#[test]
fn activate_mismatch_after_match_preserves_existing_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.05,
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
        .get_resource::<GreedConfig>()
        .expect("prior GreedConfig must be preserved when mismatched tuning is applied");
    assert!(
        (cfg.rarity_boost_per_skip - 5.0).abs() < f32::EPSILON,
        "mismatched activate must not touch existing config; expected 5.0, got {}",
        cfg.rarity_boost_per_skip
    );
}
