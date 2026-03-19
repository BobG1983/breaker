//! Physics plugin registration.

use bevy::prelude::*;

use crate::{
    bolt::BoltSystems,
    physics::{
        messages::{BoltHitBreaker, BoltHitCell, BoltLost},
        systems::{
            bolt_breaker_collision, bolt_cell_collision, bolt_lost, clamp_bolt_to_playfield,
        },
    },
    shared::{GameRng, PlayingState},
};

/// Plugin for the physics domain.
///
/// Owns collision detection and collision response systems.
/// Runs CCD collision in `FixedUpdate`.
pub(crate) struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameRng>()
            .add_message::<BoltHitBreaker>()
            .add_message::<BoltHitCell>()
            .add_message::<BoltLost>()
            .add_systems(
                FixedUpdate,
                (
                    bolt_cell_collision.after(BoltSystems::PrepareVelocity),
                    bolt_breaker_collision
                        .after(bolt_cell_collision)
                        .in_set(super::PhysicsSystems::BreakerCollision),
                    clamp_bolt_to_playfield.after(bolt_breaker_collision),
                    bolt_lost
                        .after(clamp_bolt_to_playfield)
                        .in_set(super::PhysicsSystems::BoltLost),
                )
                    .run_if(in_state(PlayingState::Active)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<crate::shared::GameState>()
            .add_sub_state::<PlayingState>()
            .init_resource::<crate::shared::PlayfieldConfig>()
            .add_plugins(PhysicsPlugin)
            .update();
    }
}
