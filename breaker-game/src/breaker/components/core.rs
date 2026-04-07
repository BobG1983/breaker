//! Breaker core components.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{InterpolateTransform2D, Spatial2D};

/// Marker component identifying the breaker entity.
#[derive(Component, Debug, Default)]
#[require(Spatial2D, InterpolateTransform2D)]
pub struct Breaker;

/// Y position of the breaker at rest.
#[derive(Component, Debug)]
pub struct BreakerBaseY(pub f32);

/// Maximum reflection angle from vertical in radians.
#[derive(Component, Debug)]
pub struct BreakerReflectionSpread(pub f32);

/// Marker: the primary breaker (persists across nodes, cleaned up on run end).
#[derive(Component, Debug)]
pub struct PrimaryBreaker;

/// Marker: an extra breaker (cleaned up on node exit).
#[derive(Component, Debug)]
pub struct ExtraBreaker;

/// Marker: breaker entity has been initialized by `init_breaker`.
/// Prevents duplicate chain pushes on node re-entry.
#[derive(Component)]
pub struct BreakerInitialized;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breaker::definition::BreakerDefinition;

    // ── Breaker #[require] tests ─────────────────────────────────

    #[test]
    fn breaker_require_inserts_spatial2d() {
        use rantzsoft_spatial2d::components::Spatial2D;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let def = BreakerDefinition::default();
        let entity = app
            .world_mut()
            .spawn(
                Breaker::builder()
                    .definition(&def)
                    .headless()
                    .primary()
                    .build(),
            )
            .id();
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
        let def = BreakerDefinition::default();
        let entity = app
            .world_mut()
            .spawn(
                Breaker::builder()
                    .definition(&def)
                    .headless()
                    .primary()
                    .build(),
            )
            .id();
        app.update();
        assert!(
            app.world().get::<InterpolateTransform2D>(entity).is_some(),
            "Breaker should auto-insert InterpolateTransform2D via #[require]"
        );
    }

    #[test]
    fn primary_builder_inserts_cleanup_on_exit_run_state() {
        use rantzsoft_stateflow::CleanupOnExit;

        use crate::state::types::RunState;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let def = BreakerDefinition::default();
        let entity = app
            .world_mut()
            .spawn(
                Breaker::builder()
                    .definition(&def)
                    .headless()
                    .primary()
                    .build(),
            )
            .id();
        app.update();
        assert!(
            app.world().get::<CleanupOnExit<RunState>>(entity).is_some(),
            "Primary builder should insert CleanupOnExit<RunState>"
        );
    }

    #[test]
    fn breaker_require_does_not_insert_cleanup_on_exit_node_state() {
        use rantzsoft_stateflow::CleanupOnExit;

        use crate::state::types::NodeState;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let def = BreakerDefinition::default();
        let entity = app
            .world_mut()
            .spawn(
                Breaker::builder()
                    .definition(&def)
                    .headless()
                    .primary()
                    .build(),
            )
            .id();
        app.update();
        assert!(
            app.world()
                .get::<CleanupOnExit<NodeState>>(entity)
                .is_none(),
            "Breaker #[require] should NOT auto-insert CleanupOnExit<NodeState>"
        );
    }

    // ── CollisionLayers tests ──────────────────────────────────────

    #[test]
    fn breaker_collision_layers_have_correct_values() {
        use rantzsoft_physics2d::collision_layers::CollisionLayers;

        use crate::shared::{BOLT_LAYER, BREAKER_LAYER};
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let def = BreakerDefinition::default();
        let entity = app
            .world_mut()
            .spawn(
                Breaker::builder()
                    .definition(&def)
                    .headless()
                    .primary()
                    .build(),
            )
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
