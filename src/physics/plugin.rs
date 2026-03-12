//! Physics plugin registration.

use bevy::prelude::*;

use crate::{
    bolt::BoltSystems,
    physics::{
        messages::{BoltHitBreaker, BoltHitCell, BoltLost},
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
        app.add_message::<BoltHitBreaker>();
        app.add_message::<BoltHitCell>();
        app.add_message::<BoltLost>();

        app.add_systems(OnEnter(GameState::Playing), spawn_walls);

        app.add_systems(
            FixedUpdate,
            (
                bolt_cell_collision.after(BoltSystems::PrepareVelocity),
                bolt_breaker_collision
                    .after(bolt_cell_collision)
                    .in_set(super::PhysicsSystems::BreakerCollision),
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
