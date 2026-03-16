//! Components for the fx domain.

use bevy::prelude::*;

/// A fade-out animation timer. Entities with this component will have their
/// alpha reduced over `duration` seconds and be despawned when finished.
///
/// Used across domains (bolt-lost text, bump grade text) for floating feedback.
#[derive(Component, Debug)]
pub struct FadeOut {
    /// Remaining time in the fade animation (seconds).
    pub timer: f32,
    /// Total duration of the fade animation (seconds).
    pub duration: f32,
}
