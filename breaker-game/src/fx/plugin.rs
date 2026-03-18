//! Fx plugin registration.

use bevy::prelude::*;

use crate::{fx::systems::animate_fade_out, shared::PlayingState};

/// Plugin for the fx domain.
///
/// Owns cross-cutting visual effects: fade-out animations, and (future) flash,
/// screen shake, and particles.
pub(crate) struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            animate_fade_out.run_if(in_state(PlayingState::Active)),
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
            .add_plugins(FxPlugin)
            .update();
    }
}
