//! Wall domain components.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Spatial2D;

use crate::shared::CleanupOnNodeExit;

/// Marker component identifying wall entities (left, right, ceiling).
#[derive(Component, Debug, Default)]
#[require(Spatial2D, CleanupOnNodeExit)]
pub(crate) struct Wall;

/// Half-extents for a wall entity used in CCD collision.
///
/// Walls are invisible collision boundaries, so they carry their own size
/// rather than relying on cell config.
#[derive(Component, Debug)]
pub(crate) struct WallSize {
    /// Half-width in world units.
    pub half_width: f32,
    /// Half-height in world units.
    pub half_height: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Wall #[require] tests ────────────────────────────────────

    #[test]
    fn wall_require_inserts_spatial2d() {
        use rantzsoft_spatial2d::components::Spatial2D;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(Wall).id();
        app.update();
        assert!(
            app.world().get::<Spatial2D>(entity).is_some(),
            "Wall should auto-insert Spatial2D via #[require]"
        );
    }

    #[test]
    fn wall_require_inserts_cleanup_on_node_exit() {
        use crate::shared::CleanupOnNodeExit;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(Wall).id();
        app.update();
        assert!(
            app.world().get::<CleanupOnNodeExit>(entity).is_some(),
            "Wall should auto-insert CleanupOnNodeExit via #[require]"
        );
    }

    #[test]
    fn wall_require_does_not_insert_interpolate_transform2d() {
        use rantzsoft_spatial2d::components::InterpolateTransform2D;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(Wall).id();
        app.update();
        assert!(
            app.world().get::<InterpolateTransform2D>(entity).is_none(),
            "Wall #[require] should NOT auto-insert InterpolateTransform2D (walls are static)"
        );
    }

    #[test]
    fn wall_explicit_values_override_defaults() {
        use rantzsoft_spatial2d::components::{Position2D, Scale2D, Spatial2D};
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app
            .world_mut()
            .spawn((
                Wall,
                Spatial2D,
                Position2D(Vec2::new(-490.0, 0.0)),
                Scale2D { x: 90.0, y: 300.0 },
            ))
            .id();
        app.update();
        let position = app
            .world()
            .get::<Position2D>(entity)
            .expect("Position2D should be present");
        assert_eq!(
            position.0,
            Vec2::new(-490.0, 0.0),
            "explicit Position2D(-490.0, 0.0) should be preserved"
        );
        let scale = app
            .world()
            .get::<Scale2D>(entity)
            .expect("Scale2D should be present");
        assert!(
            (scale.x - 90.0).abs() < f32::EPSILON && (scale.y - 300.0).abs() < f32::EPSILON,
            "explicit Scale2D {{ x: 90.0, y: 300.0 }} should be preserved, got {{ x: {}, y: {} }}",
            scale.x,
            scale.y
        );
    }
}
