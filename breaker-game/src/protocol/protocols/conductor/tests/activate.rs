//! Group A — `activate` lifecycle (Behaviors 20–23).
//!
//! Pins that matching `ProtocolTuning::Conductor` inserts `ConductorConfig`;
//! mismatched tuning is a no-op; repeat activates are last-write-wins; and
//! mismatched activate after a matched activate preserves the earlier config.

use super::{
    super::system::ConductorConfig,
    helpers::{activate_now, canonical_conductor_config, install_conductor_config},
};
use crate::{prelude::*, protocol::definition::ProtocolTuning};

// ── Behavior 20 — matching tuning inserts the presence marker ───────────────

#[test]
fn activate_with_matching_tuning_inserts_conductor_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(&mut app, &ProtocolTuning::Conductor);

    let cfg = app
        .world()
        .get_resource::<ConductorConfig>()
        .expect("ConductorConfig should be inserted after matching activate");
    assert_eq!(*cfg, ConductorConfig);
}

// ── Behavior 21 — mismatched tuning does nothing ────────────────────────────

#[test]
fn activate_with_mismatched_tuning_does_not_insert_conductor_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.05,
        },
    );

    assert!(
        app.world().get_resource::<ConductorConfig>().is_none(),
        "mismatched tuning must not insert ConductorConfig"
    );
}

// ── Behavior 21 (edge case) — prior config preserved across mismatched call

#[test]
fn mismatched_activate_preserves_existing_conductor_config() {
    let mut app = TestAppBuilder::new().build();
    install_conductor_config(&mut app, canonical_conductor_config());

    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    let cfg = app
        .world()
        .get_resource::<ConductorConfig>()
        .expect("prior ConductorConfig must remain after mismatched activate");
    assert_eq!(*cfg, canonical_conductor_config());
}

// ── Behavior 22 — last-write-wins on matching re-activate ───────────────────

#[test]
fn second_activate_with_conductor_tuning_overwrites_prior_config() {
    let mut app = TestAppBuilder::new().build();
    install_conductor_config(&mut app, canonical_conductor_config());

    activate_now(&mut app, &ProtocolTuning::Conductor);

    let cfg = app
        .world()
        .get_resource::<ConductorConfig>()
        .expect("ConductorConfig should still be present after re-activate");
    assert_eq!(*cfg, ConductorConfig);
}

// ── Behavior 23 — mismatched activate after matched activate preserves config

#[test]
fn mismatched_activate_after_matched_activate_preserves_conductor_config() {
    let mut app = TestAppBuilder::new().build();

    // First mismatched call — no insert.
    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );
    assert!(app.world().get_resource::<ConductorConfig>().is_none());

    // Matched call — insert canonical.
    activate_now(&mut app, &ProtocolTuning::Conductor);

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
        .get_resource::<ConductorConfig>()
        .expect("prior ConductorConfig must be preserved after second mismatched activate");
    assert_eq!(
        *cfg,
        canonical_conductor_config(),
        "canonical must survive mismatched activate"
    );
}
