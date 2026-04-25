//! Group E — `debt_collector_on_bolt_lost` (Behaviors 24–27).
//!
//! Pins the `BoltLost` consumer:
//! - Resets `DebtStack` to 0 AND removes `DebtCashOut`.
//! - Tolerates untracked bolts (no panic).
//! - Multi-bolt isolation.
//! - No-op when no `DebtStack` / `DebtCashOut` exist on the lost bolt.

use super::{
    super::system::{DebtCashOut, DebtStack},
    helpers::{
        build_debt_collector_app, install_debt_cash_out, seed_active_protocols_with_debt_collector,
        spawn_bolt_with_stack, write_bolt_lost,
    },
};
use crate::prelude::*;

// ── Behavior 24 — BoltLost resets DebtStack and removes DebtCashOut ────────-

#[test]
fn bolt_lost_resets_debt_stack_and_removes_debt_cash_out() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 2.0);
    install_debt_cash_out(&mut app, bolt, 3.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let stack = app.world().get::<DebtStack>(bolt).expect("stack retained");
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "DebtStack must reset to 0.0 after BoltLost, got {}",
        stack.0
    );
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "DebtCashOut must be removed after BoltLost"
    );
}

// ── Behavior 24 (edge case) — bolt with only DebtStack (no cashout) ────────-

#[test]
fn bolt_lost_with_only_debt_stack_idempotent_removes() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 1.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let stack = app.world().get::<DebtStack>(bolt).expect("stack retained");
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "DebtStack reset to 0.0 even when no DebtCashOut existed, got {}",
        stack.0
    );
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "DebtCashOut must remain absent (idempotent remove)"
    );
}

// ── Behavior 25 — BoltLost for an untracked bolt is tolerated ──────────────-

#[test]
fn bolt_lost_for_untracked_bolt_is_tolerated() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = app.world_mut().spawn(Bolt).id();

    write_bolt_lost(&mut app, bolt);
    tick(&mut app); // must not panic

    // attach_stack fires in this harness (protocol is active), so DebtStack may
    // be present with value 0.0 — we tolerate that. The critical invariant is
    // that on_bolt_lost doesn't panic and doesn't insert DebtCashOut on an
    // untracked bolt.
    if let Some(stack) = app.world().get::<DebtStack>(bolt) {
        assert!(
            (stack.0 - 0.0).abs() < f32::EPSILON,
            "if DebtStack was attached by attach_stack, it must be 0.0 (fresh default); got {}",
            stack.0
        );
    }
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "untracked bolt should still have no DebtCashOut"
    );
}

// ── Behavior 26 — BoltLost only affects the lost bolt ──────────────────────-

#[test]
fn bolt_lost_only_affects_lost_bolt() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let a = spawn_bolt_with_stack(&mut app, 1.0);
    install_debt_cash_out(&mut app, a, 0.5);
    let b = spawn_bolt_with_stack(&mut app, 2.0);
    install_debt_cash_out(&mut app, b, 1.5);

    write_bolt_lost(&mut app, a);
    tick(&mut app);

    // Bolt A: reset / removed.
    let stack_a = app.world().get::<DebtStack>(a).expect("stack retained");
    assert!((stack_a.0 - 0.0).abs() < f32::EPSILON);
    assert!(app.world().get::<DebtCashOut>(a).is_none());

    // Bolt B: untouched.
    let stack_b = app.world().get::<DebtStack>(b).expect("stack retained");
    assert!(
        (stack_b.0 - 2.0).abs() < f32::EPSILON,
        "bolt B's DebtStack must remain 2.0, got {}",
        stack_b.0
    );
    let cashout_b = app.world().get::<DebtCashOut>(b).expect("cashout retained");
    assert!(
        (cashout_b.0 - 1.5).abs() < f32::EPSILON,
        "bolt B's DebtCashOut must remain 1.5, got {}",
        cashout_b.0
    );
}

// ── Behavior 27 — BoltLost on a bolt with no DebtStack is a no-op ──────────-

#[test]
fn bolt_lost_on_bolt_without_debt_stack_is_no_op() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = app.world_mut().spawn(Bolt).id();

    write_bolt_lost(&mut app, bolt);
    tick(&mut app); // must not panic

    // Under the unconditional `attach_stack` system this bolt may acquire
    // DebtStack(0.0) in the same tick. Tolerated either way — the critical
    // assertion is that BoltLost does not insert a DebtStack itself beyond
    // value 0.0 and does not insert a DebtCashOut.
    if let Some(stack) = app.world().get::<DebtStack>(bolt) {
        assert!(
            (stack.0 - 0.0).abs() < f32::EPSILON,
            "if DebtStack present, it must be 0.0; got {}",
            stack.0
        );
    }
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "untracked bolt should never acquire a DebtCashOut from BoltLost"
    );
}
