//! Chip selection screen plugin registration.

use bevy::{ecs::schedule::ApplyDeferred, prelude::*};

use super::{
    ChipSelectScreen,
    systems::{
        generate_chip_offerings, handle_chip_input, spawn_chip_select, tick_chip_timer,
        update_chip_display,
    },
};
use crate::shared::GameState;

/// Plugin for the between-node chip selection screen.
pub(crate) struct ChipSelectPlugin;

impl Plugin for ChipSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::ChipSelect),
            (generate_chip_offerings, ApplyDeferred, spawn_chip_select).chain(),
        )
        .add_systems(
            Update,
            (handle_chip_input, tick_chip_timer, update_chip_display)
                .chain()
                .run_if(in_state(GameState::ChipSelect)),
        )
        .add_systems(
            OnExit(GameState::ChipSelect),
            crate::screen::systems::cleanup_entities::<ChipSelectScreen>,
        );
    }
}
