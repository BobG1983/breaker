//! Breaker core components.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{InterpolateTransform2D, Spatial2D};

/// Marker component identifying the breaker entity.
#[derive(Component, Debug, Default)]
#[require(Spatial2D, InterpolateTransform2D)]
pub struct Breaker;

/// Full width of the breaker in world units.
#[derive(Component, Debug)]
pub struct BreakerWidth(pub f32);

impl BreakerWidth {
    /// Returns half the breaker width.
    #[must_use]
    pub fn half_width(&self) -> f32 {
        self.0 / 2.0
    }
}

/// Full height of the breaker in world units.
#[derive(Component, Debug)]
pub struct BreakerHeight(pub f32);

impl BreakerHeight {
    /// Returns half the breaker height.
    #[must_use]
    pub fn half_height(&self) -> f32 {
        self.0 / 2.0
    }
}

/// Y position of the breaker at rest.
#[derive(Component, Debug)]
pub struct BreakerBaseY(pub f32);

/// Maximum reflection angle from vertical in radians.
#[derive(Component, Debug)]
pub struct MaxReflectionAngle(pub f32);

/// Marker: breaker entity has been initialized by `init_breaker`.
/// Prevents duplicate chain pushes on node re-entry.
#[derive(Component)]
pub struct BreakerInitialized;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaker_width_half_width() {
        let w = BreakerWidth(120.0);
        assert!((w.half_width() - 60.0).abs() < f32::EPSILON);
    }

    #[test]
    fn breaker_height_half_height() {
        let h = BreakerHeight(20.0);
        assert!((h.half_height() - 10.0).abs() < f32::EPSILON);
    }

    // ── Breaker #[require] tests ─────────────────────────────────

    #[test]
    fn breaker_require_inserts_spatial2d() {
        use rantzsoft_spatial2d::components::Spatial2D;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(Breaker).id();
        app.update();
        assert!(
            app.world().get::<Spatial2D>(entity).is_some(),
            "Breaker should auto-insert Spatial2D via #[require]"
        );
    }

    #[test]
    fn breaker_require_inserts_interpolate_transform2d() {
        use rantzsoft_spatial2d::components::InterpolateTransform2D;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(Breaker).id();
        app.update();
        assert!(
            app.world().get::<InterpolateTransform2D>(entity).is_some(),
            "Breaker should auto-insert InterpolateTransform2D via #[require]"
        );
    }

    #[test]
    fn breaker_require_does_not_insert_cleanup_on_run_end() {
        use crate::shared::CleanupOnRunEnd;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(Breaker).id();
        app.update();
        assert!(
            app.world().get::<CleanupOnRunEnd>(entity).is_none(),
            "Breaker #[require] should NOT auto-insert CleanupOnRunEnd"
        );
    }

    #[test]
    fn breaker_require_does_not_insert_cleanup_on_node_exit() {
        use crate::shared::CleanupOnNodeExit;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(Breaker).id();
        app.update();
        assert!(
            app.world().get::<CleanupOnNodeExit>(entity).is_none(),
            "Breaker #[require] should NOT auto-insert CleanupOnNodeExit"
        );
    }

    // ── CollisionLayers tests ──────────────────────────────────────

    #[test]
    fn breaker_collision_layers_have_correct_values() {
        use rantzsoft_physics2d::collision_layers::CollisionLayers;

        use crate::shared::{BOLT_LAYER, BREAKER_LAYER};
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app
            .world_mut()
            .spawn((Breaker, CollisionLayers::new(BREAKER_LAYER, BOLT_LAYER)))
            .id();
        app.update();
        let layers = app
            .world()
            .get::<CollisionLayers>(entity)
            .expect("Breaker should have CollisionLayers");
        assert_eq!(
            layers.membership, BREAKER_LAYER,
            "Breaker membership should be BREAKER_LAYER (0x{BREAKER_LAYER:02X}), got 0x{:02X}",
            layers.membership
        );
        assert_eq!(
            layers.mask, BOLT_LAYER,
            "Breaker mask should be BOLT_LAYER (0x{BOLT_LAYER:02X}), got 0x{:02X}",
            layers.mask
        );
    }
}
