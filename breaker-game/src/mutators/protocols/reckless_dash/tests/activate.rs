//! Group A — `activate` lifecycle (Behaviors 1–4).
//!
//! Pins that matching `ProtocolTuning::RecklessDash` inserts
//! `RecklessDashConfig` with all three fields passed through verbatim;
//! mismatched tuning is a no-op; repeat activates are last-write-wins; and
//! mismatched activate after a matched activate preserves the earlier
//! config.

use super::{
    super::system::RecklessDashConfig,
    helpers::{activate_now, canonical_reckless_dash_config, install_reckless_dash_config},
};
use crate::{mutators::protocols::definition::ProtocolTuning, prelude::*};

// ── Behavior 1 — matching tuning inserts config with all three fields ──────-

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::RecklessDash {
            risky_zone_start:  0.7,
            damage_multiplier: 4.0,
            double_penalty:    true,
        },
    );

    let cfg = app
        .world()
        .get_resource::<RecklessDashConfig>()
        .expect("RecklessDashConfig should be inserted after matching activate");
    assert!(
        (cfg.risky_zone_start - 0.7).abs() < f32::EPSILON,
        "risky_zone_start verbatim expected 0.7, got {}",
        cfg.risky_zone_start
    );
    assert!(
        (cfg.damage_multiplier - 4.0).abs() < f32::EPSILON,
        "damage_multiplier verbatim expected 4.0, got {}",
        cfg.damage_multiplier
    );
    assert!(
        cfg.double_penalty,
        "double_penalty verbatim expected true, got {}",
        cfg.double_penalty
    );
}

// ── Behavior 1 (edge case) — non-trivial values pass through verbatim ──────-

#[test]
fn activate_passes_non_trivial_values_through_verbatim() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::RecklessDash {
            risky_zone_start:  0.85,
            damage_multiplier: 2.5,
            double_penalty:    false,
        },
    );

    let cfg = app
        .world()
        .get_resource::<RecklessDashConfig>()
        .expect("RecklessDashConfig should be inserted after matching activate");
    assert!(
        (cfg.risky_zone_start - 0.85).abs() < f32::EPSILON,
        "risky_zone_start verbatim expected 0.85, got {}",
        cfg.risky_zone_start
    );
    assert!(
        (cfg.damage_multiplier - 2.5).abs() < f32::EPSILON,
        "damage_multiplier verbatim expected 2.5, got {}",
        cfg.damage_multiplier
    );
    assert!(
        !cfg.double_penalty,
        "double_penalty verbatim expected false, got {}",
        cfg.double_penalty
    );
}

// ── Behavior 2 — mismatched tuning does nothing ────────────────────────────-

#[test]
fn activate_with_mismatched_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.05,
        },
    );

    assert!(
        app.world().get_resource::<RecklessDashConfig>().is_none(),
        "mismatched tuning must not insert RecklessDashConfig"
    );
}

// ── Behavior 2 (edge case) — prior config preserved across mismatched call ─-

#[test]
fn mismatched_activate_preserves_existing_config() {
    let mut app = TestAppBuilder::new().build();
    install_reckless_dash_config(&mut app, canonical_reckless_dash_config());

    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    let cfg = app
        .world()
        .get_resource::<RecklessDashConfig>()
        .expect("prior RecklessDashConfig must remain after mismatched activate");
    assert_eq!(*cfg, canonical_reckless_dash_config());
}

// ── Behavior 3 — last-write-wins on matching re-activate ───────────────────-

#[test]
fn second_activate_with_reckless_dash_tuning_overwrites_prior_config() {
    let mut app = TestAppBuilder::new().build();
    install_reckless_dash_config(&mut app, canonical_reckless_dash_config());

    activate_now(
        &mut app,
        &ProtocolTuning::RecklessDash {
            risky_zone_start:  0.8,
            damage_multiplier: 3.0,
            double_penalty:    false,
        },
    );

    let cfg = app
        .world()
        .get_resource::<RecklessDashConfig>()
        .expect("RecklessDashConfig should still be present after re-activate");
    assert_eq!(
        *cfg,
        RecklessDashConfig {
            risky_zone_start:  0.8,
            damage_multiplier: 3.0,
            double_penalty:    false,
        },
        "last-write-wins overwrite"
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
    assert!(app.world().get_resource::<RecklessDashConfig>().is_none());

    // Matched call — insert canonical.
    activate_now(
        &mut app,
        &ProtocolTuning::RecklessDash {
            risky_zone_start:  0.7,
            damage_multiplier: 4.0,
            double_penalty:    true,
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
        .get_resource::<RecklessDashConfig>()
        .expect("prior RecklessDashConfig must be preserved after second mismatched activate");
    assert_eq!(
        *cfg,
        canonical_reckless_dash_config(),
        "canonical must survive mismatched activate"
    );
}
