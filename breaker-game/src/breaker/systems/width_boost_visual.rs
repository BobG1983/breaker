//! Visual system to apply [`WidthBoost`] to the breaker's mesh scale each frame.

use bevy::prelude::*;

use crate::{
    breaker::components::{Breaker, BreakerHeight, BreakerWidth},
    chips::components::WidthBoost,
};

/// Sets the breaker's [`Transform`] scale to reflect its effective width.
///
/// When [`WidthBoost`] is present, effective width = `BreakerWidth + WidthBoost`.
/// Without it, effective width equals `BreakerWidth`.
pub(crate) fn width_boost_visual(
    mut query: Query<
        (
            &BreakerWidth,
            Option<&WidthBoost>,
            &BreakerHeight,
            &mut Transform,
        ),
        With<Breaker>,
    >,
) {
    for (breaker_w, width_boost, breaker_h, mut transform) in &mut query {
        let effective_width = breaker_w.0 + width_boost.map_or(0.0, |b| b.0);
        transform.scale = Vec3::new(effective_width, breaker_h.0, 1.0);
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
