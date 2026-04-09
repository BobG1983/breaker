//! Birthing animation component and duration constant.
//!
//! Passive types only -- the `tick_birthing` system lives in the bolt domain.

use bevy::prelude::*;
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::Scale2D;

/// Duration of the birthing animation in seconds.
///
/// Must be short enough to avoid dead air (see `docs/design/decisions/animate-in-timing.md`).
pub(crate) const BIRTHING_DURATION: f32 = 0.15;

/// Tracks an entity that is animating into existence.
///
/// While `Birthing` is present, the entity's `Scale2D` lerps from zero
/// toward `target_scale` and its `CollisionLayers` are zeroed. On
/// completion, `target_scale` is applied exactly and `stashed_layers`
/// are restored.
#[derive(Component, Debug)]
pub struct Birthing {
    /// Timer tracking animation progress.
    pub(crate) timer: Timer,
    /// The scale the entity will reach when birthing completes.
    pub(crate) target_scale: Scale2D,
    /// The collision layers to restore when birthing completes.
    pub(crate) stashed_layers: CollisionLayers,
}

impl Birthing {
    /// Creates a new Birthing component that will animate from zero scale to `target_scale`,
    /// restoring `stashed_layers` on completion.
    #[must_use]
    pub fn new(target_scale: Scale2D, stashed_layers: CollisionLayers) -> Self {
        Self {
            timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
            target_scale,
            stashed_layers,
        }
    }

    /// Returns the fraction complete (0.0 to 1.0).
    #[must_use]
    pub(crate) fn fraction(&self) -> f32 {
        self.timer.fraction()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    // Behavior 1: Birthing component stores timer, target_scale, and stashed_layers
    #[test]
    fn birthing_stores_timer_target_scale_and_stashed_layers() {
        let birthing = Birthing {
            timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
            target_scale: Scale2D { x: 8.0, y: 8.0 },
            stashed_layers: CollisionLayers::new(0x01, 0x0E),
        };

        assert_eq!(
            birthing.timer.duration(),
            Duration::from_secs_f32(BIRTHING_DURATION),
            "timer duration should be {BIRTHING_DURATION}s"
        );
        assert!(
            (birthing.target_scale.x - 8.0).abs() < f32::EPSILON,
            "target_scale.x should be 8.0"
        );
        assert!(
            (birthing.target_scale.y - 8.0).abs() < f32::EPSILON,
            "target_scale.y should be 8.0"
        );
        assert_eq!(
            birthing.stashed_layers.membership, 0x01,
            "stashed_layers membership should be 0x01"
        );
        assert_eq!(
            birthing.stashed_layers.mask, 0x0E,
            "stashed_layers mask should be 0x0E"
        );
    }

    #[test]
    fn birthing_stashed_layers_default_stored_exactly() {
        let birthing = Birthing {
            timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
            target_scale: Scale2D { x: 8.0, y: 8.0 },
            stashed_layers: CollisionLayers::default(),
        };

        assert_eq!(
            birthing.stashed_layers.membership, 0,
            "default stashed_layers membership should be 0"
        );
        assert_eq!(
            birthing.stashed_layers.mask, 0,
            "default stashed_layers mask should be 0"
        );
    }

    // Behavior 2: BIRTHING_DURATION constant is 0.3 seconds
    #[test]
    fn birthing_duration_is_zero_point_fifteen() {
        assert!(
            (BIRTHING_DURATION - 0.15).abs() < f32::EPSILON,
            "BIRTHING_DURATION should be 0.15, got {BIRTHING_DURATION}"
        );
    }
}
