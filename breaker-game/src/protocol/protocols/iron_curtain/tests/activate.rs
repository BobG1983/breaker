//! Group A — `activate` lifecycle (Behaviors 1–4).
//!
//! Pins that matching `ProtocolTuning::IronCurtain` inserts
//! `IronCurtainConfig` with fields passed through verbatim; mismatched tuning
//! is a no-op; repeat activates are last-write-wins; and mismatched activate
//! after a matched activate preserves the earlier config.

use bevy::prelude::{Commands, Update};

use super::{
    super::system::IronCurtainConfig,
    helpers::{activate_now, canonical_iron_curtain_config, install_iron_curtain_config},
};
use crate::{prelude::*, protocol::definition::ProtocolTuning};

// ── Behavior 1 — matching tuning inserts exact value ────────────────────────
//
// Ported verbatim from the pre-split single-file scaffold.

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        super::super::system::activate(
            &ProtocolTuning::IronCurtain {
                damage_fraction: 0.25,
                falloff_start:   0.5,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<IronCurtainConfig>();
    assert!((cfg.damage_fraction - 0.25).abs() < f32::EPSILON);
    assert!((cfg.falloff_start - 0.5).abs() < f32::EPSILON);
}

// ── Behavior 1 (edge case) — non-trivial values pass through verbatim ──────-

#[test]
fn activate_passes_non_trivial_values_through_verbatim() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::IronCurtain {
            damage_fraction: 0.75,
            falloff_start:   120.0,
        },
    );

    let cfg = app
        .world()
        .get_resource::<IronCurtainConfig>()
        .expect("IronCurtainConfig should be inserted after matching activate");
    assert!(
        (cfg.damage_fraction - 0.75).abs() < 1e-4,
        "damage_fraction expected 0.75 (verbatim), got {}",
        cfg.damage_fraction
    );
    assert!(
        (cfg.falloff_start - 120.0).abs() < 1e-4,
        "falloff_start expected 120.0 (verbatim), got {}",
        cfg.falloff_start
    );
}

// ── Behavior 2 — mismatched tuning does not insert config ───────────────────
//
// Ported verbatim from the pre-split single-file scaffold.

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

    assert!(app.world().get_resource::<IronCurtainConfig>().is_none());
}

// ── Behavior 2 (edge case) — prior config preserved across mismatched call ─-

#[test]
fn mismatched_activate_preserves_existing_config() {
    let mut app = TestAppBuilder::new().build();
    install_iron_curtain_config(&mut app, canonical_iron_curtain_config());

    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    let cfg = app
        .world()
        .get_resource::<IronCurtainConfig>()
        .expect("prior IronCurtainConfig must remain after mismatched activate");
    assert_eq!(*cfg, canonical_iron_curtain_config());
}

// ── Behavior 3 — last-write-wins on matching re-activate ────────────────────

#[test]
fn second_activate_with_iron_curtain_tuning_overwrites_prior_config() {
    let mut app = TestAppBuilder::new().build();
    install_iron_curtain_config(
        &mut app,
        IronCurtainConfig {
            damage_fraction: 0.5,
            falloff_start:   50.0,
        },
    );

    activate_now(
        &mut app,
        &ProtocolTuning::IronCurtain {
            damage_fraction: 0.1,
            falloff_start:   25.0,
        },
    );

    let cfg = app
        .world()
        .get_resource::<IronCurtainConfig>()
        .expect("IronCurtainConfig should still be present after re-activate");
    assert!(
        (cfg.damage_fraction - 0.1).abs() < f32::EPSILON,
        "damage_fraction expected 0.1 after overwrite, got {}",
        cfg.damage_fraction
    );
    assert!(
        (cfg.falloff_start - 25.0).abs() < f32::EPSILON,
        "falloff_start expected 25.0 after overwrite, got {}",
        cfg.falloff_start
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
    assert!(app.world().get_resource::<IronCurtainConfig>().is_none());

    // Matched call — insert canonical.
    activate_now(
        &mut app,
        &ProtocolTuning::IronCurtain {
            damage_fraction: 0.5,
            falloff_start:   50.0,
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
        .get_resource::<IronCurtainConfig>()
        .expect("prior IronCurtainConfig must be preserved after second mismatched activate");
    assert!(
        (cfg.damage_fraction - 0.5).abs() < f32::EPSILON,
        "damage_fraction must remain 0.5 after mismatched activate, got {}",
        cfg.damage_fraction
    );
    assert!(
        (cfg.falloff_start - 50.0).abs() < f32::EPSILON,
        "falloff_start must remain 50.0 after mismatched activate, got {}",
        cfg.falloff_start
    );
}
