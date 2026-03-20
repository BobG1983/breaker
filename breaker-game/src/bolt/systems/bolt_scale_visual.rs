//! Visual system to set bolt [`Transform`] scale from [`BoltRadius`] and optional [`EntityScale`].

use bevy::prelude::*;

use crate::{
    bolt::components::{Bolt, BoltRadius},
    shared::EntityScale,
};

/// Sets bolt [`Transform`] scale based on [`BoltRadius`] and optional [`EntityScale`].
///
/// When [`EntityScale`] is present, scale = `BoltRadius * EntityScale` on X and Y.
/// Without it, scale equals `BoltRadius` (backward compatible).
pub(crate) fn bolt_scale_visual(
    mut query: Query<(&BoltRadius, Option<&EntityScale>, &mut Transform), With<Bolt>>,
) {
    for (radius, entity_scale, mut transform) in &mut query {
        let scale = entity_scale.map_or(1.0, |s| s.0);
        let effective = radius.0 * scale;
        transform.scale = Vec3::new(effective, effective, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, bolt_scale_visual);
        app
    }

    /// Accumulates one fixed timestep then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    /// Tolerance for floating-point comparisons involving multiplication.
    const TOLERANCE: f32 = 1e-6;

    #[test]
    fn bolt_scale_visual_applies_entity_scale_to_radius() {
        // Given: Bolt with BoltRadius(8.0), EntityScale(0.7)
        // When: bolt_scale_visual runs
        // Then: transform.scale = Vec3::new(8.0 * 0.7, 8.0 * 0.7, 1.0) = Vec3::new(5.6, 5.6, 1.0)
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltRadius(8.0),
                EntityScale(0.7),
                Transform::default(),
            ))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        let expected = Vec3::new(8.0 * 0.7, 8.0 * 0.7, 1.0);
        assert!(
            (tf.scale.x - expected.x).abs() < TOLERANCE
                && (tf.scale.y - expected.y).abs() < TOLERANCE
                && (tf.scale.z - expected.z).abs() < TOLERANCE,
            "scale should be {expected:?}, got {:?}",
            tf.scale,
        );
    }

    #[test]
    fn bolt_scale_visual_entity_scale_one_equals_radius() {
        // Edge case: EntityScale(1.0) -- scale = Vec3::new(8.0, 8.0, 1.0)
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltRadius(8.0),
                EntityScale(1.0),
                Transform::default(),
            ))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        let expected = Vec3::new(8.0, 8.0, 1.0);
        assert!(
            (tf.scale.x - expected.x).abs() < TOLERANCE
                && (tf.scale.y - expected.y).abs() < TOLERANCE
                && (tf.scale.z - expected.z).abs() < TOLERANCE,
            "scale with EntityScale(1.0) should be {expected:?}, got {:?}",
            tf.scale,
        );
    }

    #[test]
    fn bolt_scale_visual_without_entity_scale_defaults_to_unscaled() {
        // Given: Bolt with BoltRadius(8.0), NO EntityScale
        // When: bolt_scale_visual runs
        // Then: transform.scale = Vec3::new(8.0, 8.0, 1.0)
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((Bolt, BoltRadius(8.0), Transform::default()))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        let expected = Vec3::new(8.0, 8.0, 1.0);
        assert!(
            (tf.scale.x - expected.x).abs() < TOLERANCE
                && (tf.scale.y - expected.y).abs() < TOLERANCE
                && (tf.scale.z - expected.z).abs() < TOLERANCE,
            "without EntityScale, scale should be {expected:?}, got {:?}",
            tf.scale,
        );
    }

    #[test]
    fn bolt_scale_visual_scales_multiple_bolts_independently() {
        // Given: Bolt A with EntityScale(0.7), Bolt B with EntityScale(0.5)
        // When: bolt_scale_visual runs
        // Then: A = Vec3::new(5.6, 5.6, 1.0), B = Vec3::new(4.0, 4.0, 1.0)
        let mut app = test_app();

        let bolt_a = app
            .world_mut()
            .spawn((
                Bolt,
                BoltRadius(8.0),
                EntityScale(0.7),
                Transform::default(),
            ))
            .id();

        let bolt_b = app
            .world_mut()
            .spawn((
                Bolt,
                BoltRadius(8.0),
                EntityScale(0.5),
                Transform::default(),
            ))
            .id();

        tick(&mut app);

        let tf_a = app.world().get::<Transform>(bolt_a).unwrap();
        let expected_a = Vec3::new(8.0 * 0.7, 8.0 * 0.7, 1.0);
        assert!(
            (tf_a.scale.x - expected_a.x).abs() < TOLERANCE
                && (tf_a.scale.y - expected_a.y).abs() < TOLERANCE
                && (tf_a.scale.z - expected_a.z).abs() < TOLERANCE,
            "bolt A scale should be {expected_a:?}, got {:?}",
            tf_a.scale,
        );

        let tf_b = app.world().get::<Transform>(bolt_b).unwrap();
        let expected_b = Vec3::new(8.0 * 0.5, 8.0 * 0.5, 1.0);
        assert!(
            (tf_b.scale.x - expected_b.x).abs() < TOLERANCE
                && (tf_b.scale.y - expected_b.y).abs() < TOLERANCE
                && (tf_b.scale.z - expected_b.z).abs() < TOLERANCE,
            "bolt B scale should be {expected_b:?}, got {:?}",
            tf_b.scale,
        );
    }
}
