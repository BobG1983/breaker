//! Visual system to apply [`WidthBoost`] to the breaker's mesh scale each frame.

use bevy::prelude::*;

use crate::breaker::{components::Breaker, queries::WidthBoostVisualQuery};
#[cfg(test)]
use crate::{
    breaker::components::{BreakerHeight, BreakerWidth},
    chips::components::WidthBoost,
};

/// Sets the breaker's [`Scale2D`] to reflect its effective width.
///
/// When `WidthBoost` is present, effective width = `BreakerWidth + WidthBoost`.
/// Without it, effective width equals `BreakerWidth`.
/// When `EntityScale` is present, both width and height are multiplied by it.
pub(crate) fn width_boost_visual(mut query: Query<WidthBoostVisualQuery, With<Breaker>>) {
    for (breaker_w, width_boost, breaker_h, entity_scale, mut scale) in &mut query {
        let entity_s = entity_scale.map_or(1.0, |s| s.0);
        let effective_width = (breaker_w.0 + width_boost.map_or(0.0, |b| b.0)) * entity_s;
        scale.x = effective_width;
        scale.y = breaker_h.0 * entity_s;
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::Scale2D;

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
    fn width_boost_visual_sets_scale2d_to_effective_width() {
        // Given: BreakerWidth(120.0), BreakerHeight(20.0), WidthBoost(40.0)
        // When: width_boost_visual runs
        // Then: Scale2D { x: 160.0, y: 20.0 }
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerWidth(120.0),
                BreakerHeight(20.0),
                WidthBoost(40.0),
                Scale2D { x: 120.0, y: 20.0 },
            ))
            .id();

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 160.0).abs() < f32::EPSILON && (scale.y - 20.0).abs() < f32::EPSILON,
            "Scale2D should be (160.0, 20.0), got ({}, {})",
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn entity_scale_applies_to_breaker_dimensions_with_width_boost() {
        // Given: BreakerWidth(120.0), BreakerHeight(20.0), WidthBoost(40.0), EntityScale(0.7)
        // When: width_boost_visual runs
        // Then: Scale2D { x: 112.0, y: 14.0 }
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerWidth(120.0),
                BreakerHeight(20.0),
                WidthBoost(40.0),
                crate::shared::EntityScale(0.7),
                Scale2D { x: 120.0, y: 20.0 },
            ))
            .id();

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 112.0).abs() < 1e-5 && (scale.y - 14.0).abs() < 1e-5,
            "Scale2D should be (112.0, 14.0) with EntityScale(0.7), got ({}, {})",
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn entity_scale_identity_matches_without_entity_scale() {
        // Given: BreakerWidth(120.0), BreakerHeight(20.0), WidthBoost(40.0), EntityScale(1.0)
        // When: width_boost_visual runs
        // Then: Scale2D { x: 160.0, y: 20.0 } — same as without EntityScale
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerWidth(120.0),
                BreakerHeight(20.0),
                WidthBoost(40.0),
                crate::shared::EntityScale(1.0),
                Scale2D { x: 120.0, y: 20.0 },
            ))
            .id();

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 160.0).abs() < f32::EPSILON && (scale.y - 20.0).abs() < f32::EPSILON,
            "EntityScale(1.0) should produce Scale2D (160.0, 20.0), got ({}, {})",
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn entity_scale_applies_without_width_boost() {
        // Given: BreakerWidth(120.0), BreakerHeight(20.0), no WidthBoost, EntityScale(0.5)
        // When: width_boost_visual runs
        // Then: Scale2D { x: 60.0, y: 10.0 }
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerWidth(120.0),
                BreakerHeight(20.0),
                crate::shared::EntityScale(0.5),
                Scale2D { x: 120.0, y: 20.0 },
            ))
            .id();

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 60.0).abs() < 1e-5 && (scale.y - 10.0).abs() < 1e-5,
            "Scale2D should be (60.0, 10.0) with EntityScale(0.5) and no WidthBoost, got ({}, {})",
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn no_width_boost_scale2d_equals_base_dimensions() {
        // Edge case: No WidthBoost -> Scale2D { x: 120.0, y: 20.0 }
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerWidth(120.0),
                BreakerHeight(20.0),
                // No WidthBoost
                Scale2D { x: 1.0, y: 1.0 },
            ))
            .id();

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 120.0).abs() < f32::EPSILON && (scale.y - 20.0).abs() < f32::EPSILON,
            "without WidthBoost, Scale2D should be (120.0, 20.0), got ({}, {})",
            scale.x,
            scale.y,
        );
    }
}
