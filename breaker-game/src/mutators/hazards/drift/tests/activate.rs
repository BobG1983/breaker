//! Group E — `activate` lifecycle.
//!
//! Fresh `TestAppBuilder::new().build()` apps with a one-shot `Update`
//! system that calls `activate(...)` via `Commands`.

use bevy::{ecs::world::CommandQueue, prelude::*};

use super::super::system::{DriftConfig, DriftWind, activate};
use crate::{mutators::hazards::definition::HazardTuning, prelude::*};

/// Invoke `activate` directly against the world without using the `Update`
/// schedule. Multiple `app.add_systems(Update, closure)` calls in a single
/// test all run on every `app.update()` with undefined ordering, so the
/// "last system" pattern for overwrites is unreliable. This helper builds
/// a standalone `CommandQueue`, invokes `activate`, and applies — giving
/// each call an independent, deterministic flush.
fn activate_now(app: &mut App, tuning: &HazardTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

// ── Behavior 40 — matching tuning inserts both DriftConfig and DriftWind ─

#[test]
fn activate_with_matching_tuning_inserts_config_and_wind() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Drift {
                force:           100.0,
                period_secs:     8.0,
                per_level_force: 33.3,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<DriftConfig>();
    assert!((cfg.force - 100.0).abs() < f32::EPSILON);
    let wind = app.world().resource::<DriftWind>();
    assert!((wind.timer - 0.0).abs() < f32::EPSILON);
}

#[test]
fn activate_with_matching_tuning_pins_all_drift_config_and_wind_fields() {
    // Edge: widen the preserved test to cover every field of both resources.
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Drift {
                force:           100.0,
                period_secs:     8.0,
                per_level_force: 33.3,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<DriftConfig>();
    assert!((cfg.force - 100.0).abs() < f32::EPSILON);
    assert!((cfg.period_secs - 8.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_force - 33.3).abs() < 1e-4);

    let wind = app.world().resource::<DriftWind>();
    assert_eq!(wind.direction, Vec2::X);
    assert_eq!(wind.timer.to_bits(), 0.0_f32.to_bits());
}

// ── Behavior 41 — mismatched tuning variant inserts NEITHER resource ─────

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
    assert!(app.world().get_resource::<DriftConfig>().is_none());
    assert!(app.world().get_resource::<DriftWind>().is_none());
}

#[test]
fn activate_with_haste_tuning_also_inserts_neither() {
    // Edge: mismatch is Drift-specific, not just Decay-specific.
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Haste {
                base_percent:      5.0,
                per_level_percent: 3.0,
            },
            &mut commands,
        );
    });
    app.update();
    assert!(app.world().get_resource::<DriftConfig>().is_none());
    assert!(app.world().get_resource::<DriftWind>().is_none());
}

// ── Behavior 42 — second activate overwrites (last-write-wins) ───────────

#[test]
fn second_activate_overwrites_both_resources() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Drift {
            force:           50.0,
            period_secs:     4.0,
            per_level_force: 10.0,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Drift {
            force:           200.0,
            period_secs:     16.0,
            per_level_force: 66.6,
        },
    );

    let cfg = app.world().resource::<DriftConfig>();
    assert!((cfg.force - 200.0).abs() < f32::EPSILON);
    assert!((cfg.period_secs - 16.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_force - 66.6).abs() < 1e-4);
    let wind = app.world().resource::<DriftWind>();
    assert_eq!(wind.direction, Vec2::X);
    assert_eq!(wind.timer.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn third_activate_also_overwrites_last_write_wins() {
    // Edge: three sequential activates — final values reflect only the
    // latest tuning.
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Drift {
            force:           50.0,
            period_secs:     4.0,
            per_level_force: 10.0,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Drift {
            force:           200.0,
            period_secs:     16.0,
            per_level_force: 66.6,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Drift {
            force:           500.0,
            period_secs:     2.0,
            per_level_force: 0.0,
        },
    );

    let cfg = app.world().resource::<DriftConfig>();
    assert!((cfg.force - 500.0).abs() < f32::EPSILON);
    assert!((cfg.period_secs - 2.0).abs() < f32::EPSILON);
    assert!(cfg.per_level_force.abs() < f32::EPSILON);
}

// ── Behavior 43 — re-activate after manual mutation resets DriftWind ─────

#[test]
fn activate_resets_drift_wind_after_manual_mutation() {
    let mut app = TestAppBuilder::new().build();
    app.world_mut().insert_resource(DriftWind {
        direction: Vec2::Y,
        timer:     3.7,
    });
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Drift {
                force:           100.0,
                period_secs:     8.0,
                per_level_force: 33.3,
            },
            &mut commands,
        );
    });
    app.update();

    let wind = app.world().resource::<DriftWind>();
    assert_eq!(
        wind.direction,
        Vec2::X,
        "activate should reset direction to Vec2::X"
    );
    assert_eq!(
        wind.timer.to_bits(),
        0.0_f32.to_bits(),
        "activate should reset timer to 0.0"
    );
}

#[test]
fn activate_overwrites_manually_mutated_drift_config() {
    // Edge: manual DriftConfig mutation is replaced by the new tuning's values.
    let mut app = TestAppBuilder::new().build();
    app.world_mut().insert_resource(DriftConfig {
        force:           999.0,
        period_secs:     999.0,
        per_level_force: 999.0,
    });
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Drift {
                force:           100.0,
                period_secs:     8.0,
                per_level_force: 33.3,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<DriftConfig>();
    assert!((cfg.force - 100.0).abs() < f32::EPSILON);
    assert!((cfg.period_secs - 8.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_force - 33.3).abs() < 1e-4);
}

// ── Behavior 44 — activate on empty world does not panic ─────────────────

#[test]
fn activate_on_empty_world_does_not_panic() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Drift {
                force:           100.0,
                period_secs:     8.0,
                per_level_force: 33.3,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<DriftConfig>();
    assert!((cfg.force - 100.0).abs() < f32::EPSILON);
    let wind = app.world().resource::<DriftWind>();
    assert_eq!(wind.direction, Vec2::X);
    assert_eq!(wind.timer.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn activate_mismatch_after_match_preserves_prior_resources() {
    // Edge: a Drift activate then a Decay activate on the same app —
    // Decay warns+no-ops, leaving the Drift resources intact.
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Drift {
                force:           100.0,
                period_secs:     8.0,
                per_level_force: 33.3,
            },
            &mut commands,
        );
    });
    app.update();

    // Replace the one-shot with a mismatch; prior resources must survive.
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

    let cfg = app.world().resource::<DriftConfig>();
    assert!((cfg.force - 100.0).abs() < f32::EPSILON);
    assert!((cfg.period_secs - 8.0).abs() < f32::EPSILON);
    let wind = app.world().resource::<DriftWind>();
    assert_eq!(wind.direction, Vec2::X);
}
