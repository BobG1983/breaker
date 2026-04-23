//! Generic heal message — one Bevy message queue per target type `T`.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{HealCap, SourceId, traits::Dmgable};

/// Generic heal-dealt message. One Bevy message queue per target type `T`.
///
/// Senders pick a `cap` per message: `HealCap::Starting` clamps the recipient's
/// current value to its starting value; `HealCap::Max` clamps to the configured
/// maximum (falling back to starting when no maximum is set). The `_marker`
/// selects the per-`T` queue.
#[derive(Message, Debug)]
pub struct HealDealt<T: Dmgable> {
    /// The entity that originated this heal (for attribution / UI).
    pub healer:  Option<Entity>,
    /// The entity receiving the heal.
    pub target:  Entity,
    /// Pre-calculated heal amount. Values `<= 0.0` will be ignored by the applier.
    pub amount:  f32,
    /// Optional origin label for attribution, UI, and stats.
    pub source:  Option<SourceId>,
    /// Ceiling selector — picked per message by the sender.
    pub cap:     HealCap,
    /// Marker selecting the per-`T` message queue.
    pub _marker: PhantomData<T>,
}

// Manual `Clone` impl — `PhantomData` is always `Clone`; `T` is NOT required
// to be `Clone`. Every data field forwards from `self`.
impl<T: Dmgable> Clone for HealDealt<T> {
    fn clone(&self) -> Self {
        Self {
            healer:  self.healer,
            target:  self.target,
            amount:  self.amount,
            source:  self.source.clone(),
            cap:     self.cap,
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

    /// Compares two `f32` values for "equality" without tripping
    /// `clippy::float_cmp`. Replicated per file per the test spec
    /// copy-paste policy.
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

    // Routed through a generic `T: Clone` bound — proves a `Clone` impl exists
    // for the value type at compile time without tripping
    // `clippy::redundant_clone`.
    #[must_use]
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    // ── Behavior 7: `HealDealt<T>` constructs with all fields populated
    //     (cap = Starting) ──

    #[test]
    fn constructs_with_all_fields_populated_starting() {
        let msg = HealDealt::<TestT> {
            healer:  Some(Entity::PLACEHOLDER),
            target:  Entity::PLACEHOLDER,
            amount:  3.5,
            source:  Some(SourceId::from("module:heal")),
            cap:     HealCap::Starting,
            _marker: PhantomData,
        };
        assert_eq!(msg.healer, Some(Entity::PLACEHOLDER));
        assert_eq!(msg.target, Entity::PLACEHOLDER);
        assert_f32_eq(msg.amount, 3.5);
        assert_eq!(msg.source, Some(SourceId::from("module:heal")));
        assert_eq!(msg.cap, HealCap::Starting);
    }

    #[test]
    fn constructs_with_zero_amount_starting() {
        // Edge case: amount == 0.0 constructs cleanly — no constructor validation.
        let msg = HealDealt::<TestT> {
            healer:  Some(Entity::PLACEHOLDER),
            target:  Entity::PLACEHOLDER,
            amount:  0.0,
            source:  None,
            cap:     HealCap::Starting,
            _marker: PhantomData,
        };
        assert_f32_eq(msg.amount, 0.0);
    }

    // ── Behavior 8: `HealDealt<T>` constructs with `HealCap::Max` and
    //     `healer: None`, `source: None` ──

    #[test]
    fn constructs_with_none_healer_and_max_cap() {
        let msg = HealDealt::<TestT> {
            healer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  1.0,
            source:  None,
            cap:     HealCap::Max,
            _marker: PhantomData,
        };
        assert!(msg.healer.is_none());
        assert!(msg.source.is_none());
        assert_eq!(msg.cap, HealCap::Max);
        assert_f32_eq(msg.amount, 1.0);
    }

    #[test]
    fn constructs_with_negative_amount() {
        // Edge case: negative amount constructs cleanly — type does not validate.
        let msg = HealDealt::<TestT> {
            healer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  -1.0,
            source:  None,
            cap:     HealCap::Max,
            _marker: PhantomData,
        };
        assert_f32_eq(msg.amount, -1.0);
    }

    // ── Behavior 9: `HealDealt<T>::clone()` preserves the `cap` field
    //     (both variants) ──

    #[test]
    fn clone_preserves_every_field_starting_cap() {
        let original = HealDealt::<TestT> {
            healer:  Some(Entity::PLACEHOLDER),
            target:  Entity::PLACEHOLDER,
            amount:  5.0,
            source:  Some(SourceId::from("src:heal")),
            cap:     HealCap::Starting,
            _marker: PhantomData,
        };
        let cloned = original.clone();
        assert_eq!(cloned.healer, original.healer);
        assert_eq!(cloned.target, original.target);
        assert_f32_eq(cloned.amount, 5.0);
        assert_eq!(cloned.source, original.source);
        assert_eq!(cloned.cap, HealCap::Starting);
    }

    #[test]
    fn clone_preserves_every_field_max_cap() {
        let original = HealDealt::<TestT> {
            healer:  Some(Entity::PLACEHOLDER),
            target:  Entity::PLACEHOLDER,
            amount:  5.0,
            source:  Some(SourceId::from("src:heal")),
            cap:     HealCap::Max,
            _marker: PhantomData,
        };
        let cloned = original.clone();
        assert_eq!(cloned.healer, original.healer);
        assert_eq!(cloned.target, original.target);
        assert_f32_eq(cloned.amount, 5.0);
        assert_eq!(cloned.source, original.source);
        assert_eq!(cloned.cap, HealCap::Max);
    }

    #[test]
    fn clone_preserves_none_source() {
        // Edge case: cloning with source: None preserves the None.
        let original = HealDealt::<TestT> {
            healer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  2.5,
            source:  None,
            cap:     HealCap::Starting,
            _marker: PhantomData,
        };
        let cloned = original.clone();
        assert_eq!(cloned.healer, original.healer);
        assert_eq!(cloned.target, original.target);
        assert_eq!(cloned.source, original.source);
        assert_eq!(cloned.cap, original.cap);
        assert!(cloned.source.is_none());
        assert_f32_eq(cloned.amount, 2.5);
    }

    // ── Behavior 10: `HealDealt<T>::clone()` does NOT require `T: Clone` ──

    #[test]
    fn clone_does_not_require_t_clone() {
        let original = HealDealt::<NotCloneable> {
            healer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  3.5,
            source:  None,
            cap:     HealCap::Starting,
            _marker: PhantomData,
        };
        let cloned = require_clone(&original);
        assert_f32_eq(cloned.amount, 3.5);
    }

    // ── Behavior 11: `HealDealt<T>` is a Bevy `Message` — `add_message`
    //     registration succeeds ──

    #[test]
    fn add_message_registration_succeeds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<HealDealt<TestT>>();
        assert!(
            app.world()
                .contains_resource::<Messages<HealDealt<TestT>>>()
        );
    }

    #[test]
    fn init_resource_messages_succeeds() {
        // Edge case: init_resource works directly.
        let mut world = World::new();
        world.init_resource::<Messages<HealDealt<TestT>>>();
        assert!(world.contains_resource::<Messages<HealDealt<TestT>>>());
    }
}
