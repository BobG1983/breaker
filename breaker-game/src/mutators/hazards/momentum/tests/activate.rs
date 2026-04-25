//! Group B — `activate` lifecycle (Behaviors 7–10).
//!
//! Pins the tuning-to-config boundary: matching `HazardTuning::Momentum`
//! inserts `MomentumConfig` with exact field values; non-matching variants
//! are no-ops; last-write-wins on repeated matching calls; mismatch after
//! match preserves existing config.

use super::{
    super::system::MomentumConfig,
    helpers::{activate_now, canonical_momentum_config, install_momentum_config},
};
use crate::{mutators::hazards::definition::HazardTuning, prelude::*};

// ── Behavior 7 — matching tuning inserts config with exact fields ───────────

#[test]
fn activate_with_matching_tuning_inserts_config_with_both_fields() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Momentum {
            base_hp_per_hit:      10.0,
            per_level_hp_per_hit: 10.0,
        },
    );

    let cfg = app.world().resource::<MomentumConfig>();
    assert!(
        (cfg.base_hp_per_hit - 10.0).abs() < f32::EPSILON,
        "base_hp_per_hit expected 10.0, got {}",
        cfg.base_hp_per_hit
    );
    assert!(
        (cfg.per_level_hp_per_hit - 10.0).abs() < f32::EPSILON,
        "per_level_hp_per_hit expected 10.0, got {}",
        cfg.per_level_hp_per_hit
    );
}

#[test]
fn activate_preserves_fractional_fields_verbatim() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Momentum {
            base_hp_per_hit:      0.5,
            per_level_hp_per_hit: 0.25,
        },
    );

    let cfg = app.world().resource::<MomentumConfig>();
    assert!(
        (cfg.base_hp_per_hit - 0.5).abs() < f32::EPSILON,
        "fractional base must pass through verbatim, got {}",
        cfg.base_hp_per_hit
    );
    assert!(
        (cfg.per_level_hp_per_hit - 0.25).abs() < f32::EPSILON,
        "fractional per_level must pass through verbatim, got {}",
        cfg.per_level_hp_per_hit
    );
}

// ── Behavior 8 — non-matching tuning variants are no-ops ────────────────────

#[test]
fn activate_with_decay_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Decay {
            base_percent:      0.05,
            per_level_percent: 0.03,
        },
    );
    assert!(
        app.world().get_resource::<MomentumConfig>().is_none(),
        "Decay tuning must not insert MomentumConfig"
    );
}

#[test]
fn activate_with_haste_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Haste {
            base_percent:      0.1,
            per_level_percent: 0.05,
        },
    );
    assert!(
        app.world().get_resource::<MomentumConfig>().is_none(),
        "Haste tuning must not insert MomentumConfig"
    );
}

#[test]
fn activate_with_cascade_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Cascade {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    assert!(
        app.world().get_resource::<MomentumConfig>().is_none(),
        "Cascade tuning must not insert MomentumConfig"
    );
}

// ── Behavior 9 — second activate overwrites prior config ────────────────────

#[test]
fn second_activate_overwrites_prior_momentum_config() {
    let mut app = TestAppBuilder::new().build();
    install_momentum_config(
        &mut app,
        MomentumConfig {
            base_hp_per_hit:      5.0,
            per_level_hp_per_hit: 2.0,
        },
    );

    activate_now(
        &mut app,
        &HazardTuning::Momentum {
            base_hp_per_hit:      15.0,
            per_level_hp_per_hit: 7.0,
        },
    );

    let cfg = app.world().resource::<MomentumConfig>();
    assert!(
        (cfg.base_hp_per_hit - 15.0).abs() < f32::EPSILON,
        "second activate must overwrite base_hp_per_hit, got {}",
        cfg.base_hp_per_hit
    );
    assert!(
        (cfg.per_level_hp_per_hit - 7.0).abs() < f32::EPSILON,
        "second activate must overwrite per_level_hp_per_hit, got {}",
        cfg.per_level_hp_per_hit
    );
}

#[test]
fn activate_mismatch_after_match_preserves_existing_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Momentum {
            base_hp_per_hit:      10.0,
            per_level_hp_per_hit: 10.0,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Decay {
            base_percent:      0.05,
            per_level_percent: 0.03,
        },
    );

    let cfg = app.world().resource::<MomentumConfig>();
    assert!(
        (cfg.base_hp_per_hit - 10.0).abs() < f32::EPSILON,
        "mismatched tuning must NOT overwrite existing MomentumConfig; \
         base_hp_per_hit expected 10.0, got {}",
        cfg.base_hp_per_hit
    );
    assert!(
        (cfg.per_level_hp_per_hit - 10.0).abs() < f32::EPSILON,
        "per_level_hp_per_hit expected 10.0 (preserved), got {}",
        cfg.per_level_hp_per_hit
    );
}

// ── Behavior 10 — activate on empty world does not panic ────────────────────

#[test]
fn activate_on_empty_world_does_not_panic() {
    let mut app = TestAppBuilder::new().build();
    assert!(
        app.world().get_resource::<MomentumConfig>().is_none(),
        "sanity: pre-activate world has no MomentumConfig"
    );
    activate_now(
        &mut app,
        &HazardTuning::Momentum {
            base_hp_per_hit:      10.0,
            per_level_hp_per_hit: 10.0,
        },
    );
    assert!(
        app.world().get_resource::<MomentumConfig>().is_some(),
        "post-flush activate must leave MomentumConfig present"
    );
    let _ = canonical_momentum_config();
}
