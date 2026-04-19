//! Section B — `activate` lifecycle (Behaviours 14–19).
//!
//! Pins the tuning-to-config boundary: fractional `HazardTuning::Tether`
//! authoring fields translate by `* 100.0` into percent-unit `TetherConfig`
//! fields. Non-matching tuning variants are no-ops and never touch the
//! resource.

use super::{
    super::system::TetherConfig,
    helpers::{activate_now, canonical_tether_config, install_tether_config},
};
use crate::{hazard::definition::HazardTuning, prelude::*};

// ── Behavior 14 — matching tuning inserts translated config ──────────────────

#[test]
fn activate_inserts_config_with_percent_translated_fields() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Tether {
            base_share_frac:         0.25,
            per_level_share_frac:    0.10,
            base_coverage_frac:      0.40,
            per_level_coverage_frac: 0.10,
        },
    );

    let cfg = app.world().resource::<TetherConfig>();
    assert!(
        (cfg.base_damage - 25.0).abs() < f32::EPSILON,
        "base_damage expected 25.0, got {}",
        cfg.base_damage
    );
    assert!(
        (cfg.damage_per_level - 10.0).abs() < f32::EPSILON,
        "damage_per_level expected 10.0, got {}",
        cfg.damage_per_level
    );
    assert!(
        (cfg.base_coverage - 40.0).abs() < f32::EPSILON,
        "base_coverage expected 40.0, got {}",
        cfg.base_coverage
    );
    assert!(
        (cfg.coverage_per_level - 10.0).abs() < f32::EPSILON,
        "coverage_per_level expected 10.0, got {}",
        cfg.coverage_per_level
    );
}

// ── Behavior 15 — non-trivial fraction translations preserved within epsilon ──

#[test]
fn activate_translates_nontrivial_fractions() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Tether {
            base_share_frac:         0.275,
            per_level_share_frac:    0.125,
            base_coverage_frac:      0.333,
            per_level_coverage_frac: 0.075,
        },
    );

    let cfg = app.world().resource::<TetherConfig>();
    assert!(
        (cfg.base_damage - 27.5).abs() < f32::EPSILON,
        "base_damage expected 27.5, got {}",
        cfg.base_damage
    );
    assert!(
        (cfg.damage_per_level - 12.5).abs() < f32::EPSILON,
        "damage_per_level expected 12.5, got {}",
        cfg.damage_per_level
    );
    assert!(
        // 0.333 * 100.0 has f32 rounding error larger than EPSILON; use 1e-4.
        (cfg.base_coverage - 33.3).abs() < 1e-4,
        "base_coverage expected ≈ 33.3, got {}",
        cfg.base_coverage
    );
    assert!(
        (cfg.coverage_per_level - 7.5).abs() < 1e-4,
        "coverage_per_level expected ≈ 7.5, got {}",
        cfg.coverage_per_level
    );
}

// ── Behavior 16 — Cascade tuning does nothing (no-op + warn) ─────────────────

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
        app.world().get_resource::<TetherConfig>().is_none(),
        "non-Tether tuning must not insert TetherConfig"
    );
}

// ── Behavior 17 — Diffusion tuning (similar shape) does nothing ──────────────

#[test]
fn activate_with_diffusion_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Diffusion {
            base_share_frac:      0.20,
            per_level_share_frac: 0.10,
            depth_every_levels:   5,
        },
    );
    assert!(
        app.world().get_resource::<TetherConfig>().is_none(),
        "Diffusion tuning must NOT bind to Tether's activate branch despite \
         similar field shape"
    );
}

// ── Behavior 18 — second activate overwrites prior config ────────────────────

#[test]
fn second_activate_overwrites_prior_tether_config() {
    let mut app = TestAppBuilder::new().build();
    install_tether_config(&mut app, canonical_tether_config());

    activate_now(
        &mut app,
        &HazardTuning::Tether {
            base_share_frac:         0.50,
            per_level_share_frac:    0.25,
            base_coverage_frac:      0.60,
            per_level_coverage_frac: 0.15,
        },
    );

    let cfg = app.world().resource::<TetherConfig>();
    assert!((cfg.base_damage - 50.0).abs() < 1e-4);
    assert!((cfg.damage_per_level - 25.0).abs() < 1e-4);
    assert!((cfg.base_coverage - 60.0).abs() < 1e-4);
    assert!((cfg.coverage_per_level - 15.0).abs() < 1e-4);
}

// ── Behavior 19 — mismatch after match preserves existing config ─────────────

#[test]
fn activate_mismatch_after_match_preserves_existing_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Tether {
            base_share_frac:         0.25,
            per_level_share_frac:    0.10,
            base_coverage_frac:      0.40,
            per_level_coverage_frac: 0.10,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Decay {
            base_percent:      0.05,
            per_level_percent: 0.03,
        },
    );

    let cfg = app.world().resource::<TetherConfig>();
    assert!((cfg.base_damage - 25.0).abs() < f32::EPSILON);
    assert!((cfg.damage_per_level - 10.0).abs() < f32::EPSILON);
    assert!((cfg.base_coverage - 40.0).abs() < f32::EPSILON);
    assert!((cfg.coverage_per_level - 10.0).abs() < f32::EPSILON);
}
