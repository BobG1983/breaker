//! Group B — `ConductorConfig` resource shape (Behavior 24).
//!
//! Pins that `ConductorConfig` derives `Resource + Debug + Clone + Copy +
//! PartialEq + Eq`. The config is a unit-struct presence marker — no fields
//! to compare, but the derive set is load-bearing for use as a resource gate.

use super::super::system::ConductorConfig;

// ── Behavior 24 — ConductorConfig is Copy + Clone + PartialEq ───────────────

#[test]
fn conductor_config_is_copy_clone_partial_eq() {
    fn takes_by_value(_cfg: ConductorConfig) {}
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    let orig = ConductorConfig;

    // PartialEq: two identical values compare equal.
    let same = ConductorConfig;
    assert_eq!(orig, same, "identical configs must compare equal");

    // Copy: two pass-by-value assignments both succeed (original not moved).
    let copy1 = orig;
    let copy2 = orig;
    assert_eq!(orig, copy1, "copy must equal original");
    assert_eq!(copy1, copy2, "both copies must equal each other");

    // Copy: pass-by-value into a function compiles and original remains usable.
    takes_by_value(orig);
    let still_usable = orig;
    assert_eq!(
        still_usable, orig,
        "orig must still be usable after Copy move"
    );

    // Clone: routed through a generic that requires `T: Clone` — proves the
    // derive exists at compile time without tripping `clippy::clone_on_copy`.
    let cloned = require_clone(&orig);
    assert_eq!(orig, cloned, "Clone-returned copy must equal original");
}
