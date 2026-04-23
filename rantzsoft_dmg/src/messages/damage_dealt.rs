//! Generic damage message — one Bevy message queue per victim type `T`.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{SourceId, traits::Dmgable};

/// Generic damage-dealt message. One Bevy message queue per victim type `T`.
///
/// Senders populate `dealer` (for kill attribution), `target` (the entity taking
/// the damage), the pre-calculated `amount`, and an optional `source`
/// identifier. The `_marker: PhantomData<T>` selects the per-`T` queue so
/// `DamageDealt<A>` and `DamageDealt<B>` never collide.
#[derive(Message, Debug)]
pub struct DamageDealt<T: Dmgable> {
    /// The entity that originated this damage (for kill attribution).
    pub dealer:  Option<Entity>,
    /// The entity taking the damage.
    pub target:  Entity,
    /// Pre-calculated damage amount (final multipliers already applied by the sender).
    pub amount:  f32,
    /// Optional origin label for attribution, UI, and stats.
    pub source:  Option<SourceId>,
    /// Marker selecting the per-`T` message queue.
    pub _marker: PhantomData<T>,
}

// Manual `Clone` impl — `PhantomData` is always `Clone`; `T` is NOT required
// to be `Clone`. Every data field forwards from `self`.
impl<T: Dmgable> Clone for DamageDealt<T> {
    fn clone(&self) -> Self {
        Self {
            dealer:  self.dealer,
            target:  self.target,
            amount:  self.amount,
            source:  self.source.clone(),
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

    // ── Behavior 1: `DamageDealt<T>` constructs with all fields populated ──

    #[test]
    fn constructs_with_all_fields_populated() {
        let msg = DamageDealt::<TestT> {
            dealer:  Some(Entity::PLACEHOLDER),
            target:  Entity::PLACEHOLDER,
            amount:  10.0,
            source:  Some(SourceId::from("src:alpha")),
            _marker: PhantomData,
        };
        assert_eq!(msg.dealer, Some(Entity::PLACEHOLDER));
        assert_eq!(msg.target, Entity::PLACEHOLDER);
        assert_f32_eq(msg.amount, 10.0);
        assert_eq!(msg.source, Some(SourceId::from("src:alpha")));
    }

    #[test]
    fn constructs_with_zero_amount() {
        // Edge case: amount == 0.0 constructs cleanly — no constructor validation.
        let msg = DamageDealt::<TestT> {
            dealer:  Some(Entity::PLACEHOLDER),
            target:  Entity::PLACEHOLDER,
            amount:  0.0,
            source:  Some(SourceId::from("src:alpha")),
            _marker: PhantomData,
        };
        assert_f32_eq(msg.amount, 0.0);
    }

    // ── Behavior 2: `DamageDealt<T>` constructs with `dealer: None` and
    //     `source: None` (environmental damage path) ──

    #[test]
    fn constructs_with_none_dealer_and_none_source() {
        let msg = DamageDealt::<TestT> {
            dealer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  1.5,
            source:  None,
            _marker: PhantomData,
        };
        assert!(msg.dealer.is_none());
        assert!(msg.source.is_none());
        assert_f32_eq(msg.amount, 1.5);
    }

    #[test]
    fn constructs_with_negative_amount() {
        // Edge case: negative amount constructs cleanly — no sign validation.
        let msg = DamageDealt::<TestT> {
            dealer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  -3.0,
            source:  None,
            _marker: PhantomData,
        };
        assert_f32_eq(msg.amount, -3.0);
    }

    // ── Behavior 3: `DamageDealt<T>::clone()` preserves every field
    //     (manual `Clone` impl) ──

    #[test]
    fn clone_preserves_every_field() {
        let original = DamageDealt::<TestT> {
            dealer:  Some(Entity::PLACEHOLDER),
            target:  Entity::PLACEHOLDER,
            amount:  7.25,
            source:  Some(SourceId::from("module:action")),
            _marker: PhantomData,
        };
        let cloned = original.clone();
        assert_eq!(cloned.dealer, original.dealer);
        assert_eq!(cloned.target, original.target);
        assert_f32_eq(cloned.amount, 7.25);
        assert_eq!(cloned.source, original.source);
    }

    #[test]
    fn clone_preserves_none_source() {
        // Edge case: cloning with source: None preserves None and other fields.
        let original = DamageDealt::<TestT> {
            dealer:  Some(Entity::PLACEHOLDER),
            target:  Entity::PLACEHOLDER,
            amount:  2.5,
            source:  None,
            _marker: PhantomData,
        };
        let cloned = original.clone();
        assert!(cloned.source.is_none());
        assert_eq!(cloned.dealer, original.dealer);
        assert_eq!(cloned.target, original.target);
        assert_f32_eq(cloned.amount, 2.5);
    }

    // ── Behavior 4: `DamageDealt<T>::clone()` does NOT require `T: Clone` ──

    #[test]
    fn clone_does_not_require_t_clone() {
        let original = DamageDealt::<NotCloneable> {
            dealer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  2.0,
            source:  None,
            _marker: PhantomData,
        };
        // require_clone routes the value through a generic `T: Clone` bound,
        // proving DamageDealt<NotCloneable>: Clone exists despite NotCloneable
        // not implementing Clone.
        let cloned = require_clone(&original);
        assert_f32_eq(cloned.amount, 2.0);
    }

    // ── Behavior 5: `DamageDealt<T>` is a Bevy 0.18 `Message` —
    //     `add_message` registration succeeds ──

    #[test]
    fn add_message_registration_succeeds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<DamageDealt<TestT>>();
        assert!(
            app.world()
                .contains_resource::<Messages<DamageDealt<TestT>>>()
        );
    }

    #[test]
    fn init_resource_messages_succeeds() {
        // Edge case: init_resource works directly — confirms the `Message`
        // derive produced a Default `Messages` resource shape.
        let mut world = World::new();
        world.init_resource::<Messages<DamageDealt<TestT>>>();
        assert!(world.contains_resource::<Messages<DamageDealt<TestT>>>());
    }

    // ── Behavior 6: `DamageDealt<T>` round-trips through `MessageWriter` →
    //     `MessageReader` ──

    #[test]
    fn round_trip_writer_to_reader() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<DamageDealt<TestT>>();

        // Write a single message via direct resource access (one of the
        // permitted write idioms per the test spec).
        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .write(DamageDealt::<TestT> {
                dealer:  None,
                target:  Entity::PLACEHOLDER,
                amount:  4.0,
                source:  Some(SourceId::from("module:round")),
                _marker: PhantomData,
            });

        // Tick once so the Bevy message lifecycle exposes the message to readers.
        app.update();

        // Read via direct resource drain (the test spec permits either a
        // registered MessageReader system or `Messages::drain()`).
        let drained: Vec<DamageDealt<TestT>> = app
            .world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .drain()
            .collect();
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 4.0);
        assert_eq!(drained[0].source, Some(SourceId::from("module:round")));
    }

    #[test]
    fn round_trip_second_update_without_write_yields_zero_messages() {
        // Edge case: a second app.update() without another write yields zero
        // messages on a freshly-collected drain (standard Bevy queue drain
        // semantics — the original write has been consumed by the previous
        // update lifecycle).
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<DamageDealt<TestT>>();

        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .write(DamageDealt::<TestT> {
                dealer:  None,
                target:  Entity::PLACEHOLDER,
                amount:  4.0,
                source:  Some(SourceId::from("module:round")),
                _marker: PhantomData,
            });

        // First update — message becomes readable.
        app.update();

        // First drain — collects the message that was written.
        let _first: Vec<DamageDealt<TestT>> = app
            .world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .drain()
            .collect();

        // Second update — no new writes; the queue should now be empty.
        app.update();

        let drained_count = app
            .world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .drain()
            .count();
        assert_eq!(drained_count, 0);
    }
}
