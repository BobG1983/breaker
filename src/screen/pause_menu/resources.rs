//! Pause menu resources.

use bevy::prelude::*;

use super::components::PauseMenuItem;

/// Resource tracking the currently selected pause menu item.
#[derive(Resource, Debug)]
pub struct PauseMenuSelection {
    /// The currently highlighted menu item.
    pub selected: PauseMenuItem,
}
