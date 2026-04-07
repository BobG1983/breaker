use super::super::system::*;
use crate::types::InvariantKind;

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
