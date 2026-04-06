//! Fx plugin registration.

use bevy::prelude::*;

use crate::{
    fx::systems::{animate_fade_out, animate_punch_scale, tick_effect_flash},
    state::types::NodeState,
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
            (animate_fade_out, animate_punch_scale, tick_effect_flash)
                .run_if(in_state(NodeState::Playing)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::types::{AppState, GameState, RunState};

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<RunState>()
            .add_sub_state::<NodeState>()
            .add_plugins(FxPlugin)
            .update();
    }
}
