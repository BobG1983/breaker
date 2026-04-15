//! Tilt visual feedback — applies `BreakerTilt.angle` to the breaker's rotation.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Rotation2D;

use crate::{breaker::components::BreakerTilt, prelude::*};

/// Copies [`BreakerTilt::angle`] into the breaker's `Rotation2D` each frame.
///
/// Sign convention: `BreakerTilt` positive = tilted right (clockwise in
/// screen space), which maps to negative rotation in Bevy's CCW-positive
/// coordinate system.
pub fn animate_tilt_visual(mut query: Query<(&BreakerTilt, &mut Rotation2D), With<Breaker>>) {
    for (tilt, mut rotation) in &mut query {
        rotation.0 = Rot2::radians(-tilt.angle);
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::Rotation2D;

    use super::*;

    /// Runs `animate_tilt_visual` with the given tilt angle and returns the
    /// resulting `Rotation2D` in radians.
    fn run_tilt(angle: f32) -> f32 {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, animate_tilt_visual);

        app.world_mut().spawn((
            Breaker,
            BreakerTilt {
                angle,
                ease_start: 0.0,
                ease_target: 0.0,
            },
            Rotation2D::default(),
        ));

        app.update();

        let rotation = app
            .world_mut()
            .query_filtered::<&Rotation2D, With<Breaker>>()
            .iter(app.world())
            .next()
            .unwrap();

        rotation.as_radians()
    }

    #[test]
    fn tilt_at_zero_gives_identity_rotation2d() {
        // Edge case: angle=0.0 -> Rotation2D identity
        let radians = run_tilt(0.0);
        assert!(
            radians.abs() < 1e-5,
            "zero tilt should give identity Rotation2D, got radians={radians}"
        );
    }

    #[test]
    fn positive_tilt_produces_negative_rotation2d() {
        // Given: BreakerTilt { angle: 0.3 }, Rotation2D::default()
        // When: animate_tilt_visual runs
        // Then: Rotation2D angle is approximately -0.3 radians
        let radians = run_tilt(0.3);
        assert!(
            (radians - (-0.3)).abs() < 1e-5,
            "positive tilt 0.3 should produce Rotation2D of -0.3 radians (clockwise), got {radians}"
        );
    }

    #[test]
    fn negative_tilt_produces_positive_rotation2d() {
        let radians = run_tilt(-0.3);
        assert!(
            (radians - 0.3).abs() < 1e-5,
            "negative tilt -0.3 should produce Rotation2D of 0.3 radians (counterclockwise), got {radians}"
        );
    }
}
