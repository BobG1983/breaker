//! Bolt plugin registration.

use bevy::prelude::*;

use crate::bolt::resources::BoltConfig;
use crate::bolt::systems::{
    apply_bump_velocity, hover_bolt, launch_bolt, prepare_bolt_velocity, spawn_bolt,
};
use crate::breaker::systems::move_breaker;
use crate::shared::{GameState, PlayingState};

/// Plugin for the bolt domain.
///
/// Owns bolt components, velocity, and speed management.
pub struct BoltPlugin;

impl Plugin for BoltPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoltConfig>();
        app.add_systems(OnEnter(GameState::Playing), spawn_bolt);
        app.add_systems(
            FixedUpdate,
            (
                launch_bolt,
                hover_bolt.after(move_breaker),
                prepare_bolt_velocity.after(move_breaker),
                apply_bump_velocity,
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
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.add_sub_state::<PlayingState>();
        // BoltPlugin reads BumpPerformed messages from breaker domain
        app.add_message::<crate::breaker::messages::BumpPerformed>();
        app.add_plugins(BoltPlugin);
        app.update();
    }
}
