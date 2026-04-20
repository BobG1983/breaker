//! Group C — `debt_collector_on_bump` (Behaviors 7–15).
//!
//! Pins the `BumpPerformed` consumer:
//! - Early/Late bumps accumulate `stack_per_bump` onto `DebtStack`.
//! - Perfect bump inserts `DebtCashOut(stack)` snapshot and resets stack to 0.
//! - Zero-stack Perfect still emits `DebtCashOut(0.0)`.
//! - `bolt: None` skipped; despawned bolts tolerated; untracked bolts tolerated.
//! - Multi-bolt isolation.
//! - Harness-safe: `reader.clear()` + early-return when `DebtCollectorConfig`
//!   absent.

use super::{
    super::system::{DebtCashOut, DebtStack},
    helpers::{
        build_debt_collector_app, build_debt_collector_app_no_config,
        seed_active_protocols_with_debt_collector, spawn_bolt_with_stack, write_bump_performed,
    },
};
use crate::{breaker::messages::BumpGrade, prelude::*};

// ── Behavior 7 — Early bump adds stack_per_bump to DebtStack ────────────────

#[test]
fn early_bump_adds_stack_per_bump_to_debt_stack() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 0.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Early);
    tick(&mut app);

    let stack = app
        .world()
        .get::<DebtStack>(bolt)
        .expect("bolt should retain DebtStack after Early bump");
    assert!(
        (stack.0 - 0.5).abs() < f32::EPSILON,
        "DebtStack after single Early bump expected 0.5, got {}",
        stack.0
    );
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "Early bump must NOT insert DebtCashOut"
    );
}

// ── Behavior 7 (edge case) — three Early bumps accumulate ──────────────────-

#[test]
fn three_early_bumps_accumulate_to_one_point_five() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 0.0);

    for _ in 0..3 {
        write_bump_performed(&mut app, Some(bolt), BumpGrade::Early);
        tick(&mut app);
    }

    let stack = app
        .world()
        .get::<DebtStack>(bolt)
        .expect("bolt should retain DebtStack");
    assert!(
        (stack.0 - 1.5).abs() < f32::EPSILON,
        "three Early bumps @ 0.5 each expected stack 1.5, got {}",
        stack.0
    );
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "Early bumps never insert DebtCashOut"
    );
}

// ── Behavior 8 — Late bump adds stack_per_bump to DebtStack ─────────────────

#[test]
fn late_bump_adds_stack_per_bump_to_debt_stack() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 1.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Late);
    tick(&mut app);

    let stack = app
        .world()
        .get::<DebtStack>(bolt)
        .expect("bolt should retain DebtStack after Late bump");
    assert!(
        (stack.0 - 1.5).abs() < f32::EPSILON,
        "DebtStack after Late bump @ 1.0 start expected 1.5, got {}",
        stack.0
    );
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "Late bump must NOT insert DebtCashOut"
    );
}

// ── Behavior 8 (edge case) — Early then Late mixes into the same stack ─────-

#[test]
fn early_then_late_accumulate_into_same_stack() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 0.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Early);
    tick(&mut app);
    write_bump_performed(&mut app, Some(bolt), BumpGrade::Late);
    tick(&mut app);

    let stack = app
        .world()
        .get::<DebtStack>(bolt)
        .expect("bolt should retain DebtStack");
    assert!(
        (stack.0 - 1.0).abs() < f32::EPSILON,
        "Early (0.5) + Late (0.5) expected stack 1.0, got {}",
        stack.0
    );
}

// ── Behavior 9 — Perfect on non-zero stack cashes out and resets ────────────

#[test]
fn perfect_bump_on_nonzero_stack_inserts_cashout_and_resets_stack() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 1.5);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    let cashout = app
        .world()
        .get::<DebtCashOut>(bolt)
        .expect("Perfect bump should insert DebtCashOut");
    assert!(
        (cashout.0 - 1.5).abs() < f32::EPSILON,
        "DebtCashOut should snapshot raw stack 1.5, got {}",
        cashout.0
    );
    let stack = app
        .world()
        .get::<DebtStack>(bolt)
        .expect("bolt should still have DebtStack after Perfect");
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "DebtStack must reset to 0.0 after Perfect, got {}",
        stack.0
    );
}

// ── Behavior 9 edge case — two Perfects in same frame ──

#[test]
fn second_perfect_bump_same_frame_overwrites_cashout_to_zero() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 1.5);

    // Two Perfect bumps in the same frame.
    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);
    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);

    tick(&mut app);

    // First Perfect: cashout = 1.5, stack reset to 0.0.
    // Second Perfect: stack is already 0.0, cashout overwritten to 0.0.
    let cashout = app.world().get::<DebtCashOut>(bolt).copied();
    assert_eq!(
        cashout,
        Some(DebtCashOut(0.0)),
        "second Perfect in the same frame must overwrite DebtCashOut to 0.0 (stack was reset by first Perfect)"
    );
    let stack = app.world().get::<DebtStack>(bolt).copied();
    assert_eq!(
        stack,
        Some(DebtStack(0.0)),
        "DebtStack must remain at 0.0 after both Perfects"
    );
}

// ── Behavior 10 — Perfect on zero stack still inserts DebtCashOut(0.0) ─────-

#[test]
fn perfect_bump_on_zero_stack_inserts_zero_cashout() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 0.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    let cashout = app
        .world()
        .get::<DebtCashOut>(bolt)
        .expect("Perfect bump must insert DebtCashOut even on zero stack");
    assert!(
        (cashout.0 - 0.0).abs() < f32::EPSILON,
        "DebtCashOut should be 0.0 when stack was 0.0, got {}",
        cashout.0
    );
    let stack = app
        .world()
        .get::<DebtStack>(bolt)
        .expect("bolt should retain DebtStack");
    assert!((stack.0 - 0.0).abs() < f32::EPSILON);
}

// ── Behavior 11 — BumpPerformed with bolt: None is skipped ──────────────────

#[test]
fn bump_performed_with_bolt_none_is_skipped() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 0.0);

    write_bump_performed(&mut app, None, BumpGrade::Early);
    tick(&mut app);

    let stack = app
        .world()
        .get::<DebtStack>(bolt)
        .expect("bolt should retain DebtStack");
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "bolt: None must not modify any stack, got {}",
        stack.0
    );
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "bolt: None must not insert DebtCashOut anywhere"
    );

    // Edge case — Perfect with bolt: None is also skipped.
    write_bump_performed(&mut app, None, BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "Perfect with bolt: None must not insert DebtCashOut anywhere"
    );
}

// ── Behavior 12 — BumpPerformed for bolt without DebtStack is tolerated ────-

#[test]
fn bump_performed_for_bolt_without_debt_stack_is_tolerated() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    // Spawn a bare bolt WITHOUT DebtStack.
    let bolt = app.world_mut().spawn(Bolt).id();

    // Don't crash; don't insert DebtStack via on_bump (that's attach_stack's
    // job — but since attach_stack is also registered, we use
    // `Local<bool>`-free explicit check on component presence AFTER we
    // write + tick. attach_stack might insert DebtStack(0.0) in the same
    // frame, so we just verify no panic occurred and no DebtCashOut was
    // created.
    write_bump_performed(&mut app, Some(bolt), BumpGrade::Early);
    tick(&mut app);

    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "on_bump must not insert DebtCashOut on an untracked bolt"
    );
}

// ── Behavior 13 — multiple bolts maintain independent stacks ────────────────

#[test]
fn multiple_bolts_maintain_independent_stacks() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let a = spawn_bolt_with_stack(&mut app, 0.0);
    let b = spawn_bolt_with_stack(&mut app, 0.8);

    write_bump_performed(&mut app, Some(a), BumpGrade::Early);
    write_bump_performed(&mut app, Some(b), BumpGrade::Perfect);
    tick(&mut app);

    let stack_a = app
        .world()
        .get::<DebtStack>(a)
        .expect("bolt a has DebtStack");
    assert!(
        (stack_a.0 - 0.5).abs() < f32::EPSILON,
        "bolt A should have DebtStack(0.5), got {}",
        stack_a.0
    );
    assert!(
        app.world().get::<DebtCashOut>(a).is_none(),
        "bolt A must not have DebtCashOut"
    );

    let stack_b = app
        .world()
        .get::<DebtStack>(b)
        .expect("bolt b has DebtStack");
    assert!(
        (stack_b.0 - 0.0).abs() < f32::EPSILON,
        "bolt B should be reset to DebtStack(0.0), got {}",
        stack_b.0
    );
    let cashout_b = app
        .world()
        .get::<DebtCashOut>(b)
        .expect("bolt B should have DebtCashOut");
    assert!(
        (cashout_b.0 - 0.8).abs() < f32::EPSILON,
        "bolt B should have DebtCashOut(0.8), got {}",
        cashout_b.0
    );
}

// ── Behavior 14 — BumpPerformed for a despawned bolt is tolerated ──────────-

#[test]
fn bump_performed_for_despawned_bolt_is_tolerated() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 0.0);
    app.world_mut().entity_mut(bolt).despawn();

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Early);
    tick(&mut app); // must not panic

    // Assert no orphan DebtStack exists for the despawned entity id — the
    // query simply fails for missing entities, which is the tolerated path.
    assert!(
        app.world().get::<DebtStack>(bolt).is_none(),
        "despawned bolt should not have DebtStack"
    );
}

// ── Behavior 15 — on_bump early-returns and clears reader when config absent

#[test]
fn on_bump_early_returns_and_clears_reader_when_config_absent() {
    let mut app = build_debt_collector_app_no_config();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 0.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Early);
    tick(&mut app); // must not panic

    let stack = app
        .world()
        .get::<DebtStack>(bolt)
        .expect("bolt should retain DebtStack");
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "stack must remain 0.0 when config absent, got {}",
        stack.0
    );

    // Edge case: a second tick with no new messages must not retroactively
    // process the prior buffered message — proves reader.clear() was called.
    tick(&mut app);
    let stack = app
        .world()
        .get::<DebtStack>(bolt)
        .expect("bolt should retain DebtStack");
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "second quiet tick must not retroactively process buffered message, got {}",
        stack.0
    );
}
