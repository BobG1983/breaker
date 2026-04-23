//! `Hp` component and its constructor. Fields are `pub`: mutation is by
//! design, since the damage pipeline decrements `current` in place.

use bevy::prelude::*;

/// Unified health component for damageable entities.
///
/// `current` tracks hit points remaining, `starting` tracks the value the
/// entity spawned with (used as the `HealCap::Starting` ceiling), and `max`
/// optionally caps healing (used as the `HealCap::Max` ceiling). The
/// constructor performs no validation — callers at emit sites have the
/// context to reject bad values, so centralising validation here would be
/// both redundant and lossy.
#[derive(Component, Debug, Clone)]
pub struct Hp {
    /// Hit points remaining.
    pub current:  f32,
    /// The HP the entity spawned with.
    pub starting: f32,
    /// Optional upper bound for healing.
    pub max:      Option<f32>,
}

impl Hp {
    /// Creates a new `Hp` with `current` and `starting` set to the given value
    /// and `max` set to `None`.
    #[must_use]
    pub const fn new(starting: f32) -> Self {
        Self {
            current: starting,
            starting,
            max: None,
        }
    }
}

// Behavior 1 edge case: const evaluation of `Hp::new`. This const lives at the
// FILE's module scope (NOT inside `#[cfg(test)]`) so that if `Hp::new` is ever
// changed away from a `const fn`, this line fails at compile time in production
// builds too — not only under `cargo test`. The anonymous `_` binding means the
// compiler evaluates the expression without requiring a name or a use site.
const _: Hp = Hp::new(5.0);

#[cfg(test)]
mod tests {
    use super::*;

    /// Test-scope const that re-pins `const fn` in the test context, so the
    /// `new_is_const_fn_usable_at_compile_time` assertion has a named value to
    /// compare against.
    const HP_CONST: Hp = Hp::new(5.0);

    /// Compares two `f32` values for "equality" without tripping
    /// `clippy::float_cmp`. Handles infinity by checking sign and finiteness
    /// directly (so we never compute `INF - INF`, which produces NaN).
    #[track_caller]
    fn assert_f32_eq(actual: f32, expected: f32) {
        if expected.is_infinite() {
            assert!(
                actual.is_infinite() && actual.is_sign_positive() == expected.is_sign_positive(),
                "expected {expected}, got {actual}"
            );
        } else {
            assert!(
                (actual - expected).abs() < f32::EPSILON,
                "expected {expected}, got {actual}"
            );
        }
    }

    // ── Behavior 1: Hp::new with positive starting value sets all fields ──

    #[test]
    fn new_with_positive_starting_sets_all_fields() {
        let hp = Hp::new(10.0);
        assert_f32_eq(hp.current, 10.0);
        assert_f32_eq(hp.starting, 10.0);
        assert!(hp.max.is_none());
    }

    #[test]
    fn new_with_zero_starting_sets_all_fields() {
        let hp = Hp::new(0.0);
        assert_f32_eq(hp.current, 0.0);
        assert_f32_eq(hp.starting, 0.0);
        assert!(hp.max.is_none());
    }

    #[test]
    fn new_is_const_fn_usable_at_compile_time() {
        // HP_CONST is evaluated at compile time above. Asserting on its value
        // proves the const was materialized.
        assert_f32_eq(HP_CONST.current, 5.0);
        assert_f32_eq(HP_CONST.starting, 5.0);
        assert!(HP_CONST.max.is_none());
    }

    // ── Behavior 2: Hp::new allows a negative starting value ──

    #[test]
    fn new_allows_negative_starting_value() {
        let hp = Hp::new(-5.0);
        assert_f32_eq(hp.current, -5.0);
        assert_f32_eq(hp.starting, -5.0);
        assert!(hp.max.is_none());
    }

    // ── Behavior 3: Hp::new with infinity produces infinite HP (no clamp) ──

    #[test]
    fn new_with_infinity_does_not_clamp() {
        let hp = Hp::new(f32::INFINITY);
        assert_f32_eq(hp.current, f32::INFINITY);
        assert_f32_eq(hp.starting, f32::INFINITY);
        assert!(hp.max.is_none());
    }

    // ── Behavior 4: Hp derives Clone and preserves all fields ──

    // Routed through a generic `T: Clone` bound — proves the derive exists
    // at compile time without tripping `clippy::redundant_clone`.
    #[must_use]
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    #[test]
    fn clone_preserves_all_fields_with_some_max() {
        let hp = Hp {
            current:  7.5,
            starting: 10.0,
            max:      Some(15.0),
        };
        let cloned = require_clone(&hp);
        assert_f32_eq(cloned.current, 7.5);
        assert_f32_eq(cloned.starting, 10.0);
        assert_eq!(cloned.max, Some(15.0));
    }

    #[test]
    fn clone_preserves_none_max() {
        let hp = Hp {
            current:  1.0,
            starting: 2.0,
            max:      None,
        };
        let cloned = require_clone(&hp);
        assert!(cloned.max.is_none());
    }

    // ── Behavior 5: Hp derives Debug — formatter does not panic ──

    #[test]
    fn debug_format_contains_value() {
        let hp = Hp::new(42.0);
        let s = format!("{hp:?}");
        assert!(s.contains("42"));
    }

    #[test]
    fn debug_format_with_some_max_is_non_empty() {
        let hp = Hp {
            current:  5.0,
            starting: 10.0,
            max:      Some(99.0),
        };
        let s = format!("{hp:?}");
        assert!(!s.is_empty());
    }

    // ── Behavior 6: Hp fields are public — direct construction and mutation compile ──

    #[test]
    fn fields_are_publicly_constructible_and_mutable() {
        let mut hp = Hp {
            current:  5.0,
            starting: 5.0,
            max:      None,
        };
        hp.current = 3.0;
        hp.max = Some(10.0);
        assert_f32_eq(hp.current, 3.0);
        assert_eq!(hp.max, Some(10.0));
    }

    // ── Behavior 7: Hp is a Bevy Component — spawnable on an entity ──

    #[test]
    fn spawns_as_component_in_bare_world() {
        let mut world = World::new();
        let e = world.spawn(Hp::new(10.0)).id();
        let Some(hp) = world.get::<Hp>(e) else {
            panic!("Hp component missing on spawned entity");
        };
        assert_f32_eq(hp.current, 10.0);
        assert_f32_eq(hp.starting, 10.0);
        assert!(hp.max.is_none());
    }
}
