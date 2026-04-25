//! Group B — `activate` lifecycle (Behaviors 18–25).
//!
//! Pins the tuning-to-config boundary: matching `HazardTuning::Sympathy`
//! inserts `SympathyConfig` with the ×100 percent-translation applied;
//! non-matching variants are no-ops; last-write-wins on repeated matching
//! calls; mismatch AFTER match preserves the existing config; and activation
//! on an empty world does not panic.

use super::{
    super::system::SympathyConfig,
    helpers::{activate_now, install_sympathy_config},
};
use crate::{mutators::hazards::definition::HazardTuning, prelude::*};

// ── Behavior 18 — matching tuning inserts SympathyConfig with ×100 ──────────

#[test]
fn activate_with_matching_tuning_inserts_config_with_percent_translation() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Sympathy {
            base_heal_frac:      0.25,
            per_level_heal_frac: 0.05,
            depth_every_levels:  5,
        },
    );

    let cfg = app
        .world()
        .get_resource::<SympathyConfig>()
        .expect("SympathyConfig must be inserted after matching activate");
    assert!(
        (cfg.base_heal_percent - 25.0).abs() < f32::EPSILON,
        "base_heal_percent must be 25.0 after 0.25 × 100; got {}",
        cfg.base_heal_percent
    );
    assert!(
        (cfg.heal_per_level_percent - 5.0).abs() < f32::EPSILON,
        "heal_per_level_percent must be 5.0 after 0.05 × 100; got {}",
        cfg.heal_per_level_percent
    );
    assert_eq!(
        cfg.depth_increase_interval, 5,
        "depth_increase_interval must pass through unchanged"
    );
}

// ── Behavior 19 — fractional rounding pins the ×100 translation ─────────────

#[test]
fn activate_fractional_rounding_under_percent_translation() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Sympathy {
            base_heal_frac:      0.123,
            per_level_heal_frac: 0.021,
            depth_every_levels:  3,
        },
    );

    let cfg = app.world().resource::<SympathyConfig>();
    // 0.123 × 100 ≠ 12.3 exactly in f32 — use wider tolerance.
    assert!(
        (cfg.base_heal_percent - 12.3).abs() < 1e-5,
        "base_heal_percent expected ~12.3, got {}",
        cfg.base_heal_percent
    );
    assert!(
        (cfg.heal_per_level_percent - 2.1).abs() < 1e-5,
        "heal_per_level_percent expected ~2.1, got {}",
        cfg.heal_per_level_percent
    );
    assert_eq!(cfg.depth_increase_interval, 3);
}

// ── Behavior 20 — Cascade tuning is a no-op ─────────────────────────────────

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
        app.world().get_resource::<SympathyConfig>().is_none(),
        "Cascade tuning must not insert SympathyConfig"
    );
}

// ── Behavior 21 — Diffusion tuning is a no-op ───────────────────────────────

#[test]
fn activate_with_diffusion_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Diffusion {
            base_share_frac:      0.2,
            per_level_share_frac: 0.1,
            depth_every_levels:   5,
        },
    );
    assert!(
        app.world().get_resource::<SympathyConfig>().is_none(),
        "Diffusion tuning must not insert SympathyConfig"
    );
}

// ── Behavior 22 — Haste tuning is a no-op ───────────────────────────────────

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
        app.world().get_resource::<SympathyConfig>().is_none(),
        "Haste tuning must not insert SympathyConfig"
    );
}

// ── Behavior 23 — second matching activate overwrites prior config ──────────

#[test]
fn second_activate_overwrites_prior_sympathy_config() {
    let mut app = TestAppBuilder::new().build();
    install_sympathy_config(
        &mut app,
        SympathyConfig {
            base_heal_percent:       10.0,
            heal_per_level_percent:  2.0,
            depth_increase_interval: 3,
        },
    );

    activate_now(
        &mut app,
        &HazardTuning::Sympathy {
            base_heal_frac:      0.25,
            per_level_heal_frac: 0.05,
            depth_every_levels:  5,
        },
    );

    let cfg = app.world().resource::<SympathyConfig>();
    assert!((cfg.base_heal_percent - 25.0).abs() < f32::EPSILON);
    assert!((cfg.heal_per_level_percent - 5.0).abs() < f32::EPSILON);
    assert_eq!(cfg.depth_increase_interval, 5);
}

// ── Behavior 24 — mismatched tuning AFTER a matching call preserves config ──

#[test]
fn mismatched_tuning_after_match_preserves_existing_sympathy_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Sympathy {
            base_heal_frac:      0.25,
            per_level_heal_frac: 0.05,
            depth_every_levels:  5,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Decay {
            base_percent:      0.05,
            per_level_percent: 0.03,
        },
    );

    let cfg = app.world().resource::<SympathyConfig>();
    assert!(
        (cfg.base_heal_percent - 25.0).abs() < f32::EPSILON,
        "mismatched Decay tuning must NOT overwrite SympathyConfig; got base={}",
        cfg.base_heal_percent
    );
    assert!(
        (cfg.heal_per_level_percent - 5.0).abs() < f32::EPSILON,
        "heal_per_level_percent must remain 5.0 (preserved); got {}",
        cfg.heal_per_level_percent
    );
    assert_eq!(cfg.depth_increase_interval, 5);
}

// ── Behavior 25 — activate on empty world does not panic ────────────────────

#[test]
fn activate_on_empty_world_does_not_panic() {
    let mut app = TestAppBuilder::new().build();
    assert!(
        app.world().get_resource::<SympathyConfig>().is_none(),
        "sanity: pre-activate world has no SympathyConfig"
    );
    activate_now(
        &mut app,
        &HazardTuning::Sympathy {
            base_heal_frac:      0.25,
            per_level_heal_frac: 0.05,
            depth_every_levels:  5,
        },
    );
    assert!(
        app.world().get_resource::<SympathyConfig>().is_some(),
        "post-flush activate must leave SympathyConfig present"
    );
}
