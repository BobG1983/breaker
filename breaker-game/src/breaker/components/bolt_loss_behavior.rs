//! `BoltLossBehavior` — first-class component that controls the breaker's
//! response when a [`crate::bolt::messages::BoltLost`] message fires for it.
//!
//! Replaces the diffuse effect-tree handling of bolt-loss with a single,
//! explicit enum on the breaker entity. RON-deserializable so breaker
//! definitions can pick a variant without referring to the effect tree.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// What a breaker does when it loses a bolt.
///
/// - `LifeLoss(n)` — decrements the breaker's `Hp.current` by `n`, clamped at 0.0.
///   No-op if the breaker has no `Hp` component (Godmode pattern).
/// - `TimeLoss(delta)` — writes a single `ReduceNodeTimer { delta }` message.
///   Does not touch `Hp` even when present.
/// - `None` — complete no-op. Used by Godmode to opt out entirely.
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum BoltLossBehavior {
    /// Lose `n` lives — decrements `Hp.current` by `n` (clamped at 0.0).
    LifeLoss(u32),
    /// Lose `delta` seconds off the node timer — writes one `ReduceNodeTimer { delta }`.
    TimeLoss(f32),
    /// Do nothing.
    None,
}

impl Default for BoltLossBehavior {
    fn default() -> Self {
        Self::LifeLoss(1)
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;

    // ── Behavior 11: BoltLossBehavior round-trips through RON serialization ──

    #[test]
    fn life_loss_variant_round_trips_through_ron() {
        let original = BoltLossBehavior::LifeLoss(1);
        let serialized = ron::ser::to_string(&original).expect("LifeLoss should serialize");
        let deserialized: BoltLossBehavior =
            ron::de::from_str(&serialized).expect("LifeLoss should deserialize");
        assert_eq!(deserialized, original);
    }

    #[test]
    fn time_loss_variant_round_trips_through_ron() {
        let original = BoltLossBehavior::TimeLoss(5.0);
        let serialized = ron::ser::to_string(&original).expect("TimeLoss should serialize");
        let deserialized: BoltLossBehavior =
            ron::de::from_str(&serialized).expect("TimeLoss should deserialize");
        assert_eq!(deserialized, original);
    }

    #[test]
    fn none_variant_round_trips_through_ron() {
        let original = BoltLossBehavior::None;
        let serialized = ron::ser::to_string(&original).expect("None should serialize");
        let deserialized: BoltLossBehavior =
            ron::de::from_str(&serialized).expect("None should deserialize");
        assert_eq!(deserialized, original);
    }

    #[test]
    fn life_loss_literal_ron_string_deserializes() {
        let deserialized: BoltLossBehavior =
            ron::de::from_str("LifeLoss(1)").expect("\"LifeLoss(1)\" should deserialize");
        assert_eq!(deserialized, BoltLossBehavior::LifeLoss(1));
    }

    #[test]
    fn time_loss_literal_ron_string_deserializes() {
        let deserialized: BoltLossBehavior =
            ron::de::from_str("TimeLoss(5.0)").expect("\"TimeLoss(5.0)\" should deserialize");
        assert_eq!(deserialized, BoltLossBehavior::TimeLoss(5.0));
    }

    #[test]
    fn none_literal_ron_string_deserializes() {
        let deserialized: BoltLossBehavior =
            ron::de::from_str("None").expect("\"None\" should deserialize");
        assert_eq!(deserialized, BoltLossBehavior::None);
    }

    // ── Behavior 12: BoltLossBehavior has required derives ──

    /// Routes `Clone` through a generic bound so the `.clone()` call is
    /// gated on the trait existing, without tripping `clippy::clone_on_copy`.
    #[must_use]
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    #[test]
    fn boltlossbehavior_has_required_derives() {
        let a = BoltLossBehavior::LifeLoss(1);
        // Copy
        let b = a;
        // Clone (gated through a generic to keep clippy happy on a Copy type)
        let c = require_clone(&a);
        // PartialEq
        assert_eq!(a, b);
        assert_eq!(a, c);
        // Debug
        let dbg = format!("{a:?}");
        assert!(
            dbg.contains("LifeLoss"),
            "Debug format should contain \"LifeLoss\", got {dbg}",
        );
        // Component
        let mut world = World::new();
        let entity = world.spawn(a).id();
        let component = world
            .get::<BoltLossBehavior>(entity)
            .expect("BoltLossBehavior should be queryable as a Component");
        assert_eq!(*component, BoltLossBehavior::LifeLoss(1));
    }
}
