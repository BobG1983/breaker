//! Group M — multi-bolt scenarios (Behaviors 12–13).
//!
//! Pins that:
//! - In a 3-bolt world (1 primary + 2 extras), a Perfect bump on one extra
//!   swaps only that extra with the primary — the uninvolved extra is left
//!   untouched across markers AND effect components.
//! - A sequence of Perfect bumps on different extras walks markers and
//!   effect components through deterministic chained swaps.

use super::helpers::{
    bound_fingerprints, build_conductor_app, has_extra, has_primary, make_distinct_bound,
    make_distinct_staged, seed_active_protocols_with_conductor, spawn_dummy_breaker,
    spawn_extra_bolt_with_bound, spawn_extra_bolt_with_bound_and_staged,
    spawn_primary_bolt_with_bound, spawn_primary_bolt_with_bound_and_staged, staged_fingerprints,
    write_bump_performed,
};
use crate::{breaker::messages::BumpGrade, prelude::*};

// ── Behavior 12 — 1 primary + 2 extras: bump on extra_a leaves extra_b alone

#[test]
fn perfect_bump_on_one_of_two_extras_leaves_the_other_extra_untouched() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PB"));
    let extra_a = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("AB"));
    let extra_b = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("BB"));

    write_bump_performed(&mut app, breaker, Some(extra_a), BumpGrade::Perfect);
    tick(&mut app);

    // extra_a ↔ primary swapped.
    assert!(has_primary(&app, extra_a));
    assert!(!has_extra(&app, extra_a));
    assert!(!has_primary(&app, primary));
    assert!(has_extra(&app, primary));

    // extra_b untouched.
    assert!(!has_primary(&app, extra_b), "extra_b still not primary");
    assert!(has_extra(&app, extra_b), "extra_b still carries ExtraBolt");

    // BoundEffects: A ↔ P swap; B untouched.
    assert_eq!(
        bound_fingerprints(&app, extra_a),
        vec!["PB".to_string()],
        "extra_a received the primary's bound"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["AB".to_string()],
        "primary received extra_a's bound"
    );
    assert_eq!(
        bound_fingerprints(&app, extra_b),
        vec!["BB".to_string()],
        "extra_b's bound untouched"
    );
}

// ── Behavior 12 (edge case) — StagedEffects also swap only between A and P

#[test]
fn perfect_bump_on_one_of_two_extras_swaps_staged_only_between_involved_pair() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound_and_staged(
        &mut app,
        make_distinct_bound("PB"),
        make_distinct_staged("PS"),
    );
    let extra_a = spawn_extra_bolt_with_bound_and_staged(
        &mut app,
        make_distinct_bound("AB"),
        make_distinct_staged("AS"),
    );
    let extra_b = spawn_extra_bolt_with_bound_and_staged(
        &mut app,
        make_distinct_bound("BB"),
        make_distinct_staged("BS"),
    );

    write_bump_performed(&mut app, breaker, Some(extra_a), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        staged_fingerprints(&app, extra_a),
        vec!["PS".to_string()],
        "extra_a received primary's staged"
    );
    assert_eq!(
        staged_fingerprints(&app, primary),
        vec!["AS".to_string()],
        "primary received extra_a's staged"
    );
    assert_eq!(
        staged_fingerprints(&app, extra_b),
        vec!["BS".to_string()],
        "extra_b's staged untouched"
    );
}

// ── Behavior 13 — two bumps in sequence on different extras: chained promotion

#[test]
fn two_perfect_bumps_on_different_extras_chain_marker_and_bound_swaps() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PB"));
    let extra_a = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("AB"));
    let extra_b = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("BB"));

    // Tick 1 — bump A.
    write_bump_performed(&mut app, breaker, Some(extra_a), BumpGrade::Perfect);
    tick(&mut app);

    // Tick 2 — bump B.
    write_bump_performed(&mut app, breaker, Some(extra_b), BumpGrade::Perfect);
    tick(&mut app);

    // After tick 1: A holds "PB", primary holds "AB", B unchanged "BB".
    // After tick 2 (bump B): B takes what A held = "PB"; A takes what B held = "BB".
    // primary never involved in tick 2 → still "AB".

    assert!(has_primary(&app, extra_b), "B promoted on tick 2");
    assert!(!has_extra(&app, extra_b), "B lost ExtraBolt on promotion");
    assert!(!has_primary(&app, extra_a), "A demoted on tick 2");
    assert!(has_extra(&app, extra_a), "A carries ExtraBolt again");
    assert!(
        !has_primary(&app, primary),
        "primary still demoted from tick 1"
    );
    assert!(
        has_extra(&app, primary),
        "primary carries ExtraBolt since tick 1"
    );

    assert_eq!(
        bound_fingerprints(&app, extra_b),
        vec!["PB".to_string()],
        "B holds what A held after tick 1"
    );
    assert_eq!(
        bound_fingerprints(&app, extra_a),
        vec!["BB".to_string()],
        "A holds what B held after tick 1"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["AB".to_string()],
        "primary frozen at what A held pre-tick-1"
    );
}

// ── Behavior 13 (edge case) — three bumps in sequence (A, B, A) ─────────────

#[test]
fn three_bumps_in_sequence_a_b_a_walk_through_deterministic_swaps() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PB"));
    let extra_a = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("AB"));
    let extra_b = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("BB"));

    // Walk the spec edge case:
    // After bump-A-1: A=primary holding PB; P=extra holding AB; B=extra holding BB.
    // After bump-B-1: B=primary holding PB; A=extra holding BB; P=extra holding AB.
    // After bump-A-2: A=primary holding PB; B=extra holding BB; P=extra holding AB.

    write_bump_performed(&mut app, breaker, Some(extra_a), BumpGrade::Perfect);
    tick(&mut app);
    write_bump_performed(&mut app, breaker, Some(extra_b), BumpGrade::Perfect);
    tick(&mut app);
    write_bump_performed(&mut app, breaker, Some(extra_a), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        has_primary(&app, extra_a),
        "A holds PrimaryBolt after 3 bumps"
    );
    assert_eq!(
        bound_fingerprints(&app, extra_a),
        vec!["PB".to_string()],
        "A holds PB throughout chain"
    );
    assert_eq!(
        bound_fingerprints(&app, extra_b),
        vec!["BB".to_string()],
        "B holds BB after A-B-A chain"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["AB".to_string()],
        "primary holds AB — frozen since tick 1"
    );
}

// ── Regression — same-tick double bump on two distinct extras ───────────────
//
// Pins that when two Perfect bumps on different extras arrive in the SAME tick,
// exactly one entity ends up carrying `PrimaryBolt` after the tick. The bug:
// `primary.single()` inside the inner reader loop returns the stale (pre-flush)
// primary for the second message, so both extras are promoted to `PrimaryBolt`
// and the exactly-one-primary invariant is violated.

#[test]
fn two_perfect_bumps_on_distinct_extras_same_tick_preserves_single_primary() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("P"));
    let extra_a = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("A"));
    let extra_b = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("B"));

    // Two Perfect bumps on distinct extras, same tick.
    write_bump_performed(&mut app, breaker, Some(extra_a), BumpGrade::Perfect);
    write_bump_performed(&mut app, breaker, Some(extra_b), BumpGrade::Perfect);
    tick(&mut app);

    // Exactly one of {P, A, B} carries PrimaryBolt.
    let primary_count = [primary, extra_a, extra_b]
        .into_iter()
        .filter(|&e| has_primary(&app, e))
        .count();
    assert_eq!(
        primary_count, 1,
        "exactly one of the three bolts must carry PrimaryBolt after same-tick double swap"
    );

    // No entity carries BOTH PrimaryBolt and ExtraBolt simultaneously.
    for entity in [primary, extra_a, extra_b] {
        assert!(
            !(has_primary(&app, entity) && has_extra(&app, entity)),
            "entity {entity:?} must not carry both PrimaryBolt and ExtraBolt"
        );
    }
}
