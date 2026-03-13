//! Bolt plugin registration.

use bevy::prelude::*;

use crate::{
    bolt::{
        BoltSystems,
        resources::BoltConfig,
        systems::{
            animate_fade_out, apply_bump_velocity, hover_bolt, init_bolt_params, launch_bolt,
            prepare_bolt_velocity, spawn_bolt, spawn_bolt_lost_text,
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
        app.init_resource::<BoltConfig>();
        app.add_systems(
            OnEnter(GameState::Playing),
            (spawn_bolt, init_bolt_params.after(spawn_bolt)),
        );
        app.add_systems(
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
                spawn_bolt_lost_text,
            )
                .run_if(in_state(PlayingState::Active)),
        );
        app.add_systems(
            Update,
            animate_fade_out.run_if(in_state(PlayingState::Active)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.add_sub_state::<PlayingState>();
        // InputPlugin owns InputActions
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_message::<bevy::input::keyboard::KeyboardInput>();
        app.add_plugins(crate::input::InputPlugin);
        // BoltPlugin reads BumpPerformed messages from breaker domain
        // and BoltLost messages from physics domain
        app.add_message::<crate::breaker::messages::BumpPerformed>();
        app.add_message::<crate::physics::messages::BoltLost>();
        app.add_plugins(BoltPlugin);
        app.update();
    }
}
