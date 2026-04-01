//! Visual system to apply size multiplier to the breaker's mesh scale each frame.

use bevy::prelude::*;

#[cfg(test)]
use crate::breaker::components::{BaseHeight, BaseWidth};
use crate::{
    breaker::{components::Breaker, queries::WidthBoostVisualQuery},
    effect::effects::size_boost::ActiveSizeBoosts,
};

/// Sets the breaker's [`Scale2D`] to reflect its effective width.
///
/// When `ActiveSizeBoosts` is present, effective width = `BaseWidth * multiplier`.
/// Without it, effective width equals `BaseWidth`.
/// When `NodeScalingFactor` is present, both width and height are multiplied by it.
pub(crate) fn width_boost_visual(mut query: Query<WidthBoostVisualQuery, With<Breaker>>) {
    for (breaker_w, size_mult, breaker_h, entity_scale, mut scale) in &mut query {
        let entity_s = entity_scale.map_or(1.0, |s| s.0);
        let effective_width =
            breaker_w.0 * size_mult.map_or(1.0, ActiveSizeBoosts::multiplier) * entity_s;
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
    fn effective_size_multiplier_visual_sets_scale2d_multiplicatively() {
        // Given: BaseWidth(120.0), BaseHeight(20.0), ActiveSizeBoosts(vec![4/3])
        // When: width_boost_visual runs
        // Then: Scale2D { x: 160.0, y: 20.0 } (120 * 4/3 = 160)
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BaseWidth(120.0),
                BaseHeight(20.0),
                ActiveSizeBoosts(vec![4.0_f32 / 3.0]),
                Scale2D { x: 120.0, y: 20.0 },
            ))
            .id();

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 160.0).abs() < 1e-5 && (scale.y - 20.0).abs() < f32::EPSILON,
            "Scale2D should be (160.0, 20.0), got ({}, {})",
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn effective_size_multiplier_visual_with_entity_scale() {
        // Given: BaseWidth(120.0), BaseHeight(20.0), ActiveSizeBoosts(vec![4/3]), NodeScalingFactor(0.7)
        // When: width_boost_visual runs
        // Then: Scale2D { x: 112.0, y: 14.0 } (120 * 4/3 * 0.7 = 112)
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BaseWidth(120.0),
                BaseHeight(20.0),
                ActiveSizeBoosts(vec![4.0_f32 / 3.0]),
                crate::shared::NodeScalingFactor(0.7),
                Scale2D { x: 120.0, y: 20.0 },
            ))
            .id();

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 112.0).abs() < 1e-5 && (scale.y - 14.0).abs() < 1e-5,
            "Scale2D should be (112.0, 14.0) with ActiveSizeBoosts([4/3]) and NodeScalingFactor(0.7), got ({}, {})",
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn effective_size_multiplier_visual_with_entity_scale_identity() {
        // Given: BaseWidth(120.0), BaseHeight(20.0), ActiveSizeBoosts(vec![4/3]), NodeScalingFactor(1.0)
        // When: width_boost_visual runs
        // Then: Scale2D { x: 160.0, y: 20.0 } — same as without NodeScalingFactor (120 * 4/3 * 1.0)
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BaseWidth(120.0),
                BaseHeight(20.0),
                ActiveSizeBoosts(vec![4.0_f32 / 3.0]),
                crate::shared::NodeScalingFactor(1.0),
                Scale2D { x: 120.0, y: 20.0 },
            ))
            .id();

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 160.0).abs() < 1e-5 && (scale.y - 20.0).abs() < f32::EPSILON,
            "NodeScalingFactor(1.0) should produce Scale2D (160.0, 20.0), got ({}, {})",
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn no_effective_size_multiplier_with_entity_scale() {
        // Given: BaseWidth(120.0), BaseHeight(20.0), no ActiveSizeBoosts, NodeScalingFactor(0.5)
        // When: width_boost_visual runs
        // Then: Scale2D { x: 60.0, y: 10.0 } (120 * 0.5, 20 * 0.5)
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BaseWidth(120.0),
                BaseHeight(20.0),
                crate::shared::NodeScalingFactor(0.5),
                Scale2D { x: 120.0, y: 20.0 },
            ))
            .id();

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 60.0).abs() < 1e-5 && (scale.y - 10.0).abs() < 1e-5,
            "Scale2D should be (60.0, 10.0) with NodeScalingFactor(0.5) and no ActiveSizeBoosts, got ({}, {})",
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn no_effective_size_multiplier_scale2d_equals_base_dimensions() {
        // Edge case: No ActiveSizeBoosts -> Scale2D { x: 120.0, y: 20.0 }
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BaseWidth(120.0),
                BaseHeight(20.0),
                // No ActiveSizeBoosts
                Scale2D { x: 1.0, y: 1.0 },
            ))
            .id();

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 120.0).abs() < f32::EPSILON && (scale.y - 20.0).abs() < f32::EPSILON,
            "without ActiveSizeBoosts, Scale2D should be (120.0, 20.0), got ({}, {})",
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn width_boost_visual_reads_active_size_boosts_for_scale() {
        // Given: Breaker with ActiveSizeBoosts(vec![1.5]), BaseWidth(120.0),
        //        BaseHeight(20.0), Scale2D(Vec2::ONE)
        // When: width_boost_visual runs
        // Then: Scale2D.x = 120.0 * 1.5 = 180.0, Scale2D.y = 20.0
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BaseWidth(120.0),
                BaseHeight(20.0),
                ActiveSizeBoosts(vec![1.5]),
                Scale2D { x: 1.0, y: 1.0 },
            ))
            .id();

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 180.0).abs() < 1e-5,
            "Scale2D.x should be 180.0 (120.0 * 1.5) with ActiveSizeBoosts([1.5]), got {}",
            scale.x,
        );
        assert!(
            (scale.y - 20.0).abs() < f32::EPSILON,
            "Scale2D.y should be 20.0, got {}",
            scale.y,
        );
    }

    #[test]
    fn width_boost_visual_no_active_size_boosts_uses_base_width() {
        // Edge case: No ActiveSizeBoosts -> Scale2D.x = 120.0 (base BaseWidth)
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BaseWidth(120.0),
                BaseHeight(20.0),
                Scale2D { x: 1.0, y: 1.0 },
            ))
            .id();

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 120.0).abs() < f32::EPSILON,
            "Scale2D.x should be 120.0 (base width, no boosts), got {}",
            scale.x,
        );
        assert!(
            (scale.y - 20.0).abs() < f32::EPSILON,
            "Scale2D.y should be 20.0, got {}",
            scale.y,
        );
    }
}
