//! Physics plugin registration.

use bevy::prelude::*;

use crate::{
    bolt::BoltSystems,
    physics::{
        messages::{BoltHitBreaker, BoltHitCell, BoltLost},
        systems::{bolt_breaker_collision, bolt_cell_collision, bolt_lost},
    },
    shared::PlayingState,
};

/// Plugin for the physics domain.
///
/// Owns collision detection and collision response systems.
/// Runs CCD collision in `FixedUpdate`.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BoltHitBreaker>()
            .add_message::<BoltHitCell>()
            .add_message::<BoltLost>()
            .add_systems(
                FixedUpdate,
                (
                    bolt_cell_collision.after(BoltSystems::PrepareVelocity),
                    bolt_breaker_collision
                        .after(bolt_cell_collision)
                        .in_set(super::PhysicsSystems::BreakerCollision),
                    bolt_lost
                        .after(bolt_breaker_collision)
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
