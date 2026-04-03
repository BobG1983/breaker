//! HUD plugin registration.

use bevy::{ecs::schedule::ApplyDeferred, prelude::*};

use super::{
    UiSystems,
    systems::{spawn_side_panels, spawn_timer_hud, update_timer_display},
};
use crate::{
    shared::{GameState, PlayingState},
    state::run::chip_select::messages::ChipSelected,
};

/// Plugin for the HUD — timer display, side panels, status panel.
pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
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
            .add_plugins(HudPlugin)
            .update();
    }
}
