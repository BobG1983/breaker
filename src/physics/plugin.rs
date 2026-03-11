//! Physics plugin registration.

use bevy::prelude::*;

use crate::{
    bolt::systems::prepare_bolt_velocity,
    physics::{
        messages::{BoltHitBreaker, BoltHitCell, BoltLost},
        resources::PhysicsConfig,
        systems::{bolt_breaker_collision, bolt_cell_collision, bolt_lost, spawn_walls},
    },
    shared::{GameState, PlayingState},
};

/// Plugin for the physics domain.
///
/// Owns collision detection and collision response systems.
/// Spawns wall entities on node entry and runs CCD collision in `FixedUpdate`.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsConfig>();
        app.add_message::<BoltHitBreaker>();
        app.add_message::<BoltHitCell>();
        app.add_message::<BoltLost>();

        app.add_systems(OnEnter(GameState::Playing), spawn_walls);

        app.add_systems(
            FixedUpdate,
            (
                bolt_cell_collision.after(prepare_bolt_velocity),
                bolt_breaker_collision.after(bolt_cell_collision),
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
