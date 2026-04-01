//! Visual system to set bolt [`Scale2D`] from [`BoltRadius`] and optional [`NodeScalingFactor`].

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Scale2D;

use crate::{
    bolt::components::{Bolt, BoltRadius},
    shared::NodeScalingFactor,
};

/// Sets bolt [`Scale2D`] based on [`BoltRadius`] and optional [`NodeScalingFactor`].
///
/// When [`NodeScalingFactor`] is present, scale = `BoltRadius * NodeScalingFactor` on X and Y.
/// Without it, scale equals `BoltRadius` (backward compatible).
pub(crate) fn bolt_scale_visual(
    mut query: Query<(&BoltRadius, Option<&NodeScalingFactor>, &mut Scale2D), With<Bolt>>,
) {
    for (radius, entity_scale, mut scale) in &mut query {
        let factor = entity_scale.map_or(1.0, |s| s.0);
        let effective = radius.0 * factor;
        scale.x = effective;
        scale.y = effective;
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::Velocity2D;

    use super::*;
    use crate::bolt::definition::BoltDefinition;

    fn test_bolt_definition() -> BoltDefinition {
        BoltDefinition {
            name: "Bolt".to_string(),
            base_speed: 400.0,
            min_speed: 200.0,
            max_speed: 800.0,
            radius: 8.0,
            base_damage: 10.0,
            effects: vec![],
            color_rgb: [6.0, 5.0, 0.5],
            min_angle_horizontal: 5.0,
            min_angle_vertical: 5.0,
        }
    }

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
    fn bolt_scale_visual_writes_scale2d_with_entity_scale() {
        // Given: Bolt with BoltRadius(8.0), NodeScalingFactor(0.7), Scale2D { x: 1.0, y: 1.0 }
        // When: bolt_scale_visual runs
        // Then: Scale2D { x: 5.6, y: 5.6 }
        let mut app = test_app();

        let entity = Bolt::builder()
            .at_position(Vec2::ZERO)
            .definition(&test_bolt_definition())
            .with_velocity(Velocity2D(Vec2::ZERO))
            .primary()
            .spawn(app.world_mut());
        app.world_mut()
            .entity_mut(entity)
            .insert(NodeScalingFactor(0.7));

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        let expected_val = 8.0 * 0.7;
        assert!(
            (scale.x - expected_val).abs() < TOLERANCE
                && (scale.y - expected_val).abs() < TOLERANCE,
            "Scale2D should be ({expected_val}, {expected_val}), got ({}, {})",
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn bolt_scale_visual_without_entity_scale_defaults_to_radius() {
        // Edge case: no NodeScalingFactor -> Scale2D { x: 8.0, y: 8.0 }
        let mut app = test_app();

        let entity = Bolt::builder()
            .at_position(Vec2::ZERO)
            .definition(&test_bolt_definition())
            .with_velocity(Velocity2D(Vec2::ZERO))
            .primary()
            .spawn(app.world_mut());

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 8.0).abs() < TOLERANCE && (scale.y - 8.0).abs() < TOLERANCE,
            "without NodeScalingFactor, Scale2D should be (8.0, 8.0), got ({}, {})",
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn bolt_scale_visual_entity_scale_one_equals_radius() {
        // Edge case: NodeScalingFactor(1.0) -> Scale2D { x: 8.0, y: 8.0 }
        let mut app = test_app();

        let entity = Bolt::builder()
            .at_position(Vec2::ZERO)
            .definition(&test_bolt_definition())
            .with_velocity(Velocity2D(Vec2::ZERO))
            .primary()
            .spawn(app.world_mut());
        app.world_mut()
            .entity_mut(entity)
            .insert(NodeScalingFactor(1.0));

        tick(&mut app);

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 8.0).abs() < TOLERANCE && (scale.y - 8.0).abs() < TOLERANCE,
            "with NodeScalingFactor(1.0), Scale2D should be (8.0, 8.0), got ({}, {})",
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn bolt_scale_visual_scales_multiple_bolts_independently() {
        // Given: Bolt A with NodeScalingFactor(0.7), Bolt B with NodeScalingFactor(0.5)
        // When: bolt_scale_visual runs
        // Then: A = Scale2D { x: 5.6, y: 5.6 }, B = Scale2D { x: 4.0, y: 4.0 }
        let mut app = test_app();

        let bolt_a = Bolt::builder()
            .at_position(Vec2::ZERO)
            .definition(&test_bolt_definition())
            .with_velocity(Velocity2D(Vec2::ZERO))
            .primary()
            .spawn(app.world_mut());
        app.world_mut()
            .entity_mut(bolt_a)
            .insert(NodeScalingFactor(0.7));

        let bolt_b = Bolt::builder()
            .at_position(Vec2::ZERO)
            .definition(&test_bolt_definition())
            .with_velocity(Velocity2D(Vec2::ZERO))
            .extra()
            .spawn(app.world_mut());
        app.world_mut()
            .entity_mut(bolt_b)
            .insert(NodeScalingFactor(0.5));

        tick(&mut app);

        let scale_a = app.world().get::<Scale2D>(bolt_a).unwrap();
        let expected_a = 8.0 * 0.7;
        assert!(
            (scale_a.x - expected_a).abs() < TOLERANCE
                && (scale_a.y - expected_a).abs() < TOLERANCE,
            "bolt A Scale2D should be ({expected_a}, {expected_a}), got ({}, {})",
            scale_a.x,
            scale_a.y,
        );

        let scale_b = app.world().get::<Scale2D>(bolt_b).unwrap();
        let expected_b = 8.0 * 0.5;
        assert!(
            (scale_b.x - expected_b).abs() < TOLERANCE
                && (scale_b.y - expected_b).abs() < TOLERANCE,
            "bolt B Scale2D should be ({expected_b}, {expected_b}), got ({}, {})",
            scale_b.x,
            scale_b.y,
        );
    }
}
