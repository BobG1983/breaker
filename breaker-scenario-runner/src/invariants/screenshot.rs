//! First-failure screenshot tracking: detects new invariant violations,
//! formats screenshot output paths, and tracks which kinds have been captured.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use bevy::prelude::*;

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
    _commands: Commands,
    _tracker: ResMut<ScreenshotTracker>,
    _vl: Res<ViolationLog>,
    _output_dir: Option<Res<ScreenshotOutputDir>>,
    _scenario_name: Option<Res<ScenarioName>>,
) {
    // Stub — no-op. Production logic will be added in the GREEN phase.
}

#[cfg(test)]
mod tests {
    use super::{super::ViolationEntry, *};

    // =========================================================================
    // ScreenshotTracker — construction and mark_captured
    // =========================================================================

    /// Default-constructed `ScreenshotTracker` has an empty `captured` set.
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
                "/tmp/out/2026-04-06/1/bolt_speed_stress/TimerMonotonicallyDecreasing.png"
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
                "/tmp/breaker-scenario-runner/2026-04-06/0/self_test_bolt_in_bounds/NoNaN.png"
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
            PathBuf::from("/out/test/NoEntityLeaks.png"),
            "very short path components must still produce valid output"
        );
    }
}
