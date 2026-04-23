//! Death-request message — sent when an entity should die.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::traits::Dmgable;

/// Death request. Consumed by per-`T` kill handlers that perform any domain-
/// specific death logic before confirming the kill via `Destroyed<T>`.
///
/// The victim entity must remain alive through handler dispatch, trigger
/// evaluation, and death animation.
///
/// The `_marker: PhantomData<T>` field selects the per-`T` queue — each
/// concrete `T: Dmgable` produces its own `Messages<KillYourself<T>>`
/// resource, isolating death requests across entity kinds.
#[derive(Message, Debug)]
pub struct KillYourself<T: Dmgable> {
    /// The entity to kill.
    pub victim:  Entity,
    /// The entity that caused the death (from `KilledBy`), or `None` for
    /// environmental deaths.
    pub killer:  Option<Entity>,
    /// Marker selecting the per-`T` message queue.
    pub _marker: PhantomData<T>,
}

// Manual `Clone` impl — `PhantomData` is always `Clone`; `T` is NOT required
// to be `Clone`. Every data field forwards from `self`.
impl<T: Dmgable> Clone for KillYourself<T> {
    fn clone(&self) -> Self {
        Self {
            victim:  self.victim,
            killer:  self.killer,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test-local dummy `Dmgable` types (no `Clone` derive — proves the manual
    // `Clone` impl on the messages does NOT require `T: Clone`).
    #[derive(Component)]
    struct TestT;
    impl Dmgable for TestT {}

    #[derive(Component)]
    struct NotCloneable;
    impl Dmgable for NotCloneable {}

    // Routed through a generic `T: Clone` bound — proves a `Clone` impl exists
    // for the value type at compile time without tripping
    // `clippy::redundant_clone`.
    #[must_use]
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    // ── Behavior 12: `KillYourself<T>` constructs with `victim` and
    //     `killer: Some(_)` ──

    #[test]
    fn constructs_with_victim_and_some_killer() {
        let msg = KillYourself::<TestT> {
            victim:  Entity::PLACEHOLDER,
            killer:  Some(Entity::PLACEHOLDER),
            _marker: PhantomData,
        };
        assert_eq!(msg.victim, Entity::PLACEHOLDER);
        assert_eq!(msg.killer, Some(Entity::PLACEHOLDER));
    }

    #[test]
    fn constructs_with_self_kill_attribution() {
        // Edge case: victim and killer reference the same Entity (self-kill
        // attribution) — no uniqueness check.
        let msg = KillYourself::<TestT> {
            victim:  Entity::PLACEHOLDER,
            killer:  Some(Entity::PLACEHOLDER),
            _marker: PhantomData,
        };
        assert_eq!(msg.victim, Entity::PLACEHOLDER);
        assert_eq!(msg.killer, Some(Entity::PLACEHOLDER));
    }

    // ── Behavior 13: `KillYourself<T>` constructs with `killer: None`
    //     (environmental kill) ──

    #[test]
    fn constructs_with_none_killer() {
        let msg = KillYourself::<TestT> {
            victim:  Entity::PLACEHOLDER,
            killer:  None,
            _marker: PhantomData,
        };
        assert!(msg.killer.is_none());
    }

    // ── Behavior 14: `KillYourself<T>::clone()` preserves both Entity fields ──

    #[test]
    fn clone_preserves_both_entity_fields() {
        let original = KillYourself::<TestT> {
            victim:  Entity::PLACEHOLDER,
            killer:  Some(Entity::PLACEHOLDER),
            _marker: PhantomData,
        };
        let cloned = original.clone();
        assert_eq!(cloned.victim, original.victim);
        assert_eq!(cloned.killer, original.killer);
    }

    #[test]
    fn clone_preserves_none_killer() {
        // Edge case: cloning with killer: None preserves the None.
        let original = KillYourself::<TestT> {
            victim:  Entity::PLACEHOLDER,
            killer:  None,
            _marker: PhantomData,
        };
        let cloned = original.clone();
        assert!(cloned.killer.is_none());
        assert_eq!(cloned.victim, original.victim);
    }

    // ── Behavior 15: `KillYourself<T>::clone()` does NOT require `T: Clone` ──

    #[test]
    fn clone_does_not_require_t_clone() {
        let original = KillYourself::<NotCloneable> {
            victim:  Entity::PLACEHOLDER,
            killer:  Some(Entity::PLACEHOLDER),
            _marker: PhantomData,
        };
        let cloned = require_clone(&original);
        assert_eq!(cloned.victim, Entity::PLACEHOLDER);
        assert_eq!(cloned.killer, Some(Entity::PLACEHOLDER));
    }

    // ── Behavior 16: `KillYourself<T>` is a Bevy `Message` — `add_message`
    //     registration succeeds ──

    #[test]
    fn add_message_registration_succeeds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<KillYourself<TestT>>();
        assert!(
            app.world()
                .contains_resource::<Messages<KillYourself<TestT>>>()
        );
    }

    // ── Behavior 16 edge: `init_resource::<Messages<KillYourself<T>>>()`
    //     succeeds on a bare `World` ──

    #[test]
    fn init_resource_messages_succeeds() {
        // Edge case: init_resource works directly — confirms the `Message`
        // derive produced a Default `Messages` resource shape.
        let mut world = World::new();
        world.init_resource::<Messages<KillYourself<TestT>>>();
        assert!(world.contains_resource::<Messages<KillYourself<TestT>>>());
    }
}
