//! Unified health component for all damageable entities.

use bevy::prelude::*;

/// Unified health component for all damageable entities. Replaces `CellHealth`
/// and `LivesCount`.
///
/// - `current`: hit points remaining. Damage decrements this. Death occurs when
///   `current <= 0.0`.
/// - `starting`: the HP this entity spawned with. Used for visual damage feedback
///   (health fraction = `current / starting`).
/// - `max`: optional upper bound. If `Some`, healing cannot exceed this value.
#[derive(Component, Debug, Clone)]
pub(crate) struct Hp {
    /// Hit points remaining.
    pub current:  f32,
    /// The HP this entity spawned with.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "awaiting health-fraction visual feedback consumer"
        )
    )]
    pub starting: f32,
    /// Optional upper bound for healing.
    #[expect(dead_code, reason = "awaiting healing system consumer")]
    pub max:      Option<f32>,
}

impl Hp {
    /// Creates a new `Hp` with `current` and `starting` set to the given value,
    /// and `max` set to `None`.
    #[must_use]
    pub(crate) const fn new(starting: f32) -> Self {
        Self {
            current: starting,
            starting,
            max: None,
        }
    }
}
