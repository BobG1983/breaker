//! Transition registry — maps `TypeId` to lifecycle closures.

use std::{any::TypeId, collections::HashMap};

use bevy::prelude::*;

use super::{
    resources::{EndingTransition, RunningTransition, StartingTransition},
    traits::Transition,
};

/// Closure that operates on the World to insert/remove marker resources.
type WorldFn = Box<dyn Fn(&mut World) + Send + Sync>;

/// A single registry entry storing closures for all four lifecycle operations.
struct TransitionEntry {
    /// Inserts `StartingTransition<T>`.
    insert_starting: WorldFn,
    /// Removes `StartingTransition<T>`, inserts `RunningTransition<T>`.
    advance_to_running: WorldFn,
    /// Removes `RunningTransition<T>`, inserts `EndingTransition<T>`.
    advance_to_ending: WorldFn,
    /// Removes `EndingTransition<T>`.
    remove_ending: WorldFn,
}

/// Registry mapping concrete transition effect `TypeId`s to their lifecycle
/// closures.
///
/// When a route fires with a transition, the orchestrator looks up the
/// effect's `TypeId` in this registry and calls the appropriate closure to
/// manage the `StartingTransition<T>` / `RunningTransition<T>` /
/// `EndingTransition<T>` marker resources.
#[derive(Resource, Default)]
pub struct TransitionRegistry {
    entries: HashMap<TypeId, TransitionEntry>,
}

impl TransitionRegistry {
    /// Register a concrete transition effect type.
    ///
    /// Creates closures for all four lifecycle operations:
    /// `insert_starting`, `advance_to_running`, `advance_to_ending`,
    /// `remove_ending`. Idempotent — registering the same type twice is a
    /// no-op.
    pub fn register<T: Transition>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.entries
            .entry(type_id)
            .or_insert_with(|| TransitionEntry {
                insert_starting: Box::new(|world: &mut World| {
                    world.insert_resource(StartingTransition::<T>::new());
                }),
                advance_to_running: Box::new(|world: &mut World| {
                    world.remove_resource::<StartingTransition<T>>();
                    world.insert_resource(RunningTransition::<T>::new());
                }),
                advance_to_ending: Box::new(|world: &mut World| {
                    world.remove_resource::<RunningTransition<T>>();
                    world.insert_resource(EndingTransition::<T>::new());
                }),
                remove_ending: Box::new(|world: &mut World| {
                    world.remove_resource::<EndingTransition<T>>();
                }),
            });
    }

    /// Returns `true` if the given type has been registered.
    #[must_use]
    pub fn contains<T: Transition>(&self) -> bool {
        self.entries.contains_key(&TypeId::of::<T>())
    }

    /// Trigger the start of a transition by inserting `StartingTransition<T>`
    /// for the given `TypeId`.
    ///
    /// Returns `true` if the type was found and the resource was inserted,
    /// `false` if the `TypeId` is not registered.
    pub fn start_transition(&self, type_id: TypeId, world: &mut World) -> bool {
        self.entries.get(&type_id).is_some_and(|entry| {
            (entry.insert_starting)(world);
            true
        })
    }

    /// Advance from Starting to Running phase for the given `TypeId`.
    ///
    /// Removes `StartingTransition<T>` and inserts `RunningTransition<T>`.
    /// Returns `true` if found and invoked, `false` if not registered.
    pub(crate) fn advance_to_running(&self, type_id: TypeId, world: &mut World) -> bool {
        self.entries.get(&type_id).is_some_and(|entry| {
            (entry.advance_to_running)(world);
            true
        })
    }

    /// Advance from Running to Ending phase for the given `TypeId`.
    ///
    /// Removes `RunningTransition<T>` and inserts `EndingTransition<T>`.
    /// Returns `true` if found and invoked, `false` if not registered.
    pub(crate) fn advance_to_ending(&self, type_id: TypeId, world: &mut World) -> bool {
        self.entries.get(&type_id).is_some_and(|entry| {
            (entry.advance_to_ending)(world);
            true
        })
    }

    /// Remove the ending marker resource for the given `TypeId`.
    ///
    /// Returns `true` if found and invoked, `false` if not registered.
    pub(crate) fn remove_ending(&self, type_id: TypeId, world: &mut World) -> bool {
        self.entries.get(&type_id).is_some_and(|entry| {
            (entry.remove_ending)(world);
            true
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transition::traits::{InTransition, OneShotTransition, OutTransition, Transition};

    struct TestEffectOut;
    impl Transition for TestEffectOut {}
    impl OutTransition for TestEffectOut {}

    struct TestEffectIn;
    impl Transition for TestEffectIn {}
    impl InTransition for TestEffectIn {}

    struct TestEffectOneShot;
    impl Transition for TestEffectOneShot {}
    impl OneShotTransition for TestEffectOneShot {}

    // --- Section B behavior 1: Register and look up by TypeId ---

    #[test]
    fn register_and_lookup_by_type_id() {
        let mut registry = TransitionRegistry::default();
        registry.register::<TestEffectOut>();
        assert!(registry.contains::<TestEffectOut>());
    }

    #[test]
    fn unregistered_type_not_found() {
        let mut registry = TransitionRegistry::default();
        registry.register::<TestEffectOut>();
        assert!(!registry.contains::<TestEffectIn>());
    }

    // --- Section B behavior 2: Register multiple distinct types ---

    #[test]
    fn register_multiple_distinct_types() {
        let mut registry = TransitionRegistry::default();
        registry.register::<TestEffectOut>();
        registry.register::<TestEffectIn>();
        registry.register::<TestEffectOneShot>();
        assert!(registry.contains::<TestEffectOut>());
        assert!(registry.contains::<TestEffectIn>());
        assert!(registry.contains::<TestEffectOneShot>());
    }

    #[test]
    fn registering_same_type_twice_is_idempotent() {
        let mut registry = TransitionRegistry::default();
        registry.register::<TestEffectOut>();
        registry.register::<TestEffectOut>(); // no panic
        assert!(registry.contains::<TestEffectOut>());
    }

    // --- Section B behavior 3: start_transition inserts StartingTransition<T> ---

    #[test]
    fn start_transition_inserts_starting_transition_resource() {
        let mut registry = TransitionRegistry::default();
        registry.register::<TestEffectOut>();

        let mut world = World::new();
        let result = registry.start_transition(TypeId::of::<TestEffectOut>(), &mut world);

        assert!(result);
        assert!(
            world.contains_resource::<StartingTransition<TestEffectOut>>(),
            "StartingTransition<TestEffectOut> should be inserted by start_transition"
        );
    }

    #[test]
    fn start_transition_returns_false_for_unregistered_type() {
        let registry = TransitionRegistry::default();
        let mut world = World::new();
        let result = registry.start_transition(TypeId::of::<TestEffectOut>(), &mut world);
        assert!(!result);
    }

    // --- Section B behavior 4: Default registry is empty ---

    #[test]
    fn default_registry_does_not_contain_any_types() {
        let registry = TransitionRegistry::default();
        assert!(!registry.contains::<TestEffectOut>());
    }
}
