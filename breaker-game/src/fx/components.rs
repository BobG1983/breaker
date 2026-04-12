//! Components for the fx domain.

use bevy::prelude::*;

/// A fade-out animation timer. Entities with this component will have their
/// alpha reduced over `duration` seconds and be despawned when finished.
///
/// Used across domains (bolt-lost text, bump grade text) for floating feedback.
#[derive(Component, Debug)]
pub(crate) struct FadeOut {
    /// Remaining time in the fade animation (seconds).
    pub timer:    f32,
    /// Total duration of the fade animation (seconds).
    pub duration: f32,
}

/// A countdown timer for one-shot flash visual entities.
/// Entities with this component are despawned when the timer reaches zero.
///
/// Used for instant effect flashes (piercing beam, explode) that appear
/// briefly and disappear — no opacity fading, just spawn and despawn.
#[derive(Component, Debug)]
pub(crate) struct EffectFlashTimer(pub f32);

/// A scale-overshoot animation that punches in then settles to 1.0.
///
/// Entities with this component will have their `Transform.scale` animated
/// from `overshoot` back to 1.0 over `duration` seconds. When finished the
/// component is removed (entity is NOT despawned).
#[derive(Component, Debug)]
pub(crate) struct PunchScale {
    /// Remaining time in the animation (seconds).
    pub timer:     f32,
    /// Total duration of the animation (seconds).
    pub duration:  f32,
    /// Initial scale multiplier (e.g. 1.15 for a 15% overshoot).
    pub overshoot: f32,
}
