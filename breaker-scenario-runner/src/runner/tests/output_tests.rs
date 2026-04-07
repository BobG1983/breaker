//! Tests for `group_violations`, `group_logs`, and `is_invariant_fail_reason`.

use crate::{
    invariants::ViolationEntry,
    log_capture::LogEntry,
    runner::output::{group_logs, group_violations, is_invariant_fail_reason},
    types::InvariantKind,
};

// -------------------------------------------------------------------------
// group_violations — groups by invariant kind
// -------------------------------------------------------------------------

fn make_violation(invariant: InvariantKind, frame: u32) -> ViolationEntry {
    ViolationEntry {
        frame,
        invariant,
        entity: None,
        message: format!("test: {invariant:?}"),
    }
}

#[test]
fn group_violations_groups_by_invariant_kind() {
    let violations = vec![
        make_violation(InvariantKind::BoltInBounds, 100),
        make_violation(InvariantKind::BoltInBounds, 101),
        make_violation(InvariantKind::BoltInBounds, 105),
    ];

    let groups = group_violations(&violations);

    assert_eq!(
        groups.len(),
        1,
        "3 same-kind violations must produce 1 group"
    );
    assert_eq!(groups[0].invariant, InvariantKind::BoltInBounds);
    assert_eq!(groups[0].count, 3);
    assert_eq!(groups[0].first_frame, 100);
    assert_eq!(groups[0].last_frame, 105);
}

#[test]
fn group_violations_separates_different_invariant_kinds() {
    let violations = vec![
        make_violation(InvariantKind::BoltInBounds, 10),
        make_violation(InvariantKind::NoNaN, 20),
        make_violation(InvariantKind::BoltInBounds, 30),
    ];

    let groups = group_violations(&violations);

    assert_eq!(
        groups.len(),
        2,
        "BoltInBounds + NoNaN must produce 2 groups"
    );
    let bolt = groups
        .iter()
        .find(|g| g.invariant == InvariantKind::BoltInBounds)
        .unwrap();
    let nan = groups
        .iter()
        .find(|g| g.invariant == InvariantKind::NoNaN)
        .unwrap();
    assert_eq!(bolt.count, 2);
    assert_eq!(bolt.first_frame, 10);
    assert_eq!(bolt.last_frame, 30);
    assert_eq!(nan.count, 1);
    assert_eq!(nan.first_frame, 20);
    assert_eq!(nan.last_frame, 20);
}

#[test]
fn group_violations_single_entry_has_matching_first_last_frame() {
    let violations = vec![make_violation(InvariantKind::NoNaN, 42)];

    let groups = group_violations(&violations);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].first_frame, 42);
    assert_eq!(groups[0].last_frame, 42);
    assert_eq!(groups[0].count, 1);
}

// -------------------------------------------------------------------------
// group_logs — groups by level + message
// -------------------------------------------------------------------------

fn make_log(level: bevy::log::Level, message: &str, frame: u32) -> LogEntry {
    LogEntry {
        level,
        target: "breaker::test".to_owned(),
        message: message.to_owned(),
        frame,
    }
}

#[test]
fn group_logs_groups_by_level_and_message() {
    let logs = vec![
        make_log(bevy::log::Level::WARN, "bad thing", 100),
        make_log(bevy::log::Level::WARN, "bad thing", 200),
        make_log(bevy::log::Level::WARN, "bad thing", 300),
    ];

    let groups = group_logs(&logs);

    assert_eq!(groups.len(), 1, "3 identical logs must produce 1 group");
    assert_eq!(groups[0].count, 3);
    assert_eq!(groups[0].first_frame, 100);
    assert_eq!(groups[0].last_frame, 300);
    assert_eq!(groups[0].message, "bad thing");
}

#[test]
fn group_logs_separates_different_messages() {
    let logs = vec![
        make_log(bevy::log::Level::WARN, "msg a", 10),
        make_log(bevy::log::Level::WARN, "msg b", 20),
    ];

    let groups = group_logs(&logs);

    assert_eq!(
        groups.len(),
        2,
        "2 different messages must produce 2 groups"
    );
}

#[test]
fn group_logs_separates_different_levels_same_message() {
    let logs = vec![
        make_log(bevy::log::Level::WARN, "same msg", 10),
        make_log(bevy::log::Level::ERROR, "same msg", 20),
    ];

    let groups = group_logs(&logs);

    assert_eq!(
        groups.len(),
        2,
        "WARN + ERROR with same message must produce 2 groups"
    );
}

// -------------------------------------------------------------------------
// is_invariant_fail_reason — matches all InvariantKind fail reasons
// -------------------------------------------------------------------------

#[test]
fn is_invariant_fail_reason_returns_true_for_all_invariant_kinds() {
    for variant in InvariantKind::ALL {
        assert!(
            is_invariant_fail_reason(variant.fail_reason()),
            "is_invariant_fail_reason must return true for {:?} fail_reason: {:?}",
            variant,
            variant.fail_reason()
        );
    }
}

#[test]
fn is_invariant_fail_reason_returns_false_for_health_check_strings() {
    assert!(
        !is_invariant_fail_reason("no actions were injected during scenario run"),
        "health check string must not match as invariant fail reason"
    );
    assert!(
        !is_invariant_fail_reason("scenario never entered Playing state"),
        "health check string must not match as invariant fail reason"
    );
}
