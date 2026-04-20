//! Group B — `ConductorConfig` component/resource shape (Behavior 24).
//!
//! Pins that `ConductorConfig` derives `Resource + Debug + Clone + Copy +
//! PartialEq`. Mirror of `reckless_dash::tests::components::
//! reckless_dash_config_is_copy_clone_partial_eq`.

use super::super::system::ConductorConfig;

// ── Behavior 24 — ConductorConfig is Copy + Clone + PartialEq ───────────────

#[test]
fn conductor_config_is_copy_clone_partial_eq() {
    fn takes_by_value(cfg: ConductorConfig) -> f32 {
        cfg.primary_swap_window
    }
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    let orig = ConductorConfig {
        primary_swap_window: 0.2,
    };

    // PartialEq: two identical values compare equal.
    let same = ConductorConfig {
        primary_swap_window: 0.2,
    };
    assert_eq!(orig, same, "identical configs must compare equal");

    // PartialEq: different configs compare not-equal.
    let different = ConductorConfig {
        primary_swap_window: 0.5,
    };
    assert_ne!(orig, different, "different configs must compare not-equal");

    // Copy: two pass-by-value assignments both succeed (original not moved).
    let copy1 = orig;
    let copy2 = orig;
    assert_eq!(orig, copy1, "copy must equal original");
    assert_eq!(copy1, copy2, "both copies must equal each other");

    // Copy: pass-by-value into a function compiles and original remains usable.
    let returned = takes_by_value(orig);
    assert!(
        (returned - 0.2).abs() < f32::EPSILON,
        "takes_by_value returned {returned}, expected 0.2"
    );
    assert!(
        (orig.primary_swap_window - 0.2).abs() < f32::EPSILON,
        "original remains usable after pass-by-value (Copy, not move)"
    );

    // Clone: routed through a generic that requires `T: Clone` — proves the
    // derive exists at compile time without tripping `clippy::clone_on_copy`.
    let cloned = require_clone(&orig);
    assert_eq!(orig, cloned, "Clone-returned copy must equal original");
}
