//! Death-confirmed message — sent after a kill handler confirms the kill.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::traits::Dmgable;

/// Death confirmed. Sent after the per-`T` kill handler finishes its domain
/// work. The victim is **still alive** at this point — it survives through
/// trigger evaluation and death animation; actual despawn is queued later via
/// `DespawnEntity`.
#[derive(Message, Debug)]
pub struct Destroyed<T: Dmgable> {
    /// The entity that died (still alive at message time).
    pub victim:     Entity,
    /// The entity that caused the death (`None` for environmental deaths).
    pub killer:     Option<Entity>,
    /// World position of the victim at time of death.
    pub victim_pos: Vec2,
    /// World position of the killer, if available. Used for directional VFX.
    pub killer_pos: Option<Vec2>,
    /// Marker selecting the per-`T` message queue.
    pub _marker:    PhantomData<T>,
}

// Manual `Clone` impl — `PhantomData` is always `Clone`; `T` is NOT required
// to be `Clone`. Every data field forwards from `self`.
impl<T: Dmgable> Clone for Destroyed<T> {
    fn clone(&self) -> Self {
        Self {
            victim:     self.victim,
            killer:     self.killer,
            victim_pos: self.victim_pos,
            killer_pos: self.killer_pos,
            _marker:    PhantomData,
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

    // ── Behavior 17: `Destroyed<T>` constructs with all fields populated
    //     including `Vec2`s ──

    #[test]
    fn constructs_with_all_fields_populated() {
        let msg = Destroyed::<TestT> {
            victim:     Entity::PLACEHOLDER,
            killer:     Some(Entity::PLACEHOLDER),
            victim_pos: Vec2::new(1.0, 2.0),
            killer_pos: Some(Vec2::new(3.0, 4.0)),
            _marker:    PhantomData,
        };
        assert_eq!(msg.victim, Entity::PLACEHOLDER);
        assert_eq!(msg.killer, Some(Entity::PLACEHOLDER));
        assert_f32_eq(msg.victim_pos.x, 1.0);
        assert_f32_eq(msg.victim_pos.y, 2.0);
        let Some(killer_pos) = msg.killer_pos else {
            panic!("killer_pos should be Some");
        };
        assert_f32_eq(killer_pos.x, 3.0);
        assert_f32_eq(killer_pos.y, 4.0);
    }

    #[test]
    fn constructs_with_zero_victim_pos() {
        // Edge case: victim_pos == Vec2::ZERO constructs cleanly.
        let msg = Destroyed::<TestT> {
            victim:     Entity::PLACEHOLDER,
            killer:     Some(Entity::PLACEHOLDER),
            victim_pos: Vec2::ZERO,
            killer_pos: Some(Vec2::new(3.0, 4.0)),
            _marker:    PhantomData,
        };
        assert_f32_eq(msg.victim_pos.x, 0.0);
        assert_f32_eq(msg.victim_pos.y, 0.0);
    }

    // ── Behavior 18: `Destroyed<T>` constructs with `killer_pos: None` and
    //     `killer: None` ──

    #[test]
    fn constructs_with_none_killer_and_none_killer_pos() {
        let msg = Destroyed::<TestT> {
            victim:     Entity::PLACEHOLDER,
            killer:     None,
            victim_pos: Vec2::new(-1.5, 2.5),
            killer_pos: None,
            _marker:    PhantomData,
        };
        assert!(msg.killer.is_none());
        assert!(msg.killer_pos.is_none());
        assert_f32_eq(msg.victim_pos.x, -1.5);
        assert_f32_eq(msg.victim_pos.y, 2.5);
    }

    #[test]
    fn constructs_with_neg_infinity_victim_pos() {
        // Edge case: victim_pos.x == f32::NEG_INFINITY constructs cleanly —
        // no position validation.
        let msg = Destroyed::<TestT> {
            victim:     Entity::PLACEHOLDER,
            killer:     None,
            victim_pos: Vec2::new(f32::NEG_INFINITY, 0.0),
            killer_pos: None,
            _marker:    PhantomData,
        };
        assert!(msg.victim_pos.x.is_infinite());
        assert!(msg.victim_pos.x.is_sign_negative());
    }

    // ── Behavior 19: `Destroyed<T>::clone()` preserves every field including
    //     both `Vec2`s ──

    #[test]
    fn clone_preserves_every_field() {
        let original = Destroyed::<TestT> {
            victim:     Entity::PLACEHOLDER,
            killer:     Some(Entity::PLACEHOLDER),
            victim_pos: Vec2::new(10.0, -10.0),
            killer_pos: Some(Vec2::new(20.0, 20.0)),
            _marker:    PhantomData,
        };
        let cloned = original.clone();
        assert_eq!(cloned.victim, original.victim);
        assert_eq!(cloned.killer, original.killer);
        assert_f32_eq(cloned.victim_pos.x, 10.0);
        assert_f32_eq(cloned.victim_pos.y, -10.0);
        let Some(killer_pos) = cloned.killer_pos else {
            panic!("killer_pos should be Some after clone");
        };
        assert_f32_eq(killer_pos.x, 20.0);
        assert_f32_eq(killer_pos.y, 20.0);
    }

    #[test]
    fn clone_preserves_none_killer_pos() {
        // Edge case: cloning with killer_pos: None preserves the None.
        let original = Destroyed::<TestT> {
            victim:     Entity::PLACEHOLDER,
            killer:     None,
            victim_pos: Vec2::new(5.0, 6.0),
            killer_pos: None,
            _marker:    PhantomData,
        };
        let cloned = original.clone();
        assert_eq!(cloned.victim, original.victim);
        assert_eq!(cloned.killer, original.killer);
        assert_eq!(cloned.killer_pos, original.killer_pos);
        assert!(cloned.killer_pos.is_none());
        assert_f32_eq(cloned.victim_pos.x, 5.0);
        assert_f32_eq(cloned.victim_pos.y, 6.0);
    }

    // ── Behavior 20: `Destroyed<T>::clone()` does NOT require `T: Clone` ──

    #[test]
    fn clone_does_not_require_t_clone() {
        let original = Destroyed::<NotCloneable> {
            victim:     Entity::PLACEHOLDER,
            killer:     None,
            victim_pos: Vec2::new(1.0, 2.0),
            killer_pos: None,
            _marker:    PhantomData,
        };
        let cloned = require_clone(&original);
        assert_f32_eq(cloned.victim_pos.x, 1.0);
        assert_f32_eq(cloned.victim_pos.y, 2.0);
    }

    // ── Behavior 21: `Destroyed<T>` is a Bevy `Message` — `add_message`
    //     registration succeeds ──

    #[test]
    fn add_message_registration_succeeds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<Destroyed<TestT>>();
        assert!(
            app.world()
                .contains_resource::<Messages<Destroyed<TestT>>>()
        );
    }

    // ── Behavior 21 edge: `init_resource::<Messages<Destroyed<T>>>()`
    //     succeeds on a bare `World` ──

    #[test]
    fn init_resource_messages_succeeds() {
        // Edge case: init_resource works directly — confirms the `Message`
        // derive produced a Default `Messages` resource shape.
        let mut world = World::new();
        world.init_resource::<Messages<Destroyed<TestT>>>();
        assert!(world.contains_resource::<Messages<Destroyed<TestT>>>());
    }
}
