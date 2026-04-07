//! Tests for `now_timestamp` — behaviors 1-2.

use crate::runner::run_log::now_timestamp;

// =========================================================================
// Section A: Timestamp formatting (`now_timestamp`) — behaviors 1-2
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 1: now_timestamp returns a filesystem-safe ISO 8601 string
// -------------------------------------------------------------------------

#[test]
fn now_timestamp_returns_19_char_filesystem_safe_string() {
    let result = now_timestamp();
    assert_eq!(
        result.len(),
        19,
        "timestamp must be exactly 19 characters (YYYY-MM-DDTHH-MM-SS), got {} chars: {result:?}",
        result.len()
    );
}

#[test]
fn now_timestamp_matches_pattern_yyyy_mm_dd_t_hh_mm_ss() {
    let result = now_timestamp();
    // Validate: YYYY-MM-DDTHH-MM-SS
    let parts: Vec<&str> = result.split('T').collect();
    assert_eq!(
        parts.len(),
        2,
        "timestamp must have exactly one T separator, got: {result:?}"
    );

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    assert_eq!(
        date_parts.len(),
        3,
        "date portion must have 3 parts: {result:?}"
    );
    assert_eq!(date_parts[0].len(), 4, "year must be 4 digits: {result:?}");
    assert_eq!(date_parts[1].len(), 2, "month must be 2 digits: {result:?}");
    assert_eq!(date_parts[2].len(), 2, "day must be 2 digits: {result:?}");

    let time_parts: Vec<&str> = parts[1].split('-').collect();
    assert_eq!(
        time_parts.len(),
        3,
        "time portion must have 3 parts separated by hyphens: {result:?}"
    );
    assert_eq!(time_parts[0].len(), 2, "hour must be 2 digits: {result:?}");
    assert_eq!(
        time_parts[1].len(),
        2,
        "minute must be 2 digits: {result:?}"
    );
    assert_eq!(
        time_parts[2].len(),
        2,
        "second must be 2 digits: {result:?}"
    );
}

// -------------------------------------------------------------------------
// Behavior 1 edge case: no colons in the timestamp
// -------------------------------------------------------------------------

#[test]
fn now_timestamp_contains_no_colons() {
    let result = now_timestamp();
    assert!(
        !result.contains(':'),
        "timestamp must not contain colons (invalid in filenames), got: {result:?}"
    );
}

// -------------------------------------------------------------------------
// Behavior 2: hyphens used instead of colons for time components
// -------------------------------------------------------------------------

#[test]
fn now_timestamp_time_portion_uses_hyphens() {
    let result = now_timestamp();
    let time_portion = result
        .split('T')
        .nth(1)
        .expect("timestamp must contain a T separator");
    assert!(
        time_portion.contains('-'),
        "time portion must use hyphens as separators, got: {time_portion:?}"
    );
    // All characters in the time portion should be digits or hyphens.
    assert!(
        time_portion.chars().all(|c| c.is_ascii_digit() || c == '-'),
        "time portion must only contain digits and hyphens, got: {time_portion:?}"
    );
}
