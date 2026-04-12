use super::super::{super::ViolationEntry, system::*};
use crate::{invariants::ViolationLog, types::InvariantKind};

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
            frame:     10,
            invariant: InvariantKind::BoltInBounds,
            entity:    None,
            message:   "oob".into(),
        },
        ViolationEntry {
            frame:     12,
            invariant: InvariantKind::NoNaN,
            entity:    None,
            message:   "nan x".into(),
        },
        ViolationEntry {
            frame:     14,
            invariant: InvariantKind::NoNaN,
            entity:    None,
            message:   "nan y".into(),
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
            frame:     5,
            invariant: InvariantKind::BoltInBounds,
            entity:    None,
            message:   "oob".into(),
        },
        ViolationEntry {
            frame:     7,
            invariant: InvariantKind::NoNaN,
            entity:    None,
            message:   "nan".into(),
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
        frame:     1,
        invariant: InvariantKind::NoNaN,
        entity:    None,
        message:   "nan".into(),
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
            frame:     1,
            invariant: InvariantKind::BoltInBounds,
            entity:    None,
            message:   "oob".into(),
        },
        ViolationEntry {
            frame:     2,
            invariant: InvariantKind::NoNaN,
            entity:    None,
            message:   "nan".into(),
        },
        ViolationEntry {
            frame:     3,
            invariant: InvariantKind::TimerNonNegative,
            entity:    None,
            message:   "neg".into(),
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
