//! Tilt visual feedback — applies `BreakerTilt.angle` to the breaker's rotation.

use bevy::prelude::*;

use crate::breaker::components::{Breaker, BreakerTilt};

/// Copies [`BreakerTilt::angle`] into the breaker's Z rotation each frame.
///
/// Sign convention: `BreakerTilt` positive = tilted right (clockwise in
/// screen space), which maps to negative Z rotation in Bevy's CCW-positive
/// coordinate system.
pub fn animate_tilt_visual(mut query: Query<(&BreakerTilt, &mut Transform), With<Breaker>>) {
    for (tilt, mut transform) in &mut query {
        transform.rotation = Quat::from_rotation_z(-tilt.angle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs `animate_tilt_visual` with the given tilt angle and returns the
    /// resulting Z rotation (euler angle).
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
            Transform::default(),
        ));

        app.update();

        let tf = app
            .world_mut()
            .query_filtered::<&Transform, With<Breaker>>()
            .iter(app.world())
            .next()
            .unwrap();

        let (_, _, z) = tf.rotation.to_euler(EulerRot::XYZ);
        z
    }

    #[test]
    fn tilt_at_zero_gives_identity_rotation() {
        let z = run_tilt(0.0);
        assert!(
            z.abs() < 1e-5,
            "zero tilt should give identity rotation, got z={z}"
        );
    }

    #[test]
    fn positive_tilt_rotates_clockwise() {
        let z = run_tilt(0.3);
        assert!(
            z < 0.0,
            "positive tilt should produce negative Z rotation (clockwise), got z={z}"
        );
    }

    #[test]
    fn negative_tilt_rotates_counterclockwise() {
        let z = run_tilt(-0.3);
        assert!(
            z > 0.0,
            "negative tilt should produce positive Z rotation (counterclockwise), got z={z}"
        );
    }
}
