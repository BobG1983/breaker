use crate::RantzDmgPlugin;

// ── Behavior 40: `RantzDmgPlugin` is a unit struct at the crate root ──

#[test]
fn unit_struct_constructs_without_arguments() {
    let _ = RantzDmgPlugin;
    // Edge case: struct-literal form also compiles.
    let _ = RantzDmgPlugin {};
}

// ── Behavior 41: `RantzDmgPlugin` derives `Default` ──

#[test]
fn default_constructor_compiles() {
    // The `<_ as Default>::default()` form below exercises the Default
    // derive without tripping `clippy::default_constructed_unit_structs`
    // (which would fire on `RantzDmgPlugin::default()`).
    let _ = <RantzDmgPlugin as Default>::default();
}

// ── Behavior 42: `RantzDmgPlugin` derives `Debug`, `Clone`, `Copy`,
//     `PartialEq`, `Eq`, `Hash` ──

// Routed through a generic `T: Clone` bound — proves a `Clone` impl
// exists for the value type at compile time without tripping
// `clippy::clone_on_copy`.
#[must_use]
fn require_clone<T: Clone>(value: &T) -> T {
    value.clone()
}

#[test]
fn derives_debug_clone_copy_partial_eq_eq_hash() {
    use std::collections::HashSet;

    let p = RantzDmgPlugin;

    // Debug — non-empty, contains the type name.
    let formatted = format!("{p:?}");
    assert!(!formatted.is_empty());
    assert!(
        formatted.contains("RantzDmgPlugin"),
        "expected debug output to contain RantzDmgPlugin, got {formatted:?}"
    );

    // Clone — returns an equal value.
    let cloned = require_clone(&p);
    assert_eq!(cloned, p);

    // Copy — double-use without move.
    let a = p;
    let b = p;
    assert_eq!(a, b);

    // PartialEq + Eq.
    assert_eq!(p, RantzDmgPlugin);

    // Hash.
    let mut set: HashSet<RantzDmgPlugin> = HashSet::new();
    set.insert(p);
    assert_eq!(set.len(), 1);

    // Edge case: clone compares equal.
    assert_eq!(p, require_clone(&p));
}

// ── Behavior 43: `RantzDmgPlugin` is `pub` at the crate root
//     (re-exported from `lib.rs`) ──

#[test]
fn rantz_dmg_plugin_is_pub_at_crate_root() {
    // Resolve RantzDmgPlugin through the crate-root re-export path. If
    // `lib.rs` did not `pub use plugin::RantzDmgPlugin;`, this path
    // would fail with E0603.
    let _ = crate::RantzDmgPlugin;
    let _ = <crate::RantzDmgPlugin as Default>::default();
}

#[test]
fn rantz_dmg_plugin_resolves_via_crate_glob_import() {
    // Edge case: `use crate::*;` glob import also resolves
    // RantzDmgPlugin.
    use crate::*;

    let _ = RantzDmgPlugin;
}
