//! Bolt plugin registration.

use bevy::prelude::*;

use crate::{
    behaviors::BehaviorSystems,
    bolt::{
        BoltSystems,
        messages::{BoltHitBreaker, BoltHitCell, BoltHitWall, BoltLost, SpawnAdditionalBolt},
        resources::BoltConfig,
        systems::{
            apply_entity_scale_to_bolt, bolt_breaker_collision, bolt_cell_collision, bolt_lost,
            bolt_scale_visual, clamp_bolt_to_playfield, hover_bolt, init_bolt_params, launch_bolt,
            prepare_bolt_velocity, reset_bolt, spawn_additional_bolt, spawn_bolt,
            spawn_bolt_lost_text,
        },
    },
    breaker::BreakerSystems,
    run::node::sets::NodeSystems,
    shared::{GameRng, GameState, PlayingState},
};

/// Plugin for the bolt domain.
///
/// Owns bolt components, velocity, speed management, and collision detection.
pub struct BoltPlugin;

impl Plugin for BoltPlugin {
    fn build(&self, app: &mut App) {
        use crate::bolt::messages::BoltSpawned;
        app.init_resource::<BoltConfig>()
            .init_resource::<GameRng>()
            .add_message::<SpawnAdditionalBolt>()
            .add_message::<BoltSpawned>()
            .add_message::<BoltHitBreaker>()
            .add_message::<BoltHitCell>()
            .add_message::<BoltLost>()
            .add_message::<BoltHitWall>()
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    spawn_bolt,
                    init_bolt_params
                        .after(spawn_bolt)
                        .in_set(BoltSystems::InitParams),
                    apply_entity_scale_to_bolt
                        .after(BoltSystems::InitParams)
                        .after(NodeSystems::Spawn),
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
                    spawn_additional_bolt.after(BehaviorSystems::Bridge),
                    spawn_bolt_lost_text,
                    // Collision systems (moved from PhysicsPlugin)
                    bolt_cell_collision.after(BoltSystems::PrepareVelocity),
                    bolt_breaker_collision
                        .after(bolt_cell_collision)
                        .in_set(BoltSystems::BreakerCollision),
                    clamp_bolt_to_playfield.after(bolt_breaker_collision),
                    bolt_lost
                        .after(clamp_bolt_to_playfield)
                        .in_set(BoltSystems::BoltLost),
                )
                    .run_if(in_state(PlayingState::Active)),
            )
            .add_systems(
                Update,
                bolt_scale_visual.run_if(in_state(PlayingState::Active)),
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
            // BoltPlugin reads messages from cells domain
            .add_message::<crate::cells::messages::CellDestroyed>()
            .add_plugins(BoltPlugin)
            .update();
    }
}
