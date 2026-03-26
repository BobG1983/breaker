//! Fx plugin registration.

use bevy::prelude::*;

use crate::{
    fx::{
        systems::{animate_fade_out, animate_punch_scale},
        transition::{
            animate_transition, cleanup_transition, spawn_transition_in, spawn_transition_out,
        },
    },
    shared::{GameState, PlayingState},
};

/// Plugin for the fx domain.
///
/// Owns cross-cutting visual effects: fade-out animations, transition overlays,
/// and (future) screen shake and particles.
pub(crate) struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (animate_fade_out, animate_punch_scale).run_if(in_state(PlayingState::Active)),
        )
        .add_systems(OnEnter(GameState::TransitionOut), spawn_transition_out)
        .add_systems(OnEnter(GameState::TransitionIn), spawn_transition_in)
        .add_systems(
            Update,
            animate_transition
                .run_if(in_state(GameState::TransitionOut).or(in_state(GameState::TransitionIn))),
        )
        .add_systems(OnExit(GameState::TransitionOut), cleanup_transition)
        .add_systems(OnExit(GameState::TransitionIn), cleanup_transition);
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
            .add_plugins(FxPlugin)
            .update();
    }
}
