//! Group B — `DebtStack` / `DebtCashOut` shape (Behaviors 5–6).
//!
//! Pins the component derive set: `DebtStack` has `Default` returning zero;
//! both components are `Copy` + `Clone` + `PartialEq`; and `DebtCashOut`
//! does NOT implement `Default` (constructed only via explicit value).

use super::super::system::{DebtCashOut, DebtStack};

// ── Behavior 5 — DebtStack::default() is zero ───────────────────────────────

#[test]
fn debt_stack_default_is_zero() {
    let s = DebtStack::default();
    assert!(
        (s.0 - 0.0).abs() < f32::EPSILON,
        "DebtStack::default() expected 0.0, got {}",
        s.0
    );

    // Edge case: two default values compare equal.
    assert_eq!(DebtStack::default(), DebtStack::default());
}

// ── Behavior 6 — DebtStack and DebtCashOut are Copy + Clone + PartialEq ────-

#[test]
fn debt_stack_and_debt_cash_out_are_copy() {
    // DebtStack
    let stack_orig = DebtStack(1.5);
    let stack_copy1 = stack_orig; // Copy
    let stack_copy2 = stack_orig; // still usable after previous assignment
    assert_eq!(stack_orig, stack_copy1);
    assert_eq!(stack_copy1, stack_copy2);
    assert!((stack_orig.0 - 1.5).abs() < f32::EPSILON);

    // DebtCashOut — constructed ONLY via explicit value. Must NOT derive
    // Default. The type-level absence of Default is enforced by writer-code
    // via the derive set; at test-time we simply exercise Copy.
    let cashout_orig = DebtCashOut(2.5);
    let cashout_copy1 = cashout_orig; // Copy
    let cashout_copy2 = cashout_orig; // still usable
    assert_eq!(cashout_orig, cashout_copy1);
    assert_eq!(cashout_copy1, cashout_copy2);
    assert!((cashout_orig.0 - 2.5).abs() < f32::EPSILON);
}
