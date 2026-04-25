//! Group A — `activate` lifecycle (Behaviors 1-5).
//!
//! Pins that matching `ProtocolTuning::Fission` inserts `FissionConfig` with
//! fields passed through verbatim; mismatched tuning is a no-op; repeat
//! activates are last-write-wins; and activate does NOT reset `FissionCounter`.

use bevy::prelude::*;

use super::{
    super::system::{FissionConfig, FissionCounter},
    helpers::{activate_now, install_fission_config, install_fission_counter},
};
use crate::{mutators::protocols::definition::ProtocolTuning, prelude::*};

// ── Behavior 1 — matching tuning inserts exact value ────────────────────────

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        super::super::system::activate(
            &ProtocolTuning::Fission {
                kills_per_split: 10,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<FissionConfig>();
    assert_eq!(
        cfg.kills_per_split, 10,
        "kills_per_split must pass through verbatim; got {}",
        cfg.kills_per_split
    );
}

// ── Behavior 1 (edge case) — non-trivial values pass through verbatim ──────-

#[test]
fn activate_passes_non_trivial_values_through_verbatim() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Fission {
            kills_per_split: 42,
        },
    );

    let cfg = app
        .world()
        .get_resource::<FissionConfig>()
        .expect("FissionConfig should be inserted after matching activate");
    assert_eq!(
        cfg.kills_per_split, 42,
        "kills_per_split expected 42 (verbatim); got {}",
        cfg.kills_per_split
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

    assert!(
        app.world().get_resource::<FissionConfig>().is_none(),
        "mismatched tuning must not insert FissionConfig"
    );
}

// ── Behavior 2 (edge case) — IronCurtain tuning also does not insert ───────-

#[test]
fn activate_with_iron_curtain_tuning_does_not_insert_fission_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::IronCurtain {
            damage_fraction: 0.5,
            falloff_start:   50.0,
        },
    );

    assert!(
        app.world().get_resource::<FissionConfig>().is_none(),
        "IronCurtain tuning must not insert FissionConfig"
    );
}

// ── Behavior 3 — last-write-wins on matching re-activate ────────────────────

#[test]
fn second_activate_with_fission_tuning_overwrites_prior_config() {
    let mut app = TestAppBuilder::new().build();
    install_fission_config(&mut app, FissionConfig { kills_per_split: 8 });

    activate_now(&mut app, &ProtocolTuning::Fission { kills_per_split: 3 });

    let cfg = app
        .world()
        .get_resource::<FissionConfig>()
        .expect("FissionConfig should still be present after re-activate");
    assert_eq!(
        cfg.kills_per_split, 3,
        "kills_per_split expected 3 (last-write-wins); got {}",
        cfg.kills_per_split
    );
}

// ── Behavior 4 — mismatched activate preserves existing config ──────────────

#[test]
fn mismatched_activate_preserves_existing_fission_config() {
    let mut app = TestAppBuilder::new().build();
    install_fission_config(&mut app, FissionConfig { kills_per_split: 8 });

    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    let cfg = app
        .world()
        .get_resource::<FissionConfig>()
        .expect("prior FissionConfig must remain after mismatched activate");
    assert_eq!(
        cfg.kills_per_split, 8,
        "kills_per_split must remain 8; got {}",
        cfg.kills_per_split
    );
}

// ── Behavior 5 — activate does NOT reset FissionCounter ─────────────────────

#[test]
fn activate_does_not_reset_fission_counter() {
    let mut app = TestAppBuilder::new().build();
    install_fission_counter(&mut app, 5);

    activate_now(&mut app, &ProtocolTuning::Fission { kills_per_split: 8 });

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 5 },
        "activate must not reset FissionCounter; got {counter:?}"
    );
}

// ── Behavior 5 (edge case) — counter at 0 stays 0 after activate ────────────

#[test]
fn activate_leaves_zero_counter_unchanged() {
    let mut app = TestAppBuilder::new().build();
    install_fission_counter(&mut app, 0);

    activate_now(&mut app, &ProtocolTuning::Fission { kills_per_split: 8 });

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 0 },
        "zero counter must remain zero after activate; got {counter:?}"
    );
}
