//! Group A — `activate` lifecycle (Behaviors A1–A4).
//!
//! Pins that matching `ProtocolTuning::Burnout` inserts `BurnoutConfig`
//! with all five fields passed through verbatim; mismatched tuning is a
//! no-op; repeat activates are last-write-wins; and mismatched activate
//! after a matched activate preserves the earlier config.

use super::{
    super::system::config::BurnoutConfig,
    helpers::{activate_now, canonical_burnout_config, install_burnout_config},
};
use crate::{prelude::*, protocol::definition::ProtocolTuning};

// ── A1 — matching tuning inserts config with all five fields ───────────────-

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Burnout {
            fill_duration:               4.0,
            drain_duration:              2.0,
            still_threshold:             1.5,
            full_heat_damage_multiplier: 4.0,
            speed_boost_duration:        2.0,
        },
    );

    let cfg = app
        .world()
        .get_resource::<BurnoutConfig>()
        .expect("BurnoutConfig should be inserted after matching activate");
    assert!(
        (cfg.fill_duration - 4.0).abs() < f32::EPSILON,
        "fill_duration verbatim expected 4.0, got {}",
        cfg.fill_duration
    );
    assert!(
        (cfg.drain_duration - 2.0).abs() < f32::EPSILON,
        "drain_duration verbatim expected 2.0, got {}",
        cfg.drain_duration
    );
    assert!(
        (cfg.still_threshold - 1.5).abs() < f32::EPSILON,
        "still_threshold verbatim expected 1.5, got {}",
        cfg.still_threshold
    );
    assert!(
        (cfg.full_heat_damage_multiplier - 4.0).abs() < f32::EPSILON,
        "full_heat_damage_multiplier verbatim expected 4.0, got {}",
        cfg.full_heat_damage_multiplier
    );
    assert!(
        (cfg.speed_boost_duration - 2.0).abs() < f32::EPSILON,
        "speed_boost_duration verbatim expected 2.0, got {}",
        cfg.speed_boost_duration
    );
}

// ── A1b — non-trivial values pass through verbatim ─────────────────────────-

#[test]
fn activate_passes_non_trivial_values_through_verbatim() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Burnout {
            fill_duration:               6.5,
            drain_duration:              3.2,
            still_threshold:             0.7,
            full_heat_damage_multiplier: 2.5,
            speed_boost_duration:        1.75,
        },
    );

    let cfg = app
        .world()
        .get_resource::<BurnoutConfig>()
        .expect("BurnoutConfig should be inserted after matching activate");
    assert!((cfg.fill_duration - 6.5).abs() < f32::EPSILON);
    assert!((cfg.drain_duration - 3.2).abs() < f32::EPSILON);
    assert!((cfg.still_threshold - 0.7).abs() < f32::EPSILON);
    assert!((cfg.full_heat_damage_multiplier - 2.5).abs() < f32::EPSILON);
    assert!((cfg.speed_boost_duration - 1.75).abs() < f32::EPSILON);
}

// ── A2 — mismatched tuning does nothing ────────────────────────────────────-

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
        app.world().get_resource::<BurnoutConfig>().is_none(),
        "mismatched tuning must not insert BurnoutConfig"
    );
}

// ── A2b — prior config preserved across mismatched call ────────────────────-

#[test]
fn mismatched_activate_preserves_existing_config() {
    let mut app = TestAppBuilder::new().build();
    install_burnout_config(&mut app, canonical_burnout_config());

    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    let cfg = app
        .world()
        .get_resource::<BurnoutConfig>()
        .expect("prior BurnoutConfig must remain after mismatched activate");
    assert_eq!(*cfg, canonical_burnout_config());
}

// ── A3 — last-write-wins on matching re-activate ───────────────────────────-

#[test]
fn second_activate_with_burnout_tuning_overwrites_prior_config() {
    let mut app = TestAppBuilder::new().build();
    install_burnout_config(&mut app, canonical_burnout_config());

    activate_now(
        &mut app,
        &ProtocolTuning::Burnout {
            fill_duration:               10.0,
            drain_duration:              1.0,
            still_threshold:             0.1,
            full_heat_damage_multiplier: 1.0,
            speed_boost_duration:        10.0,
        },
    );

    let cfg = app
        .world()
        .get_resource::<BurnoutConfig>()
        .expect("BurnoutConfig should still be present after re-activate");
    assert_eq!(
        *cfg,
        BurnoutConfig {
            fill_duration:               10.0,
            drain_duration:              1.0,
            still_threshold:             0.1,
            full_heat_damage_multiplier: 1.0,
            speed_boost_duration:        10.0,
        },
        "last-write-wins overwrite"
    );
}

// ── A4 — mismatched activate after matched activate preserves cfg ──────────-

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
    assert!(app.world().get_resource::<BurnoutConfig>().is_none());

    // Matched call — insert canonical.
    activate_now(
        &mut app,
        &ProtocolTuning::Burnout {
            fill_duration:               4.0,
            drain_duration:              2.0,
            still_threshold:             1.5,
            full_heat_damage_multiplier: 4.0,
            speed_boost_duration:        2.0,
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
        .get_resource::<BurnoutConfig>()
        .expect("prior BurnoutConfig must be preserved after second mismatched activate");
    assert_eq!(
        *cfg,
        canonical_burnout_config(),
        "canonical must survive mismatched activate"
    );
}
