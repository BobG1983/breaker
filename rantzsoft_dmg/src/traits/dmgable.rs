//! `Dmgable` marker trait.

use bevy::prelude::*;

/// Marker trait for component types that participate in the damage pipeline.
///
/// Zero-method marker. Each `impl Dmgable for T` keys a separate per-`T`
/// pipeline of messages and systems. The `Component` supertrait ensures
/// implementers are attachable to entities.
pub trait Dmgable: Component {}

#[cfg(test)]
mod tests {
    use super::*;

    // Test-local dummy component used to exercise the trait bound.
    #[derive(Component)]
    struct TestMarker;

    impl Dmgable for TestMarker {}

    // ── Behavior 26: Dmgable is a zero-method marker trait ──

    #[test]
    fn marker_trait_impl_compiles() {
        // The impl above binds. Nothing else to assert — this test exists
        // so that a failure to compile is surfaced as a RED-phase signal.
    }

    const fn assert_bound<T: Dmgable>() {}

    #[test]
    fn generic_bound_on_dmgable_is_usable() {
        assert_bound::<TestMarker>();
    }

    // ── Behavior 27: Dmgable can be named through the Component supertrait ──

    const fn uses_component<T: Component>() {}
    const fn call_bound<T: Dmgable>() {
        uses_component::<T>();
    }

    #[test]
    fn dmgable_bound_implies_component_bound() {
        call_bound::<TestMarker>();
    }

    // ── Behavior 27 edge case: Dmgable requires the Component supertrait
    //     (negative compile contract) ──
    //
    // The following single line is a documented negative contract. Leave it
    // commented. Uncommenting must cause the file to fail to compile because
    // `NotAComponent` does not implement `Component`. If it ever compiles,
    // the Component supertrait contract has been broken.
    //
    // struct NotAComponent; impl Dmgable for NotAComponent {} // Uncomment to verify the Component supertrait is enforced — must fail to compile.
}
