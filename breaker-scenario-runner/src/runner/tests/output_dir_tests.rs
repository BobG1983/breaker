//! Tests for `output_dir` — structured output directories and violation log files.

use std::{fs, path::PathBuf};

use bevy::prelude::*;

use crate::{
    invariants::ViolationEntry,
    runner::output_dir::{
        clean_output_dir, create_run_dir, format_violation_entry, next_run_number,
        today_date_string, write_violations_log,
    },
    types::InvariantKind,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates an isolated temp directory for a single test, returning its path.
///
/// The caller is responsible for cleaning up via `fs::remove_dir_all`.
fn test_temp_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("breaker_test_{}_{test_name}", std::process::id()));
    // Clean up any leftover from a previous interrupted run.
    drop(fs::remove_dir_all(&dir));
    fs::create_dir_all(&dir).expect("failed to create test temp dir");
    dir
}

fn make_violation(invariant: InvariantKind, frame: u32, message: &str) -> ViolationEntry {
    ViolationEntry {
        frame,
        invariant,
        entity: None,
        message: message.to_owned(),
    }
}

fn make_violation_with_entity(
    invariant: InvariantKind,
    frame: u32,
    entity: Entity,
    message: &str,
) -> ViolationEntry {
    ViolationEntry {
        frame,
        invariant,
        entity: Some(entity),
        message: message.to_owned(),
    }
}

// =========================================================================
// next_run_number
// =========================================================================

#[test]
fn next_run_number_returns_zero_when_date_dir_does_not_exist() {
    let base = test_temp_dir("nrn_no_date_dir");
    // Do NOT create the date subdirectory.
    let result = next_run_number(&base, "2026-04-06");
    drop(fs::remove_dir_all(&base));
    assert_eq!(
        result, 0,
        "must return 0 when date directory does not exist"
    );
}

#[test]
fn next_run_number_returns_zero_when_base_dir_does_not_exist() {
    let base = std::env::temp_dir().join(format!(
        "breaker_test_{}_nrn_no_base_dir",
        std::process::id()
    ));
    // Ensure the base dir does not exist at all.
    drop(fs::remove_dir_all(&base));
    let result = next_run_number(&base, "2026-04-06");
    assert_eq!(
        result, 0,
        "must return 0 when base_dir itself does not exist"
    );
}

#[test]
fn next_run_number_returns_zero_when_date_dir_is_empty() {
    let base = test_temp_dir("nrn_empty_date");
    let date_dir = base.join("2026-04-06");
    fs::create_dir_all(&date_dir).unwrap();
    let result = next_run_number(&base, "2026-04-06");
    drop(fs::remove_dir_all(&base));
    assert_eq!(
        result, 0,
        "must return 0 when date directory exists but is empty"
    );
}

#[test]
fn next_run_number_returns_zero_when_date_dir_contains_only_files() {
    let base = test_temp_dir("nrn_files_only");
    let date_dir = base.join("2026-04-06");
    fs::create_dir_all(&date_dir).unwrap();
    fs::write(date_dir.join("notes.txt"), "some notes").unwrap();
    let result = next_run_number(&base, "2026-04-06");
    drop(fs::remove_dir_all(&base));
    assert_eq!(
        result, 0,
        "must return 0 when date dir contains only regular files"
    );
}

#[test]
fn next_run_number_returns_max_plus_one_for_contiguous_dirs() {
    let base = test_temp_dir("nrn_contiguous");
    let date_dir = base.join("2026-04-06");
    for n in 0..3 {
        fs::create_dir_all(date_dir.join(n.to_string())).unwrap();
    }
    let result = next_run_number(&base, "2026-04-06");
    drop(fs::remove_dir_all(&base));
    assert_eq!(result, 3, "must return 3 when dirs 0, 1, 2 exist");
}

#[test]
fn next_run_number_returns_max_plus_one_for_non_contiguous_dirs() {
    let base = test_temp_dir("nrn_non_contiguous");
    let date_dir = base.join("2026-04-06");
    for n in [0, 2, 5] {
        fs::create_dir_all(date_dir.join(n.to_string())).unwrap();
    }
    let result = next_run_number(&base, "2026-04-06");
    drop(fs::remove_dir_all(&base));
    assert_eq!(
        result, 6,
        "must return 6 (max 5 + 1) for non-contiguous dirs 0, 2, 5"
    );
}

#[test]
fn next_run_number_ignores_non_numeric_directory_names() {
    let base = test_temp_dir("nrn_non_numeric");
    let date_dir = base.join("2026-04-06");
    for name in ["0", "1", "notes", "temp"] {
        fs::create_dir_all(date_dir.join(name)).unwrap();
    }
    let result = next_run_number(&base, "2026-04-06");
    drop(fs::remove_dir_all(&base));
    assert_eq!(
        result, 2,
        "must return 2 (max numeric 1 + 1), ignoring non-numeric names"
    );
}

#[test]
fn next_run_number_returns_zero_when_only_non_numeric_names_exist() {
    let base = test_temp_dir("nrn_only_non_numeric");
    let date_dir = base.join("2026-04-06");
    for name in ["notes", "temp"] {
        fs::create_dir_all(date_dir.join(name)).unwrap();
    }
    let result = next_run_number(&base, "2026-04-06");
    drop(fs::remove_dir_all(&base));
    assert_eq!(
        result, 0,
        "must return 0 when only non-numeric directory names exist"
    );
}

// =========================================================================
// format_violation_entry
// =========================================================================

#[test]
fn format_violation_entry_with_entity_includes_entity_debug() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let entry = make_violation_with_entity(
        InvariantKind::BoltInBounds,
        42,
        entity,
        "bolt at (-50.0, 300.0) outside bounds",
    );
    let result = format_violation_entry(&entry);
    let expected = format!(
        "[frame 42] BoltInBounds: bolt at (-50.0, 300.0) outside bounds (entity: {entity:?})"
    );
    assert_eq!(result, expected);
}

#[test]
fn format_violation_entry_different_invariant_kind_uses_debug_name() {
    let entry = make_violation(
        InvariantKind::TimerMonotonicallyDecreasing,
        5,
        "timer went up",
    );
    let result = format_violation_entry(&entry);
    assert_eq!(
        result,
        "[frame 5] TimerMonotonicallyDecreasing: timer went up"
    );
}

#[test]
fn format_violation_entry_without_entity_omits_entity_suffix() {
    let entry = make_violation(InvariantKind::NoNaN, 100, "NaN in transform");
    let result = format_violation_entry(&entry);
    assert_eq!(result, "[frame 100] NoNaN: NaN in transform");
}

#[test]
fn format_violation_entry_empty_message() {
    let entry = make_violation(InvariantKind::NoNaN, 100, "");
    let result = format_violation_entry(&entry);
    assert_eq!(result, "[frame 100] NoNaN: ");
}

// =========================================================================
// write_violations_log
// =========================================================================

#[test]
fn write_violations_log_creates_scenario_subdir_and_log_file() {
    let run_dir = test_temp_dir("wvl_creates_file");
    let violations = vec![
        make_violation(InvariantKind::BoltInBounds, 10, "oob"),
        make_violation(InvariantKind::NoNaN, 20, "nan"),
    ];
    write_violations_log(&run_dir, "aegis_chaos", &violations).expect("write must succeed");

    let log_path = run_dir.join("aegis_chaos").join("violations.log");
    assert!(log_path.exists(), "violations.log must be created");

    let contents = fs::read_to_string(&log_path).expect("must read log file");
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 2, "log must contain exactly 2 lines");

    let expected_line_0 = format_violation_entry(&violations[0]);
    let expected_line_1 = format_violation_entry(&violations[1]);
    assert_eq!(
        lines[0], expected_line_0,
        "first line must match formatted violation 0"
    );
    assert_eq!(
        lines[1], expected_line_1,
        "second line must match formatted violation 1"
    );

    drop(fs::remove_dir_all(&run_dir));
}

#[test]
fn write_violations_log_empty_violations_does_not_create_anything() {
    let run_dir = test_temp_dir("wvl_empty_noop");
    write_violations_log(&run_dir, "aegis_chaos", &[]).expect("write must succeed");

    assert!(
        !run_dir.join("aegis_chaos").exists(),
        "scenario subdirectory must not be created for empty violations"
    );

    drop(fs::remove_dir_all(&run_dir));
}

#[test]
fn write_violations_log_overwrites_on_repeated_calls() {
    let run_dir = test_temp_dir("wvl_overwrite");

    // First call with 3 violations.
    let first_violations = vec![
        make_violation(InvariantKind::BoltInBounds, 1, "v1"),
        make_violation(InvariantKind::BoltInBounds, 2, "v2"),
        make_violation(InvariantKind::BoltInBounds, 3, "v3"),
    ];
    write_violations_log(&run_dir, "aegis_chaos", &first_violations).expect("first write");

    let log_path = run_dir.join("aegis_chaos").join("violations.log");
    let first_contents = fs::read_to_string(&log_path).expect("must read first log");
    assert_eq!(
        first_contents.lines().count(),
        3,
        "first write must produce 3 lines"
    );

    // Second call with 1 violation — must overwrite, not append.
    let second_violations = vec![make_violation(InvariantKind::NoNaN, 99, "single")];
    write_violations_log(&run_dir, "aegis_chaos", &second_violations).expect("second write");

    let second_contents = fs::read_to_string(&log_path).expect("must read second log");
    assert_eq!(
        second_contents.lines().count(),
        1,
        "second write must overwrite to exactly 1 line, not append"
    );

    drop(fs::remove_dir_all(&run_dir));
}

// =========================================================================
// create_run_dir
// =========================================================================

#[test]
fn create_run_dir_creates_path_and_returns_it() {
    let base = test_temp_dir("crd_basic");
    let result = create_run_dir(&base, "2026-04-06").expect("create_run_dir must succeed");

    let expected = base.join("2026-04-06").join("0");
    assert_eq!(
        result, expected,
        "returned path must be <base>/2026-04-06/0/"
    );
    assert!(
        result.is_dir(),
        "the returned path must exist as a directory"
    );

    drop(fs::remove_dir_all(&base));
}

#[test]
fn create_run_dir_increments_run_number_when_prior_runs_exist() {
    let base = test_temp_dir("crd_increment");
    // Pre-create run dirs 0, 1, 2 for the date.
    let date_dir = base.join("2026-04-06");
    for n in 0..3 {
        fs::create_dir_all(date_dir.join(n.to_string())).unwrap();
    }

    let result = create_run_dir(&base, "2026-04-06").expect("create_run_dir must succeed");

    let expected = base.join("2026-04-06").join("3");
    assert_eq!(
        result, expected,
        "returned path must be <base>/2026-04-06/3/ when 0, 1, 2 already exist"
    );
    assert!(result.is_dir(), "run dir 3 must exist as a directory");

    drop(fs::remove_dir_all(&base));
}

// =========================================================================
// today_date_string
// =========================================================================

#[test]
fn today_date_string_returns_valid_yyyy_mm_dd_format() {
    let result = today_date_string();
    assert_eq!(
        result.len(),
        10,
        "date string must be exactly 10 characters"
    );
    // Validate format: YYYY-MM-DD
    let parts: Vec<&str> = result.split('-').collect();
    assert_eq!(parts.len(), 3, "date must have 3 parts separated by '-'");
    assert_eq!(parts[0].len(), 4, "year must be 4 digits");
    assert_eq!(parts[1].len(), 2, "month must be 2 digits");
    assert_eq!(parts[2].len(), 2, "day must be 2 digits");
    assert!(
        parts[0].chars().all(|c| c.is_ascii_digit()),
        "year must be all digits"
    );
    assert!(
        parts[1].chars().all(|c| c.is_ascii_digit()),
        "month must be all digits"
    );
    assert!(
        parts[2].chars().all(|c| c.is_ascii_digit()),
        "day must be all digits"
    );
}

// =========================================================================
// clean_output_dir
// =========================================================================

#[test]
fn clean_output_dir_removes_entire_base_dir_recursively() {
    let base = test_temp_dir("cod_remove");

    // Build a nested structure mimicking real output.
    let nested = base.join("2026-04-05").join("0").join("aegis_chaos");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("violations.log"), "some log data").unwrap();
    fs::create_dir_all(base.join("2026-04-06").join("0")).unwrap();

    clean_output_dir(&base).expect("clean_output_dir must succeed");

    assert!(
        !base.exists(),
        "entire base directory must be removed after clean"
    );
}

#[test]
fn clean_output_dir_succeeds_when_dir_does_not_exist() {
    let base = std::env::temp_dir().join(format!(
        "breaker_test_{}_cod_nonexistent",
        std::process::id()
    ));
    // Ensure it does not exist.
    drop(fs::remove_dir_all(&base));

    let result = clean_output_dir(&base);
    assert!(
        result.is_ok(),
        "clean_output_dir must return Ok(()) when dir does not exist"
    );
}
