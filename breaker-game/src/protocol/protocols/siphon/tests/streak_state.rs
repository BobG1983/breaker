//! Group B — `SiphonStreak` default + `Copy`/`Clone` shape (Behaviors 7–8).
//!
//! Pins the zero-initialised default and the derived `Copy`/`Clone`/`PartialEq`
//! shape so behavioral tests can assert whole-struct equality.

use super::super::system::SiphonStreak;

// ── Behavior 7 — default is zero-initialized ────────────────────────────────

#[test]
fn siphon_streak_default_is_zero_initialized() {
    let s = SiphonStreak::default();
    assert!(
        (s.window_remaining - 0.0).abs() < f32::EPSILON,
        "window_remaining should be 0.0, got {}",
        s.window_remaining
    );
    assert_eq!(s.kill_count, 0, "kill_count should be 0");

    // Edge case: PartialEq derive holds — two defaults compare equal.
    assert_eq!(
        SiphonStreak::default(),
        SiphonStreak::default(),
        "two SiphonStreak::default() instances must be PartialEq-equal"
    );
}

// ── Behavior 8 — Copy + Clone semantics ─────────────────────────────────────

#[test]
fn siphon_streak_is_copy_and_clone() {
    let a = SiphonStreak {
        window_remaining: 1.5,
        kill_count:       3,
    };

    // Copy semantics: assignment through `let b = a;` leaves `a` usable.
    let b = a;
    let c = a;

    // All three compare equal via PartialEq.
    assert_eq!(a, b, "copy `b` must equal original `a`");
    assert_eq!(a, c, "clone `c` must equal original `a`");
    assert_eq!(b, c, "copy and clone must equal each other");

    // Edge case: `a` remains usable after assignments — proves Copy (the
    // assert!/assert_eq! calls below still reference `a`).
    assert!(
        (a.window_remaining - 1.5).abs() < f32::EPSILON,
        "original `a` must remain usable after copy; window_remaining got {}",
        a.window_remaining
    );
    assert_eq!(
        a.kill_count, 3,
        "original `a.kill_count` must remain 3 after copy"
    );
}
