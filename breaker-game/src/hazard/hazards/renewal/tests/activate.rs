//! Group H — `activate` / config lifecycle (retrofit-neutral regressions).
//!
//! The existing pre-retrofit tests are keepers; verify they still pass
//! after the split, plus one edge-case matching Cascade's
//! `second_activate_overwrites_prior_config` behaviour.

use bevy::prelude::*;

use super::super::system::{RenewalConfig, activate};
use crate::{hazard::definition::HazardTuning, prelude::*};

// ── Behavior 33 — activate with matching tuning inserts RenewalConfig ────

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Renewal {
                base_period_secs:         10.0,
                per_level_reduction_frac: 0.2,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<RenewalConfig>();
    assert!(
        (cfg.base_period_secs - 10.0).abs() < f32::EPSILON,
        "base_period_secs should be 10.0, got {}",
        cfg.base_period_secs
    );
    assert!(
        (cfg.per_level_reduction_frac - 0.2).abs() < f32::EPSILON,
        "per_level_reduction_frac should be 0.2, got {}",
        cfg.per_level_reduction_frac
    );
}

#[test]
fn activate_round_trips_alternate_tuning_values() {
    // Edge: different tuning values round-trip into the resource unchanged.
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Renewal {
                base_period_secs:         5.5,
                per_level_reduction_frac: 0.1,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<RenewalConfig>();
    assert!((cfg.base_period_secs - 5.5).abs() < f32::EPSILON);
    assert!((cfg.per_level_reduction_frac - 0.1).abs() < f32::EPSILON);
}

// ── Behavior 34 — activate with mismatched tuning is a no-op ────────────

#[test]
fn activate_with_mismatched_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Decay {
                base_percent:      0.05,
                per_level_percent: 0.03,
            },
            &mut commands,
        );
    });
    app.update();

    assert!(
        app.world().get_resource::<RenewalConfig>().is_none(),
        "mismatched tuning must not insert RenewalConfig"
    );
}

// ── Behavior 35 — Second activate overwrites prior RenewalConfig ─────────

#[test]
fn second_activate_overwrites_prior_renewal_config() {
    let mut app = TestAppBuilder::new().build();
    // Pre-insert a starting config.
    app.world_mut().insert_resource(RenewalConfig {
        base_period_secs:         5.0,
        per_level_reduction_frac: 0.1,
    });
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Renewal {
                base_period_secs:         12.5,
                per_level_reduction_frac: 0.25,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<RenewalConfig>();
    assert!(
        (cfg.base_period_secs - 12.5).abs() < f32::EPSILON,
        "second activate must overwrite base_period_secs, got {}",
        cfg.base_period_secs
    );
    assert!(
        (cfg.per_level_reduction_frac - 0.25).abs() < f32::EPSILON,
        "second activate must overwrite per_level_reduction_frac, got {}",
        cfg.per_level_reduction_frac
    );
}
