//! Tests for `parse_parallelism`, `Parallelism::resolve`, and `Parallelism` Display.

use crate::runner::execution::{Parallelism, parse_parallelism};

// -------------------------------------------------------------------------
// parse_parallelism — parses "all" or a positive integer
// -------------------------------------------------------------------------

#[test]
fn parse_parallelism_parses_all() {
    let result = parse_parallelism("all");
    assert_eq!(result, Ok(Parallelism::All));
}

#[test]
fn parse_parallelism_parses_all_case_insensitive() {
    assert_eq!(parse_parallelism("ALL"), Ok(Parallelism::All));
    assert_eq!(parse_parallelism("All"), Ok(Parallelism::All));
}

#[test]
fn parse_parallelism_parses_positive_number() {
    let result = parse_parallelism("8");
    assert_eq!(result, Ok(Parallelism::Count(8)));
}

#[test]
fn parse_parallelism_rejects_zero() {
    let result = parse_parallelism("0");
    assert!(result.is_err(), "expected error for 0, got: {result:?}");
}

#[test]
fn parse_parallelism_rejects_non_numeric_string() {
    let result = parse_parallelism("abc");
    assert!(result.is_err(), "expected error for 'abc', got: {result:?}");
}

// -------------------------------------------------------------------------
// Parallelism::resolve — resolves to concrete batch size
// -------------------------------------------------------------------------

#[test]
fn parallelism_count_resolves_to_given_value() {
    assert_eq!(Parallelism::Count(4).resolve(100), 4);
}

#[test]
fn parallelism_all_resolves_to_total() {
    assert_eq!(Parallelism::All.resolve(100), 100);
}

#[test]
fn parallelism_all_resolves_to_at_least_one() {
    assert_eq!(Parallelism::All.resolve(0), 1);
}

#[test]
fn parallelism_count_zero_resolves_to_one() {
    assert_eq!(Parallelism::Count(0).resolve(100), 1);
}

// -------------------------------------------------------------------------
// Parallelism::Display — formats for user-facing output
// -------------------------------------------------------------------------

#[test]
fn parallelism_display_count() {
    assert_eq!(Parallelism::Count(4).to_string(), "4");
}

#[test]
fn parallelism_display_all() {
    assert_eq!(Parallelism::All.to_string(), "all");
}
