//! Bolt plugin registration.

use bevy::prelude::*;

use crate::{
    behaviors::BehaviorSystems,
    bolt::{
        BoltSystems,
        behaviors::BoltBehaviorsPlugin,
        messages::SpawnAdditionalBolt,
        resources::BoltConfig,
        systems::{
            apply_bump_velocity, hover_bolt, init_bolt_params, launch_bolt, prepare_bolt_velocity,
            reset_bolt, spawn_additional_bolt, spawn_bolt, spawn_bolt_lost_text,
        },
    },
    breaker::BreakerSystems,
    physics::PhysicsSystems,
    shared::{GameState, PlayingState},
};

/// Plugin for the bolt domain.
///
/// Owns bolt components, velocity, and speed management.
pub struct BoltPlugin;

impl Plugin for BoltPlugin {
    fn build(&self, app: &mut App) {
        use crate::bolt::messages::BoltSpawned;
        app.add_plugins(BoltBehaviorsPlugin)
            .init_resource::<BoltConfig>()
            .add_message::<SpawnAdditionalBolt>()
            .add_message::<BoltSpawned>()
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    spawn_bolt,
                    init_bolt_params
                        .after(spawn_bolt)
                        .in_set(BoltSystems::InitParams),
                    reset_bolt
                        .after(BoltSystems::InitParams)
                        .after(BreakerSystems::Reset)
                        .in_set(BoltSystems::Reset),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    launch_bolt,
                    (
                        hover_bolt,
                        prepare_bolt_velocity.in_set(BoltSystems::PrepareVelocity),
                    )
                        .after(BreakerSystems::Move),
                    apply_bump_velocity
                        .after(PhysicsSystems::BreakerCollision)
                        .before(PhysicsSystems::BoltLost),
                    spawn_additional_bolt.after(BehaviorSystems::Bridge),
                    spawn_bolt_lost_text,
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
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            // InputPlugin owns InputActions
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_plugins(crate::input::InputPlugin)
            // BoltPlugin reads messages from breaker, physics, and cells domains
            .add_message::<crate::breaker::messages::BumpPerformed>()
            .add_message::<crate::physics::messages::BoltHitCell>()
            .add_message::<crate::physics::messages::BoltLost>()
            .add_message::<crate::cells::messages::CellDestroyed>()
            .add_plugins(BoltPlugin)
            .update();
    }
}
