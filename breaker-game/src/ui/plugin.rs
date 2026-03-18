//! UI plugin registration.

use bevy::{ecs::schedule::ApplyDeferred, prelude::*};

use crate::{
    shared::{GameState, PlayingState},
    ui::{
        UiSystems,
        messages::ChipSelected,
        systems::{spawn_side_panels, spawn_timer_hud, update_timer_display},
    },
};

/// Plugin for the UI domain.
///
/// Owns HUD rendering, menu screens, and chip selection.
pub(crate) struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ChipSelected>()
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    spawn_side_panels,
                    ApplyDeferred,
                    spawn_timer_hud.in_set(UiSystems::SpawnTimerHud),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                update_timer_display.run_if(in_state(PlayingState::Active)),
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
            .add_plugins(UiPlugin)
            .update();
    }
}
