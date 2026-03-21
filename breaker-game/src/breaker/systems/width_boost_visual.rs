//! Visual system to apply [`WidthBoost`] to the breaker's mesh scale each frame.

use bevy::prelude::*;

use crate::{
    breaker::{
        components::{Breaker, BreakerHeight, BreakerWidth},
        queries::WidthBoostVisualQuery,
    },
    chips::components::WidthBoost,
};

/// Sets the breaker's [`Transform`] scale to reflect its effective width.
///
/// When `WidthBoost` is present, effective width = `BreakerWidth + WidthBoost`.
/// Without it, effective width equals `BreakerWidth`.
/// When `EntityScale` is present, both width and height are multiplied by it.
pub(crate) fn width_boost_visual(mut query: Query<WidthBoostVisualQuery, With<Breaker>>) {
    for (breaker_w, width_boost, breaker_h, entity_scale, mut transform) in &mut query {
        let scale = entity_scale.map_or(1.0, |s| s.0);
        let effective_width = (breaker_w.0 + width_boost.map_or(0.0, |b| b.0)) * scale;
        transform.scale = Vec3::new(effective_width, breaker_h.0 * scale, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, width_boost_visual);
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

    #[test]
    fn width_boost_visual_sets_scale_to_effective_width() {
        // Given: BreakerWidth(120.0), BreakerHeight(20.0), WidthBoost(40.0)
        // When: width_boost_visual runs
        // Then: transform.scale = Vec3::new(160.0, 20.0, 1.0)
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerWidth(120.0),
                BreakerHeight(20.0),
                WidthBoost(40.0),
                Transform::default(),
            ))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        let expected = Vec3::new(160.0, 20.0, 1.0);
        assert_eq!(
            tf.scale, expected,
            "scale should be Vec3::new(effective_width=160, height=20, 1), got {:?}",
            tf.scale
        );
    }

    #[test]
    fn entity_scale_applies_to_breaker_dimensions_with_width_boost() {
        // Given: BreakerWidth(120.0), BreakerHeight(20.0), WidthBoost(40.0), EntityScale(0.7)
        // When: width_boost_visual runs
        // Then: transform.scale = Vec3::new((120.0 + 40.0) * 0.7, 20.0 * 0.7, 1.0)
        //                       = Vec3::new(112.0, 14.0, 1.0)
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerWidth(120.0),
                BreakerHeight(20.0),
                WidthBoost(40.0),
                crate::shared::EntityScale(0.7),
                Transform::default(),
            ))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        let expected = Vec3::new(112.0, 14.0, 1.0);
        assert!(
            (tf.scale - expected).length() < 1e-5,
            "scale should be {expected:?} with EntityScale(0.7), got {:?}",
            tf.scale,
        );
    }

    #[test]
    fn entity_scale_identity_matches_without_entity_scale() {
        // Given: BreakerWidth(120.0), BreakerHeight(20.0), WidthBoost(40.0), EntityScale(1.0)
        // When: width_boost_visual runs
        // Then: transform.scale = Vec3::new(160.0, 20.0, 1.0) — same as without EntityScale
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerWidth(120.0),
                BreakerHeight(20.0),
                WidthBoost(40.0),
                crate::shared::EntityScale(1.0),
                Transform::default(),
            ))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        let expected = Vec3::new(160.0, 20.0, 1.0);
        assert_eq!(
            tf.scale, expected,
            "EntityScale(1.0) should produce same result as no EntityScale, got {:?}",
            tf.scale,
        );
    }

    #[test]
    fn entity_scale_applies_without_width_boost() {
        // Given: BreakerWidth(120.0), BreakerHeight(20.0), no WidthBoost, EntityScale(0.5)
        // When: width_boost_visual runs
        // Then: transform.scale = Vec3::new(60.0, 10.0, 1.0)
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerWidth(120.0),
                BreakerHeight(20.0),
                crate::shared::EntityScale(0.5),
                Transform::default(),
            ))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        let expected = Vec3::new(60.0, 10.0, 1.0);
        assert!(
            (tf.scale - expected).length() < 1e-5,
            "scale should be {expected:?} with EntityScale(0.5) and no WidthBoost, got {:?}",
            tf.scale,
        );
    }

    #[test]
    fn no_width_boost_scale_equals_base_dimensions() {
        // Given: BreakerWidth(120.0), BreakerHeight(20.0), no WidthBoost
        // When: width_boost_visual runs
        // Then: transform.scale = Vec3::new(120.0, 20.0, 1.0)
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerWidth(120.0),
                BreakerHeight(20.0),
                // No WidthBoost
                Transform::default(),
            ))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        let expected = Vec3::new(120.0, 20.0, 1.0);
        assert_eq!(
            tf.scale, expected,
            "without WidthBoost, scale should be Vec3::new(width=120, height=20, 1), got {:?}",
            tf.scale
        );
    }
}
