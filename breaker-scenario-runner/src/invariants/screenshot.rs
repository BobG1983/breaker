//! First-failure screenshot tracking: detects new invariant violations,
//! formats screenshot output paths, and tracks which kinds have been captured.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};

use super::ViolationLog;
use crate::types::InvariantKind;

/// Tracks which [`InvariantKind`] values have already had a screenshot taken
/// during this scenario run, ensuring at most one screenshot per kind.
#[derive(Resource, Default)]
pub struct ScreenshotTracker {
    /// The set of invariant kinds that have already been captured.
    pub captured: HashSet<InvariantKind>,
}

impl ScreenshotTracker {
    /// Records that a screenshot has been taken for the given invariant kind.
    pub fn mark_captured(&mut self, kind: InvariantKind) {
        self.captured.insert(kind);
    }
}

/// Holds the scenario name (RON file stem), inserted by `run_scenario`.
#[derive(Resource)]
pub struct ScenarioName(pub String);

/// Holds the output directory path, inserted conditionally when running in visual mode.
#[derive(Resource)]
pub struct ScreenshotOutputDir(pub PathBuf);

/// Returns the set of [`InvariantKind`] values present in the [`ViolationLog`]
/// but not yet in `tracker.captured`. Pure function, no side effects.
#[must_use]
pub fn detect_new_violations(
    tracker: &ScreenshotTracker,
    log: &ViolationLog,
) -> HashSet<InvariantKind> {
    log.0
        .iter()
        .map(|entry| entry.invariant)
        .filter(|kind| !tracker.captured.contains(kind))
        .collect()
}

/// Constructs `<output_dir>/<scenario_name>-<kind:?>.png`.
#[must_use]
pub fn screenshot_path(output_dir: &Path, scenario_name: &str, kind: InvariantKind) -> PathBuf {
    output_dir.join(format!("{scenario_name}-{kind:?}.png"))
}

/// Detects new invariant violations and captures screenshots for each.
///
/// Early-returns when `ScreenshotOutputDir` or `ScenarioName` resources are
/// absent (headless mode). Otherwise, for each new violation kind not yet in
/// the tracker, creates the output directory, spawns a `Screenshot` entity
/// with a `save_to_disk` observer, and marks the kind as captured.
pub fn capture_violation_screenshots(
    mut commands: Commands,
    mut tracker: ResMut<ScreenshotTracker>,
    vl: Res<ViolationLog>,
    output_dir: Option<Res<ScreenshotOutputDir>>,
    scenario_name: Option<Res<ScenarioName>>,
) {
    let (Some(output_dir), Some(scenario_name)) = (output_dir, scenario_name) else {
        return;
    };

    let new_kinds = detect_new_violations(&tracker, &vl);
    if new_kinds.is_empty() {
        return;
    }

    for kind in &new_kinds {
        let path = screenshot_path(&output_dir.0, &scenario_name.0, *kind);

        drop(std::fs::create_dir_all(
            path.parent().unwrap_or_else(|| Path::new(".")),
        ));

        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));

        tracker.mark_captured(*kind);

        println!("Screenshot requested: {}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::{super::ViolationEntry, *};

    // =========================================================================
    // ScreenshotTracker — construction and mark_captured
    // =========================================================================

    /// Default-constructed [`ScreenshotTracker`] has an empty `captured` set.
    #[test]
    fn tracker_default_has_empty_captured_set() {
        let tracker = ScreenshotTracker::default();
        assert!(
            tracker.captured.is_empty(),
            "default tracker must have empty captured set"
        );
        assert_eq!(
            tracker.captured.len(),
            0,
            "default tracker captured.len() must be 0"
        );
    }

    /// `mark_captured` inserts a single `InvariantKind` into the tracker.
    #[test]
    fn mark_captured_inserts_single_kind() {
        let mut tracker = ScreenshotTracker::default();
        tracker.mark_captured(InvariantKind::NoNaN);
        assert!(
            tracker.captured.contains(&InvariantKind::NoNaN),
            "tracker must contain NoNaN after mark_captured"
        );
        assert_eq!(
            tracker.captured.len(),
            1,
            "tracker must contain exactly 1 kind after single mark_captured"
        );
    }

    /// Calling `mark_captured` with the same kind twice does not duplicate it
    /// (`HashSet` dedup).
    #[test]
    fn mark_captured_same_kind_twice_deduplicates() {
        let mut tracker = ScreenshotTracker::default();
        tracker.mark_captured(InvariantKind::NoNaN);
        tracker.mark_captured(InvariantKind::NoNaN);
        assert_eq!(
            tracker.captured.len(),
            1,
            "marking the same kind twice must not increase set size beyond 1"
        );
    }

    /// `mark_captured` with multiple distinct kinds accumulates all of them.
    #[test]
    fn mark_captured_accumulates_multiple_distinct_kinds() {
        let mut tracker = ScreenshotTracker::default();
        tracker.mark_captured(InvariantKind::BoltInBounds);
        tracker.mark_captured(InvariantKind::NoNaN);
        tracker.mark_captured(InvariantKind::TimerNonNegative);
        assert_eq!(
            tracker.captured.len(),
            3,
            "tracker must contain exactly 3 kinds after 3 distinct mark_captured calls"
        );
        assert!(
            tracker.captured.contains(&InvariantKind::BoltInBounds),
            "tracker must contain BoltInBounds"
        );
        assert!(
            tracker.captured.contains(&InvariantKind::NoNaN),
            "tracker must contain NoNaN"
        );
        assert!(
            tracker.captured.contains(&InvariantKind::TimerNonNegative),
            "tracker must contain TimerNonNegative"
        );
    }

    // =========================================================================
    // detect_new_violations
    // =========================================================================

    /// `detect_new_violations` returns new `InvariantKind`s from `ViolationLog`
    /// that are not yet in the tracker. Duplicate log entries for the same kind
    /// are deduplicated in the result.
    #[test]
    fn detect_new_violations_returns_unseen_kinds_deduplicating_log_entries() {
        let mut tracker = ScreenshotTracker::default();
        tracker.captured.insert(InvariantKind::BoltInBounds);

        let log = ViolationLog(vec![
            ViolationEntry {
                frame: 10,
                invariant: InvariantKind::BoltInBounds,
                entity: None,
                message: "oob".into(),
            },
            ViolationEntry {
                frame: 12,
                invariant: InvariantKind::NoNaN,
                entity: None,
                message: "nan x".into(),
            },
            ViolationEntry {
                frame: 14,
                invariant: InvariantKind::NoNaN,
                entity: None,
                message: "nan y".into(),
            },
        ]);

        let result = detect_new_violations(&tracker, &log);

        assert_eq!(
            result.len(),
            1,
            "must return exactly 1 new kind (NoNaN), not 2 for duplicate log entries"
        );
        assert!(
            result.contains(&InvariantKind::NoNaN),
            "result must contain NoNaN"
        );
        assert!(
            !result.contains(&InvariantKind::BoltInBounds),
            "result must not contain already-captured BoltInBounds"
        );
    }

    /// `detect_new_violations` returns empty set when all violation kinds are
    /// already captured.
    #[test]
    fn detect_new_violations_returns_empty_when_all_kinds_captured() {
        let mut tracker = ScreenshotTracker::default();
        tracker.captured.insert(InvariantKind::BoltInBounds);
        tracker.captured.insert(InvariantKind::NoNaN);

        let log = ViolationLog(vec![
            ViolationEntry {
                frame: 5,
                invariant: InvariantKind::BoltInBounds,
                entity: None,
                message: "oob".into(),
            },
            ViolationEntry {
                frame: 7,
                invariant: InvariantKind::NoNaN,
                entity: None,
                message: "nan".into(),
            },
        ]);

        let result = detect_new_violations(&tracker, &log);

        assert!(
            result.is_empty(),
            "must return empty set when all violation kinds already captured, got {} kinds",
            result.len()
        );
    }

    /// Edge case: single-kind exact match between tracker and log returns empty.
    #[test]
    fn detect_new_violations_returns_empty_for_single_kind_exact_match() {
        let mut tracker = ScreenshotTracker::default();
        tracker.captured.insert(InvariantKind::NoNaN);

        let log = ViolationLog(vec![ViolationEntry {
            frame: 1,
            invariant: InvariantKind::NoNaN,
            entity: None,
            message: "nan".into(),
        }]);

        let result = detect_new_violations(&tracker, &log);

        assert!(
            result.is_empty(),
            "must return empty set for single-kind exact match"
        );
    }

    /// `detect_new_violations` returns empty set for empty `ViolationLog`,
    /// even when tracker is also empty.
    #[test]
    fn detect_new_violations_returns_empty_for_empty_log() {
        let tracker = ScreenshotTracker::default();
        let log = ViolationLog(vec![]);

        let result = detect_new_violations(&tracker, &log);

        assert!(
            result.is_empty(),
            "must return empty set for empty ViolationLog"
        );
    }

    /// `detect_new_violations` returns multiple new kinds when multiple unseen
    /// violations exist in the log and the tracker is empty.
    #[test]
    fn detect_new_violations_returns_multiple_new_kinds_from_empty_tracker() {
        let tracker = ScreenshotTracker::default();

        let log = ViolationLog(vec![
            ViolationEntry {
                frame: 1,
                invariant: InvariantKind::BoltInBounds,
                entity: None,
                message: "oob".into(),
            },
            ViolationEntry {
                frame: 2,
                invariant: InvariantKind::NoNaN,
                entity: None,
                message: "nan".into(),
            },
            ViolationEntry {
                frame: 3,
                invariant: InvariantKind::TimerNonNegative,
                entity: None,
                message: "neg".into(),
            },
        ]);

        let result = detect_new_violations(&tracker, &log);

        assert_eq!(
            result.len(),
            3,
            "must return all 3 new kinds when tracker is empty"
        );
        assert!(result.contains(&InvariantKind::BoltInBounds));
        assert!(result.contains(&InvariantKind::NoNaN));
        assert!(result.contains(&InvariantKind::TimerNonNegative));
    }

    /// `detect_new_violations` correctly filters a mix of seen and unseen kinds.
    #[test]
    fn detect_new_violations_filters_mix_of_seen_and_unseen_kinds() {
        let mut tracker = ScreenshotTracker::default();
        tracker.captured.insert(InvariantKind::BoltInBounds);
        tracker.captured.insert(InvariantKind::TimerNonNegative);

        let log = ViolationLog(vec![
            ViolationEntry {
                frame: 1,
                invariant: InvariantKind::BoltInBounds,
                entity: None,
                message: "a".into(),
            },
            ViolationEntry {
                frame: 2,
                invariant: InvariantKind::NoNaN,
                entity: None,
                message: "b".into(),
            },
            ViolationEntry {
                frame: 3,
                invariant: InvariantKind::TimerNonNegative,
                entity: None,
                message: "c".into(),
            },
            ViolationEntry {
                frame: 4,
                invariant: InvariantKind::BreakerInBounds,
                entity: None,
                message: "d".into(),
            },
        ]);

        let result = detect_new_violations(&tracker, &log);

        assert_eq!(
            result.len(),
            2,
            "must return exactly 2 unseen kinds (NoNaN, BreakerInBounds)"
        );
        assert!(
            result.contains(&InvariantKind::NoNaN),
            "result must contain NoNaN (unseen)"
        );
        assert!(
            result.contains(&InvariantKind::BreakerInBounds),
            "result must contain BreakerInBounds (unseen)"
        );
        assert!(
            !result.contains(&InvariantKind::BoltInBounds),
            "result must not contain BoltInBounds (already captured)"
        );
        assert!(
            !result.contains(&InvariantKind::TimerNonNegative),
            "result must not contain TimerNonNegative (already captured)"
        );
    }

    // =========================================================================
    // screenshot_path
    // =========================================================================

    /// `screenshot_path` formats path as `<output_dir>/<scenario_name>-<Kind:?>.png`.
    #[test]
    fn screenshot_path_formats_with_output_dir_scenario_name_and_kind_debug() {
        let result = screenshot_path(
            Path::new("/tmp/breaker-scenario-runner/2026-04-06/0"),
            "aegis_chaos",
            InvariantKind::BoltInBounds,
        );
        assert_eq!(
            result,
            PathBuf::from("/tmp/breaker-scenario-runner/2026-04-06/0/aegis_chaos-BoltInBounds.png"),
            "path must be <output_dir>/<scenario_name>-<Kind:?>.png"
        );
    }

    /// `screenshot_path` uses the Debug format of `InvariantKind` for the
    /// filename, including long variant names without truncation.
    #[test]
    fn screenshot_path_uses_debug_format_for_long_variant_name() {
        let result = screenshot_path(
            Path::new("/tmp/out/2026-04-06/1"),
            "bolt_speed_stress",
            InvariantKind::TimerMonotonicallyDecreasing,
        );
        assert_eq!(
            result,
            PathBuf::from(
                "/tmp/out/2026-04-06/1/bolt_speed_stress-TimerMonotonicallyDecreasing.png"
            ),
            "long variant name must not be truncated or mangled"
        );
    }

    /// `screenshot_path` preserves scenario names with underscores exactly.
    #[test]
    fn screenshot_path_preserves_underscored_scenario_name() {
        let result = screenshot_path(
            Path::new("/tmp/breaker-scenario-runner/2026-04-06/0"),
            "self_test_bolt_in_bounds",
            InvariantKind::NoNaN,
        );
        assert_eq!(
            result,
            PathBuf::from(
                "/tmp/breaker-scenario-runner/2026-04-06/0/self_test_bolt_in_bounds-NoNaN.png"
            ),
            "scenario name with underscores must be preserved exactly"
        );
    }

    /// `screenshot_path` handles minimal (short) path components correctly.
    #[test]
    fn screenshot_path_handles_minimal_output_dir() {
        let result = screenshot_path(Path::new("/out"), "test", InvariantKind::NoEntityLeaks);
        assert_eq!(
            result,
            PathBuf::from("/out/test-NoEntityLeaks.png"),
            "very short path components must still produce valid output"
        );
    }

    // =========================================================================
    // capture_violation_screenshots system
    // =========================================================================

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    fn tick(app: &mut App) {
        app.update();
    }

    /// System marks tracker when a new violation is detected.
    #[test]
    fn capture_violation_screenshots_marks_tracker_on_new_violation() {
        let mut app = test_app();
        app.insert_resource(ScreenshotTracker::default())
            .insert_resource(ViolationLog(vec![ViolationEntry {
                frame: 5,
                invariant: InvariantKind::BoltInBounds,
                entity: None,
                message: "oob".into(),
            }]))
            .insert_resource(ScreenshotOutputDir(PathBuf::from(
                "/tmp/test-output/2026-04-07/0",
            )))
            .insert_resource(ScenarioName("aegis_chaos".into()))
            .add_systems(Last, capture_violation_screenshots);

        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert!(
            tracker.captured.contains(&InvariantKind::BoltInBounds),
            "tracker must contain BoltInBounds after system runs with new violation"
        );
        assert_eq!(
            tracker.captured.len(),
            1,
            "tracker must contain exactly 1 kind after a single new violation"
        );
    }

    /// System does not update tracker when no new violations exist (all already captured).
    #[test]
    fn capture_violation_screenshots_no_update_when_no_new_violations() {
        let mut tracker = ScreenshotTracker::default();
        tracker.captured.insert(InvariantKind::BoltInBounds);

        let mut app = test_app();
        app.insert_resource(tracker)
            .insert_resource(ViolationLog(vec![ViolationEntry {
                frame: 5,
                invariant: InvariantKind::BoltInBounds,
                entity: None,
                message: "oob".into(),
            }]))
            .insert_resource(ScreenshotOutputDir(PathBuf::from(
                "/tmp/test-output/2026-04-07/0",
            )))
            .insert_resource(ScenarioName("aegis_chaos".into()))
            .add_systems(Last, capture_violation_screenshots);

        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert_eq!(
            tracker.captured.len(),
            1,
            "tracker must remain at 1 kind when violation already captured"
        );
        assert!(
            tracker.captured.contains(&InvariantKind::BoltInBounds),
            "tracker must still contain BoltInBounds"
        );
    }

    /// System does not update tracker when `ViolationLog` is empty.
    #[test]
    fn capture_violation_screenshots_no_update_on_empty_violation_log() {
        let mut app = test_app();
        app.insert_resource(ScreenshotTracker::default())
            .insert_resource(ViolationLog(vec![]))
            .insert_resource(ScreenshotOutputDir(PathBuf::from(
                "/tmp/test-output/2026-04-07/0",
            )))
            .insert_resource(ScenarioName("test_scenario".into()))
            .add_systems(Last, capture_violation_screenshots);

        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert!(
            tracker.captured.is_empty(),
            "tracker must remain empty when ViolationLog has no entries"
        );
    }

    /// System marks all new violation kinds as captured in one tick.
    #[test]
    fn capture_violation_screenshots_marks_all_new_kinds_in_one_tick() {
        let mut app = test_app();
        app.insert_resource(ScreenshotTracker::default())
            .insert_resource(ViolationLog(vec![
                ViolationEntry {
                    frame: 10,
                    invariant: InvariantKind::BoltInBounds,
                    entity: None,
                    message: "oob".into(),
                },
                ViolationEntry {
                    frame: 10,
                    invariant: InvariantKind::NoNaN,
                    entity: None,
                    message: "nan".into(),
                },
                ViolationEntry {
                    frame: 10,
                    invariant: InvariantKind::TimerNonNegative,
                    entity: None,
                    message: "neg".into(),
                },
                // Duplicate BoltInBounds entry — should not affect count
                ViolationEntry {
                    frame: 10,
                    invariant: InvariantKind::BoltInBounds,
                    entity: None,
                    message: "oob again".into(),
                },
            ]))
            .insert_resource(ScreenshotOutputDir(PathBuf::from(
                "/tmp/test-output/2026-04-07/0",
            )))
            .insert_resource(ScenarioName("multi_fail".into()))
            .add_systems(Last, capture_violation_screenshots);

        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert_eq!(
            tracker.captured.len(),
            3,
            "tracker must contain exactly 3 distinct kinds, got {}",
            tracker.captured.len()
        );
        assert!(tracker.captured.contains(&InvariantKind::BoltInBounds));
        assert!(tracker.captured.contains(&InvariantKind::NoNaN));
        assert!(tracker.captured.contains(&InvariantKind::TimerNonNegative));
    }

    /// System filters mix of already-captured and new violation kinds.
    #[test]
    fn capture_violation_screenshots_filters_already_captured_and_adds_new() {
        let mut tracker = ScreenshotTracker::default();
        tracker.captured.insert(InvariantKind::BoltInBounds);
        tracker.captured.insert(InvariantKind::NoNaN);

        let mut app = test_app();
        app.insert_resource(tracker)
            .insert_resource(ViolationLog(vec![
                ViolationEntry {
                    frame: 1,
                    invariant: InvariantKind::BoltInBounds,
                    entity: None,
                    message: "a".into(),
                },
                ViolationEntry {
                    frame: 2,
                    invariant: InvariantKind::NoNaN,
                    entity: None,
                    message: "b".into(),
                },
                ViolationEntry {
                    frame: 3,
                    invariant: InvariantKind::TimerNonNegative,
                    entity: None,
                    message: "c".into(),
                },
                ViolationEntry {
                    frame: 4,
                    invariant: InvariantKind::BreakerInBounds,
                    entity: None,
                    message: "d".into(),
                },
            ]))
            .insert_resource(ScreenshotOutputDir(PathBuf::from(
                "/tmp/test-output/2026-04-07/0",
            )))
            .insert_resource(ScenarioName("mixed_scenario".into()))
            .add_systems(Last, capture_violation_screenshots);

        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert_eq!(
            tracker.captured.len(),
            4,
            "tracker must contain 4 kinds (2 pre-existing + 2 new), got {}",
            tracker.captured.len()
        );
        assert!(tracker.captured.contains(&InvariantKind::BoltInBounds));
        assert!(tracker.captured.contains(&InvariantKind::NoNaN));
        assert!(tracker.captured.contains(&InvariantKind::TimerNonNegative));
        assert!(tracker.captured.contains(&InvariantKind::BreakerInBounds));
    }

    /// System early-returns when `ScreenshotOutputDir` resource is absent (headless mode).
    /// Positive case first (both resources present -> tracker updated), then negative
    /// case (`ScreenshotOutputDir` absent -> tracker empty).
    #[test]
    fn capture_violation_screenshots_early_returns_without_output_dir() {
        // Positive case: both resources present -> tracker is updated
        let mut positive_app = test_app();
        positive_app
            .insert_resource(ScreenshotTracker::default())
            .insert_resource(ViolationLog(vec![ViolationEntry {
                frame: 1,
                invariant: InvariantKind::NoNaN,
                entity: None,
                message: "nan".into(),
            }]))
            .insert_resource(ScreenshotOutputDir(PathBuf::from(
                "/tmp/test-output/2026-04-07/0",
            )))
            .insert_resource(ScenarioName("headless_test".into()))
            .add_systems(Last, capture_violation_screenshots);

        tick(&mut positive_app);

        let tracker = positive_app.world().resource::<ScreenshotTracker>();
        assert!(
            !tracker.captured.is_empty(),
            "positive case: tracker must NOT be empty when both resources are present"
        );

        // Negative case: ScreenshotOutputDir absent -> tracker stays empty
        let mut negative_app = test_app();
        negative_app
            .insert_resource(ScreenshotTracker::default())
            .insert_resource(ViolationLog(vec![ViolationEntry {
                frame: 1,
                invariant: InvariantKind::NoNaN,
                entity: None,
                message: "nan".into(),
            }]))
            .insert_resource(ScenarioName("headless_test".into()))
            // No ScreenshotOutputDir inserted
            .add_systems(Last, capture_violation_screenshots);

        tick(&mut negative_app);

        let tracker = negative_app.world().resource::<ScreenshotTracker>();
        assert!(
            tracker.captured.is_empty(),
            "negative case: tracker must remain empty when ScreenshotOutputDir is absent"
        );
    }

    /// System early-returns when `ScenarioName` resource is absent.
    /// Positive case first (both resources present -> tracker updated), then negative
    /// case (`ScenarioName` absent -> tracker empty).
    #[test]
    fn capture_violation_screenshots_early_returns_without_scenario_name() {
        // Positive case: both resources present -> tracker is updated
        let mut positive_app = test_app();
        positive_app
            .insert_resource(ScreenshotTracker::default())
            .insert_resource(ViolationLog(vec![ViolationEntry {
                frame: 1,
                invariant: InvariantKind::NoNaN,
                entity: None,
                message: "nan".into(),
            }]))
            .insert_resource(ScreenshotOutputDir(PathBuf::from(
                "/tmp/test-output/2026-04-07/0",
            )))
            .insert_resource(ScenarioName("scenario_name_test".into()))
            .add_systems(Last, capture_violation_screenshots);

        tick(&mut positive_app);

        let tracker = positive_app.world().resource::<ScreenshotTracker>();
        assert!(
            !tracker.captured.is_empty(),
            "positive case: tracker must NOT be empty when both resources are present"
        );

        // Negative case: ScenarioName absent -> tracker stays empty
        let mut negative_app = test_app();
        negative_app
            .insert_resource(ScreenshotTracker::default())
            .insert_resource(ViolationLog(vec![ViolationEntry {
                frame: 1,
                invariant: InvariantKind::NoNaN,
                entity: None,
                message: "nan".into(),
            }]))
            .insert_resource(ScreenshotOutputDir(PathBuf::from(
                "/tmp/test-output/2026-04-07/0",
            )))
            // No ScenarioName inserted
            .add_systems(Last, capture_violation_screenshots);

        tick(&mut negative_app);

        let tracker = negative_app.world().resource::<ScreenshotTracker>();
        assert!(
            tracker.captured.is_empty(),
            "negative case: tracker must remain empty when ScenarioName is absent"
        );
    }

    /// System creates the output directory when it does not yet exist.
    #[test]
    fn capture_violation_screenshots_creates_output_directory() {
        let temp_base =
            std::env::temp_dir().join(format!("screenshot_test_{}_output", std::process::id()));
        // Ensure the directory does NOT exist before the test
        drop(std::fs::remove_dir_all(&temp_base));
        assert!(
            !temp_base.exists(),
            "temp directory must not exist before test"
        );

        let mut app = test_app();
        app.insert_resource(ScreenshotTracker::default())
            .insert_resource(ViolationLog(vec![ViolationEntry {
                frame: 1,
                invariant: InvariantKind::BoltInBounds,
                entity: None,
                message: "oob".into(),
            }]))
            .insert_resource(ScreenshotOutputDir(temp_base.clone()))
            .insert_resource(ScenarioName("dir_test".into()))
            .add_systems(Last, capture_violation_screenshots);

        tick(&mut app);

        // The system should have created the output directory
        assert!(
            temp_base.exists(),
            "output directory must exist after system runs"
        );

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert!(
            tracker.captured.contains(&InvariantKind::BoltInBounds),
            "tracker must contain BoltInBounds after directory creation"
        );

        // Cleanup
        drop(std::fs::remove_dir_all(&temp_base));
    }

    /// System is idempotent — running twice with the same `ViolationLog` does not change tracker.
    #[test]
    fn capture_violation_screenshots_idempotent_across_frames() {
        let mut app = test_app();
        app.insert_resource(ScreenshotTracker::default())
            .insert_resource(ViolationLog(vec![ViolationEntry {
                frame: 1,
                invariant: InvariantKind::NoNaN,
                entity: None,
                message: "nan".into(),
            }]))
            .insert_resource(ScreenshotOutputDir(PathBuf::from(
                "/tmp/test-output/2026-04-07/0",
            )))
            .insert_resource(ScenarioName("idem_test".into()))
            .add_systems(Last, capture_violation_screenshots);

        // First tick
        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert_eq!(
            tracker.captured.len(),
            1,
            "tracker must contain 1 kind after first tick"
        );

        // Second tick with same ViolationLog
        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert_eq!(
            tracker.captured.len(),
            1,
            "tracker must still contain exactly 1 kind after second tick (idempotent)"
        );
    }

    /// Idempotency edge case: `ViolationLog` grows between ticks with a new kind.
    #[test]
    fn capture_violation_screenshots_picks_up_new_kind_added_between_ticks() {
        let mut app = test_app();
        app.insert_resource(ScreenshotTracker::default())
            .insert_resource(ViolationLog(vec![ViolationEntry {
                frame: 1,
                invariant: InvariantKind::NoNaN,
                entity: None,
                message: "nan".into(),
            }]))
            .insert_resource(ScreenshotOutputDir(PathBuf::from(
                "/tmp/test-output/2026-04-07/0",
            )))
            .insert_resource(ScenarioName("idem_test".into()))
            .add_systems(Last, capture_violation_screenshots);

        // First tick
        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert_eq!(
            tracker.captured.len(),
            1,
            "tracker must contain 1 kind after first tick"
        );

        // Add a new violation kind to the log before second tick
        app.world_mut()
            .resource_mut::<ViolationLog>()
            .0
            .push(ViolationEntry {
                frame: 2,
                invariant: InvariantKind::BreakerInBounds,
                entity: None,
                message: "breaker oob".into(),
            });

        // Second tick
        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert_eq!(
            tracker.captured.len(),
            2,
            "tracker must contain 2 kinds after second tick with new violation added"
        );
        assert!(tracker.captured.contains(&InvariantKind::NoNaN));
        assert!(tracker.captured.contains(&InvariantKind::BreakerInBounds));
    }

    // =========================================================================
    // Pure function coverage extensions
    // =========================================================================

    /// `detect_new_violations` returns all `InvariantKind::ALL` variants when
    /// tracker is empty and log has one entry per variant.
    #[test]
    fn detect_new_violations_returns_all_variants_from_empty_tracker() {
        let tracker = ScreenshotTracker::default();
        let log = ViolationLog(
            InvariantKind::ALL
                .iter()
                .enumerate()
                .map(|(i, &kind)| ViolationEntry {
                    frame: u32::try_from(i).expect("frame index fits u32"),
                    invariant: kind,
                    entity: None,
                    message: format!("{kind:?}"),
                })
                .collect(),
        );

        let result = detect_new_violations(&tracker, &log);

        assert_eq!(
            result.len(),
            InvariantKind::ALL.len(),
            "must return all {} variants when tracker is empty, got {}",
            InvariantKind::ALL.len(),
            result.len()
        );
        for &kind in InvariantKind::ALL {
            assert!(result.contains(&kind), "result must contain {kind:?}");
        }
    }

    /// `screenshot_path` uses flat format with hyphen separator for long variant names.
    #[test]
    fn screenshot_path_flat_format_with_long_variant_name() {
        let result = screenshot_path(
            Path::new("/tmp/out/2026-04-07/0"),
            "self_test_bolt_in_bounds",
            InvariantKind::AabbMatchesEntityDimensions,
        );
        assert_eq!(
            result,
            PathBuf::from(
                "/tmp/out/2026-04-07/0/self_test_bolt_in_bounds-AabbMatchesEntityDimensions.png"
            ),
            "flat format must use hyphen separator with long variant name"
        );
    }

    // =========================================================================
    // Additional edge cases from reviewer feedback
    // =========================================================================

    /// Edge case for behavior 2: `ViolationLog` contains 5 duplicate `BoltInBounds`
    /// entries, tracker already has `BoltInBounds` — tracker still `len()` == 1.
    #[test]
    fn capture_violation_screenshots_no_update_with_five_duplicate_entries() {
        let mut tracker = ScreenshotTracker::default();
        tracker.captured.insert(InvariantKind::BoltInBounds);

        let mut app = test_app();
        app.insert_resource(tracker)
            .insert_resource(ViolationLog(vec![
                ViolationEntry {
                    frame: 1,
                    invariant: InvariantKind::BoltInBounds,
                    entity: None,
                    message: "oob 1".into(),
                },
                ViolationEntry {
                    frame: 2,
                    invariant: InvariantKind::BoltInBounds,
                    entity: None,
                    message: "oob 2".into(),
                },
                ViolationEntry {
                    frame: 3,
                    invariant: InvariantKind::BoltInBounds,
                    entity: None,
                    message: "oob 3".into(),
                },
                ViolationEntry {
                    frame: 4,
                    invariant: InvariantKind::BoltInBounds,
                    entity: None,
                    message: "oob 4".into(),
                },
                ViolationEntry {
                    frame: 5,
                    invariant: InvariantKind::BoltInBounds,
                    entity: None,
                    message: "oob 5".into(),
                },
            ]))
            .insert_resource(ScreenshotOutputDir(PathBuf::from(
                "/tmp/test-output/2026-04-07/0",
            )))
            .insert_resource(ScenarioName("dup_test".into()))
            .add_systems(Last, capture_violation_screenshots);

        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert_eq!(
            tracker.captured.len(),
            1,
            "tracker must still be len 1 with 5 duplicate BoltInBounds entries all already captured"
        );
        assert!(tracker.captured.contains(&InvariantKind::BoltInBounds));
    }

    /// Edge case for behavior 5: after the first tick filters already-captured
    /// and adds new, a second tick with the same log does not change captured.
    #[test]
    fn capture_violation_screenshots_filters_already_captured_second_tick_unchanged() {
        let mut tracker = ScreenshotTracker::default();
        tracker.captured.insert(InvariantKind::BoltInBounds);
        tracker.captured.insert(InvariantKind::NoNaN);

        let mut app = test_app();
        app.insert_resource(tracker)
            .insert_resource(ViolationLog(vec![
                ViolationEntry {
                    frame: 1,
                    invariant: InvariantKind::BoltInBounds,
                    entity: None,
                    message: "a".into(),
                },
                ViolationEntry {
                    frame: 2,
                    invariant: InvariantKind::NoNaN,
                    entity: None,
                    message: "b".into(),
                },
                ViolationEntry {
                    frame: 3,
                    invariant: InvariantKind::TimerNonNegative,
                    entity: None,
                    message: "c".into(),
                },
                ViolationEntry {
                    frame: 4,
                    invariant: InvariantKind::BreakerInBounds,
                    entity: None,
                    message: "d".into(),
                },
            ]))
            .insert_resource(ScreenshotOutputDir(PathBuf::from(
                "/tmp/test-output/2026-04-07/0",
            )))
            .insert_resource(ScenarioName("mixed_scenario".into()))
            .add_systems(Last, capture_violation_screenshots);

        // First tick: picks up TimerNonNegative + BreakerInBounds
        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert_eq!(
            tracker.captured.len(),
            4,
            "first tick: tracker must contain 4 kinds (2 pre-existing + 2 new)"
        );

        // Second tick with same log: no change
        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert_eq!(
            tracker.captured.len(),
            4,
            "second tick: tracker must still contain exactly 4 kinds (unchanged)"
        );
    }

    /// Edge case for behaviors 6/7: both `ScreenshotOutputDir` and `ScenarioName`
    /// absent — no crash, tracker empty.
    #[test]
    fn capture_violation_screenshots_no_crash_when_both_optional_resources_absent() {
        let mut app = test_app();
        app.insert_resource(ScreenshotTracker::default())
            .insert_resource(ViolationLog(vec![ViolationEntry {
                frame: 1,
                invariant: InvariantKind::NoNaN,
                entity: None,
                message: "nan".into(),
            }]))
            // No ScreenshotOutputDir inserted
            // No ScenarioName inserted
            .add_systems(Last, capture_violation_screenshots);

        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert!(
            tracker.captured.is_empty(),
            "tracker must remain empty when both ScreenshotOutputDir and ScenarioName are absent"
        );
    }

    /// Edge case for behavior 8: output directory already exists before the
    /// system runs — no error, tracker still updated.
    #[test]
    fn capture_violation_screenshots_no_error_when_output_dir_already_exists() {
        let temp_base =
            std::env::temp_dir().join(format!("screenshot_test_{}_preexist", std::process::id()));
        // Ensure the directory exists BEFORE the test
        std::fs::create_dir_all(&temp_base).expect("failed to pre-create temp dir");
        assert!(temp_base.exists(), "temp directory must exist before test");

        let mut app = test_app();
        app.insert_resource(ScreenshotTracker::default())
            .insert_resource(ViolationLog(vec![ViolationEntry {
                frame: 1,
                invariant: InvariantKind::BoltInBounds,
                entity: None,
                message: "oob".into(),
            }]))
            .insert_resource(ScreenshotOutputDir(temp_base.clone()))
            .insert_resource(ScenarioName("preexist_test".into()))
            .add_systems(Last, capture_violation_screenshots);

        tick(&mut app);

        let tracker = app.world().resource::<ScreenshotTracker>();
        assert!(
            tracker.captured.contains(&InvariantKind::BoltInBounds),
            "tracker must contain BoltInBounds even when output dir already existed"
        );

        // Cleanup
        drop(std::fs::remove_dir_all(&temp_base));
    }

    /// Concrete value for behavior 10: `detect_new_violations` returns exactly 22
    /// elements (not just `InvariantKind::ALL.len()`) when tracker is empty.
    #[test]
    fn detect_new_violations_returns_exactly_22_variants_from_empty_tracker() {
        let tracker = ScreenshotTracker::default();
        let log = ViolationLog(
            InvariantKind::ALL
                .iter()
                .enumerate()
                .map(|(i, &kind)| ViolationEntry {
                    frame: u32::try_from(i).expect("frame index fits u32"),
                    invariant: kind,
                    entity: None,
                    message: format!("{kind:?}"),
                })
                .collect(),
        );

        let result = detect_new_violations(&tracker, &log);

        assert_eq!(
            result.len(),
            22,
            "must return exactly 22 variants (concrete count), got {}",
            result.len()
        );
    }
}
