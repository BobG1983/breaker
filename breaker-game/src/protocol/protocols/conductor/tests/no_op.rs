//! Group N — no-op / skip conditions (Behaviors 6–11).
//!
//! Pins that the swap does NOT fire when:
//! - Grade is Early or Late (only Perfect triggers a swap).
//! - The bumped bolt is already the primary.
//! - `msg.bolt == None` (spectator bump).
//! - No `PrimaryBolt` exists (zero or multiple primaries).
//! - The bolt entity was despawned before the message is processed.
//! - The target entity lacks the `Bolt` marker component.

use bevy::prelude::*;

use super::helpers::{
    bound_fingerprints, build_conductor_app, has_extra, has_primary, make_distinct_bound,
    seed_active_protocols_with_conductor, spawn_dummy_breaker, spawn_extra_bolt_with_bound,
    spawn_primary_bolt, spawn_primary_bolt_with_bound, write_bump_performed,
};
use crate::{bolt::components::PrimaryBolt, breaker::messages::BumpGrade, prelude::*};

// ── Behavior 6 — non-Perfect grade (Early) does not swap ────────────────────

#[test]
fn early_grade_bump_does_not_swap() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Early);
    tick(&mut app);

    assert!(has_primary(&app, primary), "primary marker unchanged");
    assert!(
        !has_primary(&app, extra),
        "extra must not gain PrimaryBolt on Early grade"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()],
        "primary's BoundEffects unchanged"
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()],
        "extra's BoundEffects unchanged"
    );
}

// ── Behavior 6 (edge case) — non-Perfect grade (Late) does not swap ─────────

#[test]
fn late_grade_bump_does_not_swap() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Late);
    tick(&mut app);

    assert!(has_primary(&app, primary), "primary marker unchanged");
    assert!(
        !has_primary(&app, extra),
        "extra must not gain PrimaryBolt on Late grade"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()],
        "primary's BoundEffects unchanged"
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()],
        "extra's BoundEffects unchanged"
    );
}

// ── Behavior 7 — Perfect bump on the already-primary bolt is a no-op ────────

#[test]
fn perfect_bump_on_existing_primary_is_a_no_op() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    write_bump_performed(&mut app, breaker, Some(primary), BumpGrade::Perfect);
    tick(&mut app);

    assert!(has_primary(&app, primary), "primary unchanged");
    assert!(
        !has_extra(&app, primary),
        "primary must NOT gain ExtraBolt on self-bump"
    );
    assert!(!has_primary(&app, extra), "extra still not primary");
    assert!(has_extra(&app, extra), "extra still extra");
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()],
        "primary's BoundEffects unchanged"
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()],
        "extra's BoundEffects unchanged"
    );
}

// ── Behavior 7 (edge case) — single primary bolt, no extras ─────────────────

#[test]
fn perfect_bump_on_sole_primary_bolt_does_not_panic() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt(&mut app);

    write_bump_performed(&mut app, breaker, Some(primary), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        has_primary(&app, primary),
        "sole primary retains PrimaryBolt — no panic, no change"
    );
    assert!(
        !has_extra(&app, primary),
        "sole primary must not gain ExtraBolt"
    );
}

// ── Behavior 8 — Perfect bump with bolt: None is a no-op (spectator bump) ───

#[test]
fn perfect_bump_with_none_bolt_is_a_no_op() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    write_bump_performed(&mut app, breaker, None, BumpGrade::Perfect);
    tick(&mut app);

    assert!(has_primary(&app, primary), "primary unchanged");
    assert!(!has_primary(&app, extra), "extra unchanged");
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()],
        "primary's BoundEffects unchanged"
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()],
        "extra's BoundEffects unchanged"
    );
}

// ── Behavior 8 (edge case) — None bolt with zero bolts in the world ─────────

#[test]
fn perfect_bump_with_none_bolt_and_zero_bolts_does_not_panic() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);

    write_bump_performed(&mut app, breaker, None, BumpGrade::Perfect);
    tick(&mut app); // must not panic
}

// ── Behavior 9 — Perfect bump with NO primary bolt is a no-op ───────────────

#[test]
fn perfect_bump_with_no_primary_bolt_is_a_no_op() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        !has_primary(&app, extra),
        "no swap happened — extra did not auto-promote"
    );
    assert!(has_extra(&app, extra), "extra marker retained");
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()],
        "extra's BoundEffects unchanged"
    );
}

// ── Behavior 9 (edge case) — two primary bolts (invariant violation tolerated)

#[test]
fn perfect_bump_with_two_primary_bolts_is_a_no_op() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    // Two bolts both carrying PrimaryBolt.
    let p1 = app.world_mut().spawn((Bolt, PrimaryBolt)).id();
    let p2 = app.world_mut().spawn((Bolt, PrimaryBolt)).id();

    write_bump_performed(&mut app, breaker, Some(p1), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        has_primary(&app, p1),
        "p1 still PrimaryBolt — no modification"
    );
    assert!(
        has_primary(&app, p2),
        "p2 still PrimaryBolt — no modification"
    );
}

// ── Behavior 10 — Perfect bump on a despawned bolt is tolerated ─────────────

#[test]
fn perfect_bump_on_despawned_extra_bolt_does_not_panic_and_does_not_swap() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    app.world_mut().despawn(extra);

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert!(has_primary(&app, primary), "old primary unchanged");
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()],
        "primary's BoundEffects unchanged"
    );
}

// ── Behavior 10 (edge case) — despawned primary, Perfect bump on extra ──────

#[test]
fn perfect_bump_when_primary_was_despawned_does_not_panic_and_does_not_swap() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    app.world_mut().despawn(primary);

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        has_extra(&app, extra),
        "extra still carries ExtraBolt — no swap"
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()],
        "extra's BoundEffects unchanged — swap was skipped"
    );
}

// ── Behavior 11 — Perfect bump on an entity lacking `ExtraBolt` is a no-op ──
//
// The `extras.contains(bumped)` guard fires first; a `spawn_empty()` entity
// lacks `ExtraBolt` (and `Bolt`) so it is rejected before any component lookup.

#[test]
fn perfect_bump_on_non_bolt_entity_does_not_swap() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));
    // A non-bolt entity.
    let not_a_bolt = app.world_mut().spawn_empty().id();

    write_bump_performed(&mut app, breaker, Some(not_a_bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert!(has_primary(&app, primary), "primary unchanged");
    assert!(
        !has_primary(&app, extra),
        "extra did not gain primary — non-bolt target skipped"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()],
        "primary's BoundEffects unchanged"
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()],
        "extra's BoundEffects unchanged"
    );
}

// ── Regression — Perfect bump on bare Bolt (no ExtraBolt) must not promote ──
//
// Pins the module-doc precondition that only `ExtraBolt` entities are eligible
// for promotion to `PrimaryBolt` on a Perfect bump. The bug: the swap system
// only checks `With<Bolt>` + `bumped != current_primary`, so a bare `(Bolt,)`
// entity is erroneously promoted and the old primary is demoted.

#[test]
fn perfect_bump_on_bare_bolt_without_extra_marker_does_not_swap() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("P"));
    // Bare `(Bolt,)` — no ExtraBolt, no PrimaryBolt, no BoundEffects.
    let bare = app.world_mut().spawn(Bolt).id();

    write_bump_performed(&mut app, breaker, Some(bare), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        has_primary(&app, primary),
        "P still carries PrimaryBolt — bare bolt is not a valid swap target"
    );
    assert!(
        !has_primary(&app, bare),
        "bare bolt must NOT be promoted to PrimaryBolt"
    );
    assert!(
        !has_extra(&app, bare),
        "bare bolt must NOT be demoted to ExtraBolt (it was never Extra)"
    );
    // Edge case: P's BoundEffects unchanged even though bare lacks BoundEffects.
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["P".to_string()],
        "P's BoundEffects fingerprint must be unchanged — no spurious swap"
    );
}
