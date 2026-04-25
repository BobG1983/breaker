//! Section B — activate lifecycle (frac → percent translation).

use super::{super::system::*, helpers::*};
use crate::{hazard::definition::HazardTuning, prelude::*};

// Behavior 15 — activate with matching tuning inserts translated config.
#[test]
fn activate_inserts_config_with_percent_translated_fields() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Diffusion {
            base_share_frac:      0.20,
            per_level_share_frac: 0.10,
            depth_every_levels:   5,
        },
    );

    let cfg = app.world().resource::<DiffusionConfig>();
    assert!(
        (cfg.base_share_percent - 20.0).abs() < f32::EPSILON,
        "base_share_percent expected 20.0, got {}",
        cfg.base_share_percent
    );
    assert!(
        (cfg.share_per_level_percent - 10.0).abs() < f32::EPSILON,
        "share_per_level_percent expected 10.0, got {}",
        cfg.share_per_level_percent
    );
}

// Behavior 16 — non-trivial fraction translations are preserved within epsilon.
#[test]
fn activate_translates_nontrivial_fractions() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Diffusion {
            base_share_frac:      0.275,
            per_level_share_frac: 0.125,
            depth_every_levels:   7,
        },
    );

    let cfg = app.world().resource::<DiffusionConfig>();
    assert!(
        (cfg.base_share_percent - 27.5).abs() < f32::EPSILON,
        "base_share_percent expected 27.5, got {}",
        cfg.base_share_percent
    );
    assert!(
        (cfg.share_per_level_percent - 12.5).abs() < f32::EPSILON,
        "share_per_level_percent expected 12.5, got {}",
        cfg.share_per_level_percent
    );
}

// Behavior 17 — Decay tuning does nothing.
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
    assert!(app.world().get_resource::<DiffusionConfig>().is_none());
}

// Behavior 18 — Sympathy tuning (most-similar field shape) does nothing.
#[test]
fn activate_with_sympathy_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Sympathy {
            base_heal_frac:      0.25,
            per_level_heal_frac: 0.05,
            depth_every_levels:  5,
        },
    );
    assert!(app.world().get_resource::<DiffusionConfig>().is_none());
}

// Behavior 19 — second activate with matching tuning overwrites (last-write-wins).
#[test]
fn second_activate_overwrites_prior_diffusion_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Diffusion {
            base_share_frac:      0.20,
            per_level_share_frac: 0.10,
            depth_every_levels:   5,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Diffusion {
            base_share_frac:      0.50,
            per_level_share_frac: 0.25,
            depth_every_levels:   3,
        },
    );

    let cfg = app.world().resource::<DiffusionConfig>();
    assert!((cfg.base_share_percent - 50.0).abs() < f32::EPSILON);
    assert!((cfg.share_per_level_percent - 25.0).abs() < f32::EPSILON);
}

// Behavior 20 — mismatch after successful activate preserves existing config.
#[test]
fn activate_mismatch_after_match_preserves_existing_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Diffusion {
            base_share_frac:      0.20,
            per_level_share_frac: 0.10,
            depth_every_levels:   5,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Decay {
            base_percent:      0.05,
            per_level_percent: 0.03,
        },
    );

    let cfg = app.world().resource::<DiffusionConfig>();
    assert!((cfg.base_share_percent - 20.0).abs() < f32::EPSILON);
    assert!((cfg.share_per_level_percent - 10.0).abs() < f32::EPSILON);
}
