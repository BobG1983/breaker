//! Upgrade selection screen plugin registration.

use bevy::prelude::*;

use super::{
    UpgradeSelectScreen,
    systems::{
        handle_upgrade_input, spawn_upgrade_select, tick_upgrade_timer, update_upgrade_display,
    },
};
use crate::shared::GameState;

/// Plugin for the between-node upgrade selection screen.
pub struct UpgradeSelectPlugin;

impl Plugin for UpgradeSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::UpgradeSelect), spawn_upgrade_select)
            .add_systems(
                Update,
                (
                    handle_upgrade_input,
                    tick_upgrade_timer,
                    update_upgrade_display,
                )
                    .chain()
                    .run_if(in_state(GameState::UpgradeSelect)),
            )
            .add_systems(
                OnExit(GameState::UpgradeSelect),
                crate::screen::systems::cleanup_entities::<UpgradeSelectScreen>,
            );
    }
}
