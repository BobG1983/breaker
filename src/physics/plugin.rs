//! Physics plugin registration.

use bevy::prelude::*;

use crate::bolt::systems::prepare_bolt_velocity;
use crate::physics::messages::{BoltHitBreaker, BoltHitCell, BoltLost};
use crate::physics::resources::PhysicsConfig;
use crate::physics::systems::{
    bolt_breaker_collision, bolt_cell_collision, bolt_lost, wall_collision,
};
use crate::shared::PlayingState;

/// Plugin for the physics domain.
///
/// Owns collision detection, quadtree, and collision response systems.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsConfig>();
        app.add_message::<BoltHitBreaker>();
        app.add_message::<BoltHitCell>();
        app.add_message::<BoltLost>();

        app.add_systems(
            FixedUpdate,
            (
                bolt_cell_collision.after(prepare_bolt_velocity),
                wall_collision.after(bolt_cell_collision),
                bolt_breaker_collision.after(wall_collision),
                bolt_lost.after(bolt_breaker_collision),
            )
                .run_if(in_state(PlayingState::Active)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::GameState;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            .init_resource::<crate::shared::PlayfieldConfig>()
            .add_plugins(PhysicsPlugin)
            .update();
    }
}
