//! Group E — `activate` lifecycle.
//!
//! Fresh `TestAppBuilder::new().build()` apps with a one-shot `Update`
//! system that calls `activate(...)` via `Commands`.

use bevy::prelude::*;

use super::super::system::{HasteConfig, activate};
use crate::{hazard::definition::HazardTuning, prelude::*};

// ── Behavior 21 — activate with matching Haste tuning inserts config ───

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Haste {
                base_percent:      0.1,
                per_level_percent: 0.05,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<HasteConfig>();
    assert!((cfg.base_percent - 0.1).abs() < f32::EPSILON);
    assert!((cfg.per_level_percent - 0.05).abs() < f32::EPSILON);
}

#[test]
fn activate_round_trips_canonical_tuning_values_into_resource() {
    // Edge: the canonical pair (20.0, 10.0) round-trips unchanged.
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Haste {
                base_percent:      20.0,
                per_level_percent: 10.0,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<HasteConfig>();
    assert!((cfg.base_percent - 20.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_percent - 10.0).abs() < f32::EPSILON);
}

// ── Behavior 22 — activate with mismatched tuning inserts nothing ──────

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
    assert!(app.world().get_resource::<HasteConfig>().is_none());
}

// ── Behavior 23 — second activate overwrites (last-write-wins) ─────────

#[test]
fn second_activate_call_overwrites_prior_haste_config() {
    let mut app = TestAppBuilder::new().build();
    // Pre-insert a stale config so `activate` must overwrite it.
    app.world_mut().insert_resource(HasteConfig {
        base_percent:      5.0,
        per_level_percent: 1.0,
    });
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Haste {
                base_percent:      22.5,
                per_level_percent: 7.5,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<HasteConfig>();
    assert!((cfg.base_percent - 22.5).abs() < f32::EPSILON);
    assert!((cfg.per_level_percent - 7.5).abs() < f32::EPSILON);
}

#[test]
fn second_activate_call_idempotent_when_tuning_unchanged() {
    // Edge: a second `app.update()` fires the registered `Update` system
    // again with the same tuning — values stay at 22.5 / 7.5.
    let mut app = TestAppBuilder::new().build();
    app.world_mut().insert_resource(HasteConfig {
        base_percent:      5.0,
        per_level_percent: 1.0,
    });
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Haste {
                base_percent:      22.5,
                per_level_percent: 7.5,
            },
            &mut commands,
        );
    });
    app.update();
    app.update();

    let cfg = app.world().resource::<HasteConfig>();
    assert!((cfg.base_percent - 22.5).abs() < f32::EPSILON);
    assert!((cfg.per_level_percent - 7.5).abs() < f32::EPSILON);
}
