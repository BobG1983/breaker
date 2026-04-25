//! Group A — `activate` lifecycle (Behaviors A1–A5).
//!
//! Pins that matching `ProtocolTuning::Afterimage` inserts `AfterimageConfig`
//! with both fields passed through verbatim, mismatched tuning is a no-op,
//! repeat activates are last-write-wins, mismatched activate after a matched
//! activate preserves the earlier config, and `AfterimageConfig` derives
//! `Resource + Debug + Clone + Copy + PartialEq`.

use super::{super::system::AfterimageConfig, helpers::activate_now};
use crate::{mutators::protocols::definition::ProtocolTuning, prelude::*};

// ── A1 — matching tuning inserts config with both fields ────────────────────

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Afterimage {
            phantom_duration:      2.0,
            phantom_bolt_duration: 3.0,
        },
    );

    let cfg = app
        .world()
        .get_resource::<AfterimageConfig>()
        .expect("AfterimageConfig should be inserted after matching activate");
    assert!(
        (cfg.phantom_duration - 2.0).abs() < f32::EPSILON,
        "phantom_duration verbatim expected 2.0, got {}",
        cfg.phantom_duration
    );
    assert!(
        (cfg.phantom_bolt_duration - 3.0).abs() < f32::EPSILON,
        "phantom_bolt_duration verbatim expected 3.0, got {}",
        cfg.phantom_bolt_duration
    );
}

// ── A1 (edge case) — non-trivial values pass through verbatim ──────────────

#[test]
fn activate_passes_non_trivial_values_through_verbatim() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Afterimage {
            phantom_duration:      0.5,
            phantom_bolt_duration: 0.125,
        },
    );

    let cfg = app
        .world()
        .get_resource::<AfterimageConfig>()
        .expect("AfterimageConfig should be inserted after matching activate");
    assert!(
        (cfg.phantom_duration - 0.5).abs() < f32::EPSILON,
        "phantom_duration verbatim expected 0.5, got {}",
        cfg.phantom_duration
    );
    assert!(
        (cfg.phantom_bolt_duration - 0.125).abs() < f32::EPSILON,
        "phantom_bolt_duration verbatim expected 0.125, got {}",
        cfg.phantom_bolt_duration
    );
}

// ── A2 — mismatched tuning does nothing ────────────────────────────────────

#[test]
fn activate_with_mismatched_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.05,
        },
    );

    assert!(
        app.world().get_resource::<AfterimageConfig>().is_none(),
        "mismatched tuning must not insert AfterimageConfig"
    );
}

// ── A2 (edge case) — prior config preserved across mismatched call ─────────

#[test]
fn mismatched_activate_preserves_existing_config() {
    let mut app = TestAppBuilder::new().build();
    app.world_mut().insert_resource(AfterimageConfig {
        phantom_duration:      2.0,
        phantom_bolt_duration: 3.0,
    });

    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    let cfg = app
        .world()
        .get_resource::<AfterimageConfig>()
        .expect("prior AfterimageConfig must remain after mismatched activate");
    assert_eq!(
        *cfg,
        AfterimageConfig {
            phantom_duration:      2.0,
            phantom_bolt_duration: 3.0,
        }
    );
}

// ── A3 — last-write-wins on matching re-activate ───────────────────────────

#[test]
fn second_activate_with_afterimage_tuning_overwrites_prior_config() {
    let mut app = TestAppBuilder::new().build();
    app.world_mut().insert_resource(AfterimageConfig {
        phantom_duration:      2.0,
        phantom_bolt_duration: 3.0,
    });

    activate_now(
        &mut app,
        &ProtocolTuning::Afterimage {
            phantom_duration:      1.0,
            phantom_bolt_duration: 0.5,
        },
    );

    let cfg = app
        .world()
        .get_resource::<AfterimageConfig>()
        .expect("AfterimageConfig should still be present after re-activate");
    assert_eq!(
        *cfg,
        AfterimageConfig {
            phantom_duration:      1.0,
            phantom_bolt_duration: 0.5,
        },
        "last-write-wins overwrite"
    );
}

// ── A3 (edge case) — third matching activate overwrites the second ─────────

#[test]
fn third_matching_activate_overwrites_the_second() {
    let mut app = TestAppBuilder::new().build();

    activate_now(
        &mut app,
        &ProtocolTuning::Afterimage {
            phantom_duration:      2.0,
            phantom_bolt_duration: 3.0,
        },
    );
    activate_now(
        &mut app,
        &ProtocolTuning::Afterimage {
            phantom_duration:      1.0,
            phantom_bolt_duration: 0.5,
        },
    );
    activate_now(
        &mut app,
        &ProtocolTuning::Afterimage {
            phantom_duration:      4.0,
            phantom_bolt_duration: 8.0,
        },
    );

    let cfg = app
        .world()
        .get_resource::<AfterimageConfig>()
        .expect("AfterimageConfig should still be present after third re-activate");
    assert_eq!(
        *cfg,
        AfterimageConfig {
            phantom_duration:      4.0,
            phantom_bolt_duration: 8.0,
        }
    );
}

// ── A4 — mismatched activate after matched activate preserves config ───────

#[test]
fn mismatched_activate_after_matched_activate_preserves_config() {
    let mut app = TestAppBuilder::new().build();

    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );
    assert!(app.world().get_resource::<AfterimageConfig>().is_none());

    activate_now(
        &mut app,
        &ProtocolTuning::Afterimage {
            phantom_duration:      2.0,
            phantom_bolt_duration: 3.0,
        },
    );

    activate_now(
        &mut app,
        &ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
    );

    let cfg = app
        .world()
        .get_resource::<AfterimageConfig>()
        .expect("AfterimageConfig must be preserved after second mismatched activate");
    assert_eq!(
        *cfg,
        AfterimageConfig {
            phantom_duration:      2.0,
            phantom_bolt_duration: 3.0,
        },
        "matched insert must survive the second mismatched activate"
    );
}

// ── A5 — AfterimageConfig is Resource + Debug + Clone + Copy + PartialEq ───

#[test]
fn afterimage_config_is_copy_clone_partial_eq() {
    fn takes_by_value(cfg: AfterimageConfig) -> f32 {
        cfg.phantom_duration
    }
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    let orig = AfterimageConfig {
        phantom_duration:      2.0,
        phantom_bolt_duration: 3.0,
    };

    // PartialEq: two identical values compare equal.
    let same = AfterimageConfig {
        phantom_duration:      2.0,
        phantom_bolt_duration: 3.0,
    };
    assert_eq!(orig, same, "identical configs must compare equal");

    // Copy: two pass-by-value assignments both succeed (original not moved).
    let copy1 = orig;
    let copy2 = orig;
    assert_eq!(orig, copy1, "copy must equal original");
    assert_eq!(copy1, copy2, "both copies must equal each other");

    // Copy: pass-by-value into a function compiles and original remains usable.
    let returned = takes_by_value(orig);
    assert!(
        (returned - 2.0).abs() < f32::EPSILON,
        "takes_by_value returned {returned}, expected 2.0"
    );
    assert!(
        (orig.phantom_duration - 2.0).abs() < f32::EPSILON,
        "original remains usable after pass-by-value (Copy, not move)"
    );

    // Clone: routed through a generic that requires `T: Clone`.
    let cloned = require_clone(&orig);
    assert_eq!(orig, cloned, "Clone-returned copy must equal original");

    // Debug: renders without panicking.
    assert!(
        format!("{orig:?}").contains("AfterimageConfig"),
        "Debug output must contain the type name"
    );
}

// ── A5 (edge case) — inequality on differing phantom_duration ──────────────

#[test]
fn afterimage_config_inequality_on_differing_fields() {
    let a = AfterimageConfig {
        phantom_duration:      2.0,
        phantom_bolt_duration: 3.0,
    };
    let b = AfterimageConfig {
        phantom_duration:      1.0,
        phantom_bolt_duration: 3.0,
    };
    assert_ne!(
        a, b,
        "configs that differ in phantom_duration must compare !="
    );
}
