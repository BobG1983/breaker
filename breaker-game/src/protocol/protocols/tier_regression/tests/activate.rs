//! Group A — `activate` lifecycle (Behaviors 1–5).
//!
//! Pins that matching `ProtocolTuning::TierRegression` inserts
//! `TierRegressionConfig` with `tiers_back` verbatim AND inserts the
//! `TierRegressionPending` marker; that mismatched tuning is a no-op for
//! both resources; that repeat matching activates are last-write-wins on
//! config and re-insert the pending marker; and that the activation path
//! does not clamp large `tiers_back` values.

use super::{
    super::system::{TierRegressionConfig, TierRegressionPending},
    helpers::{activate_now, install_config, install_pending},
};
use crate::{prelude::*, protocol::definition::ProtocolTuning};

// ── 1 — matching tuning inserts config and pending marker ──────────────────-

#[test]
fn activate_with_matching_tuning_inserts_config_and_marker() {
    let mut app = TestAppBuilder::new().build();
    activate_now(&mut app, &ProtocolTuning::TierRegression { tiers_back: 2 });

    let cfg = app
        .world()
        .get_resource::<TierRegressionConfig>()
        .expect("TierRegressionConfig should be inserted after matching activate");
    assert_eq!(
        *cfg,
        TierRegressionConfig { tiers_back: 2 },
        "tiers_back verbatim"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_some(),
        "TierRegressionPending marker must be inserted"
    );
}

// ── 1 edge — tiers_back: 0 still inserts both resources ────────────────────-

#[test]
fn activate_with_tiers_back_zero_inserts_config_and_marker() {
    let mut app = TestAppBuilder::new().build();
    activate_now(&mut app, &ProtocolTuning::TierRegression { tiers_back: 0 });

    let cfg = app
        .world()
        .get_resource::<TierRegressionConfig>()
        .expect("TierRegressionConfig should be inserted even for tiers_back: 0");
    assert_eq!(*cfg, TierRegressionConfig { tiers_back: 0 });
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_some(),
        "TierRegressionPending marker must be inserted for tiers_back: 0 too"
    );
}

// ── 2 — mismatched tuning inserts nothing ──────────────────────────────────-

#[test]
fn activate_with_greed_tuning_inserts_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.05,
        },
    );

    assert!(
        app.world().get_resource::<TierRegressionConfig>().is_none(),
        "mismatched tuning must not insert TierRegressionConfig"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "mismatched tuning must not insert TierRegressionPending"
    );
}

// ── 2 edge — Siphon tuning also a no-op ────────────────────────────────────-

#[test]
fn activate_with_siphon_tuning_inserts_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    assert!(
        app.world().get_resource::<TierRegressionConfig>().is_none(),
        "Siphon tuning must not bind to TierRegression's activate branch"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "Siphon tuning must not insert TierRegressionPending"
    );
}

// ── 3 — second matching activate overwrites config and re-inserts pending ──-

#[test]
fn second_matching_activate_overwrites_config_and_reinserts_pending() {
    let mut app = TestAppBuilder::new().build();
    install_config(&mut app, 1);
    install_pending(&mut app);

    activate_now(&mut app, &ProtocolTuning::TierRegression { tiers_back: 3 });

    let cfg = app
        .world()
        .get_resource::<TierRegressionConfig>()
        .expect("TierRegressionConfig must still be present after re-activate");
    assert_eq!(
        *cfg,
        TierRegressionConfig { tiers_back: 3 },
        "second activate is last-write-wins on config"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_some(),
        "pending marker must still be present after re-activate"
    );
}

// ── 3 edge — activate reinstates pending that was previously consumed ──────-

#[test]
fn matching_activate_reinstates_previously_consumed_pending() {
    let mut app = TestAppBuilder::new().build();
    install_config(&mut app, 1);
    // Simulate a previous `apply_tier_regression` having removed the marker:
    // we simply never install it here.
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "precondition: pending marker absent"
    );

    activate_now(&mut app, &ProtocolTuning::TierRegression { tiers_back: 2 });

    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_some(),
        "matching activate must reinstate the pending marker"
    );
    assert_eq!(
        *app.world()
            .get_resource::<TierRegressionConfig>()
            .expect("config must also be present"),
        TierRegressionConfig { tiers_back: 2 },
    );
}

// ── 4 — mismatched activate after matching activate preserves both ─────────-

#[test]
fn mismatched_activate_after_matching_activate_preserves_config_and_pending() {
    let mut app = TestAppBuilder::new().build();
    activate_now(&mut app, &ProtocolTuning::TierRegression { tiers_back: 2 });

    activate_now(
        &mut app,
        &ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.05,
        },
    );

    assert_eq!(
        *app.world()
            .get_resource::<TierRegressionConfig>()
            .expect("config must survive mismatched activate"),
        TierRegressionConfig { tiers_back: 2 },
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_some(),
        "pending marker must survive mismatched activate"
    );
}

// ── 5 — tiers_back: u32::MAX inserts verbatim (no clamping at activation) ──-

#[test]
fn activate_with_max_tiers_back_inserts_verbatim() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::TierRegression {
            tiers_back: u32::MAX,
        },
    );

    let cfg = app
        .world()
        .get_resource::<TierRegressionConfig>()
        .expect("TierRegressionConfig should be inserted for u32::MAX");
    assert_eq!(
        *cfg,
        TierRegressionConfig {
            tiers_back: u32::MAX,
        },
        "activate must not clamp tiers_back — clamping happens in apply_tier_regression"
    );
}
