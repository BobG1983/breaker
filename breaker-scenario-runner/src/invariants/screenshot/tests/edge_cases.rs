use std::path::PathBuf;

use bevy::prelude::*;

use super::super::{super::ViolationEntry, system::*};
use crate::{invariants::ViolationLog, types::InvariantKind};

// =========================================================================
// Additional edge cases from reviewer feedback
// =========================================================================

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

fn tick(app: &mut App) {
    app.update();
}

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
                frame:     1,
                invariant: InvariantKind::BoltInBounds,
                entity:    None,
                message:   "oob 1".into(),
            },
            ViolationEntry {
                frame:     2,
                invariant: InvariantKind::BoltInBounds,
                entity:    None,
                message:   "oob 2".into(),
            },
            ViolationEntry {
                frame:     3,
                invariant: InvariantKind::BoltInBounds,
                entity:    None,
                message:   "oob 3".into(),
            },
            ViolationEntry {
                frame:     4,
                invariant: InvariantKind::BoltInBounds,
                entity:    None,
                message:   "oob 4".into(),
            },
            ViolationEntry {
                frame:     5,
                invariant: InvariantKind::BoltInBounds,
                entity:    None,
                message:   "oob 5".into(),
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
                frame:     1,
                invariant: InvariantKind::BoltInBounds,
                entity:    None,
                message:   "a".into(),
            },
            ViolationEntry {
                frame:     2,
                invariant: InvariantKind::NoNaN,
                entity:    None,
                message:   "b".into(),
            },
            ViolationEntry {
                frame:     3,
                invariant: InvariantKind::TimerNonNegative,
                entity:    None,
                message:   "c".into(),
            },
            ViolationEntry {
                frame:     4,
                invariant: InvariantKind::BreakerInBounds,
                entity:    None,
                message:   "d".into(),
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
            frame:     1,
            invariant: InvariantKind::NoNaN,
            entity:    None,
            message:   "nan".into(),
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
            frame:     1,
            invariant: InvariantKind::BoltInBounds,
            entity:    None,
            message:   "oob".into(),
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
fn detect_new_violations_returns_exactly_23_variants_from_empty_tracker() {
    let tracker = ScreenshotTracker::default();
    let log = ViolationLog(
        InvariantKind::ALL
            .iter()
            .enumerate()
            .map(|(i, &kind)| ViolationEntry {
                frame:     u32::try_from(i).expect("frame index fits u32"),
                invariant: kind,
                entity:    None,
                message:   format!("{kind:?}"),
            })
            .collect(),
    );

    let result = detect_new_violations(&tracker, &log);

    assert_eq!(
        result.len(),
        23,
        "must return exactly 23 variants (concrete count), got {}",
        result.len()
    );
}
