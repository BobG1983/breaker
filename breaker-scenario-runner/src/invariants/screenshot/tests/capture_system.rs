use std::path::PathBuf;

use bevy::prelude::*;

use super::super::{super::ViolationEntry, system::*};
use crate::{invariants::ViolationLog, types::InvariantKind};

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
            frame:     5,
            invariant: InvariantKind::BoltInBounds,
            entity:    None,
            message:   "oob".into(),
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
            frame:     5,
            invariant: InvariantKind::BoltInBounds,
            entity:    None,
            message:   "oob".into(),
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
                frame:     10,
                invariant: InvariantKind::BoltInBounds,
                entity:    None,
                message:   "oob".into(),
            },
            ViolationEntry {
                frame:     10,
                invariant: InvariantKind::NoNaN,
                entity:    None,
                message:   "nan".into(),
            },
            ViolationEntry {
                frame:     10,
                invariant: InvariantKind::TimerNonNegative,
                entity:    None,
                message:   "neg".into(),
            },
            // Duplicate BoltInBounds entry — should not affect count
            ViolationEntry {
                frame:     10,
                invariant: InvariantKind::BoltInBounds,
                entity:    None,
                message:   "oob again".into(),
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
            frame:     1,
            invariant: InvariantKind::NoNaN,
            entity:    None,
            message:   "nan".into(),
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
            frame:     1,
            invariant: InvariantKind::NoNaN,
            entity:    None,
            message:   "nan".into(),
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
            frame:     1,
            invariant: InvariantKind::NoNaN,
            entity:    None,
            message:   "nan".into(),
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
            frame:     1,
            invariant: InvariantKind::NoNaN,
            entity:    None,
            message:   "nan".into(),
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
            frame:     1,
            invariant: InvariantKind::BoltInBounds,
            entity:    None,
            message:   "oob".into(),
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
            frame:     1,
            invariant: InvariantKind::NoNaN,
            entity:    None,
            message:   "nan".into(),
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
            frame:     1,
            invariant: InvariantKind::NoNaN,
            entity:    None,
            message:   "nan".into(),
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
            frame:     2,
            invariant: InvariantKind::BreakerInBounds,
            entity:    None,
            message:   "breaker oob".into(),
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
