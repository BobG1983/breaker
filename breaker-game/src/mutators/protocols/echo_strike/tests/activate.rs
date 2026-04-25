//! Group A — `activate` lifecycle (Behaviors 1–4).
//!
//! Pins that matching `ProtocolTuning::EchoStrike` inserts
//! `EchoStrikeConfig` with all four fields passed through verbatim;
//! mismatched tuning is a no-op; repeat activates are last-write-wins;
//! and mismatched activate after a matched activate preserves the earlier
//! config.

use bevy::prelude::{Commands, Update};

use super::{
    super::system::EchoStrikeConfig,
    helpers::{activate_now, canonical_echo_strike_config, install_echo_strike_config},
};
use crate::{mutators::protocols::definition::ProtocolTuning, prelude::*};

// ── Behavior 1 — matching tuning inserts config with all four fields ────────

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        super::super::system::activate(
            &ProtocolTuning::EchoStrike {
                max_echoes:      3,
                newest_fraction: 0.5,
                middle_fraction: 0.25,
                oldest_fraction: 0.125,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app
        .world()
        .get_resource::<EchoStrikeConfig>()
        .expect("EchoStrikeConfig should be inserted after matching activate");
    assert_eq!(cfg.max_echoes, 3, "max_echoes verbatim");
    assert!(
        (cfg.newest_fraction - 0.5).abs() < f32::EPSILON,
        "newest_fraction expected 0.5, got {}",
        cfg.newest_fraction
    );
    assert!(
        (cfg.middle_fraction - 0.25).abs() < f32::EPSILON,
        "middle_fraction expected 0.25, got {}",
        cfg.middle_fraction
    );
    assert!(
        (cfg.oldest_fraction - 0.125).abs() < f32::EPSILON,
        "oldest_fraction expected 0.125, got {}",
        cfg.oldest_fraction
    );
}

// ── Behavior 1 (edge case) — non-trivial values pass through verbatim ───────

#[test]
fn activate_passes_non_trivial_values_through_verbatim() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::EchoStrike {
            max_echoes:      5,
            newest_fraction: 0.75,
            middle_fraction: 0.4,
            oldest_fraction: 0.2,
        },
    );

    let cfg = app
        .world()
        .get_resource::<EchoStrikeConfig>()
        .expect("EchoStrikeConfig should be inserted after matching activate");
    assert_eq!(cfg.max_echoes, 5, "max_echoes verbatim");
    assert!(
        (cfg.newest_fraction - 0.75).abs() < f32::EPSILON,
        "newest_fraction verbatim expected 0.75, got {}",
        cfg.newest_fraction
    );
    assert!(
        (cfg.middle_fraction - 0.4).abs() < f32::EPSILON,
        "middle_fraction verbatim expected 0.4, got {}",
        cfg.middle_fraction
    );
    assert!(
        (cfg.oldest_fraction - 0.2).abs() < f32::EPSILON,
        "oldest_fraction verbatim expected 0.2, got {}",
        cfg.oldest_fraction
    );
}

// ── Behavior 2 — mismatched tuning does not insert config ───────────────────

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

    assert!(app.world().get_resource::<EchoStrikeConfig>().is_none());
}

// ── Behavior 2 (edge case) — prior config preserved across mismatched call ──

#[test]
fn mismatched_activate_preserves_existing_config() {
    let mut app = TestAppBuilder::new().build();
    install_echo_strike_config(&mut app, canonical_echo_strike_config());

    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    let cfg = app
        .world()
        .get_resource::<EchoStrikeConfig>()
        .expect("prior EchoStrikeConfig must remain after mismatched activate");
    assert_eq!(*cfg, canonical_echo_strike_config());
}

// ── Behavior 3 — last-write-wins on matching re-activate ────────────────────

#[test]
fn second_activate_with_echo_strike_tuning_overwrites_prior_config() {
    let mut app = TestAppBuilder::new().build();
    install_echo_strike_config(
        &mut app,
        EchoStrikeConfig {
            max_echoes:      3,
            newest_fraction: 0.5,
            middle_fraction: 0.25,
            oldest_fraction: 0.1,
        },
    );

    activate_now(
        &mut app,
        &ProtocolTuning::EchoStrike {
            max_echoes:      5,
            newest_fraction: 0.6,
            middle_fraction: 0.3,
            oldest_fraction: 0.15,
        },
    );

    let cfg = app
        .world()
        .get_resource::<EchoStrikeConfig>()
        .expect("EchoStrikeConfig should still be present after re-activate");
    assert_eq!(
        *cfg,
        EchoStrikeConfig {
            max_echoes:      5,
            newest_fraction: 0.6,
            middle_fraction: 0.3,
            oldest_fraction: 0.15,
        },
        "last-write-wins overwrite"
    );
}

// ── Behavior 4 — mismatched activate after matched activate preserves cfg ───

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
    assert!(app.world().get_resource::<EchoStrikeConfig>().is_none());

    // Matched call — insert canonical.
    activate_now(
        &mut app,
        &ProtocolTuning::EchoStrike {
            max_echoes:      3,
            newest_fraction: 0.5,
            middle_fraction: 0.25,
            oldest_fraction: 0.1,
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
        .get_resource::<EchoStrikeConfig>()
        .expect("prior EchoStrikeConfig must be preserved after second mismatched activate");
    assert_eq!(
        *cfg,
        canonical_echo_strike_config(),
        "canonical must survive mismatched activate"
    );
}
