use super::super::{forward_grade, retroactive_grade};
use crate::breaker::messages::BumpGrade;

// ── Pure grade helper tests ──────────────────────────────────────

#[test]
fn forward_just_activated_is_early() {
    // Timer at max (just pressed) — well above perfect_window
    let grade = forward_grade(0.20, 0.05);
    assert_eq!(grade, BumpGrade::Early);
}

#[test]
fn forward_at_perfect_boundary_is_perfect() {
    let grade = forward_grade(0.05, 0.05);
    assert_eq!(grade, BumpGrade::Perfect);
}

#[test]
fn forward_within_perfect_zone_is_perfect() {
    let grade = forward_grade(0.02, 0.05);
    assert_eq!(grade, BumpGrade::Perfect);
}

#[test]
fn forward_just_outside_perfect_is_early() {
    let grade = forward_grade(0.05 + 0.001, 0.05);
    assert_eq!(grade, BumpGrade::Early);
}

#[test]
fn retroactive_immediate_is_perfect() {
    let grade = retroactive_grade(0.0, 0.05);
    assert_eq!(grade, BumpGrade::Perfect);
}

#[test]
fn retroactive_at_boundary_is_perfect() {
    let grade = retroactive_grade(0.05, 0.05);
    assert_eq!(grade, BumpGrade::Perfect);
}

#[test]
fn retroactive_just_past_boundary_is_late() {
    let grade = retroactive_grade(0.05 + 0.001, 0.05);
    assert_eq!(grade, BumpGrade::Late);
}
