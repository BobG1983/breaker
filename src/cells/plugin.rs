//! Cells plugin registration.

use bevy::prelude::*;

use crate::{
    cells::{
        messages::CellDestroyed,
        resources::CellConfig,
        systems::{handle_cell_hit, spawn_cells},
    },
    shared::{GameState, PlayingState},
};

/// Plugin for the cells domain.
///
/// Owns cell components, grid layout, and destruction logic.
pub struct CellsPlugin;

impl Plugin for CellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CellDestroyed>();
        app.init_resource::<CellConfig>();
        app.add_systems(OnEnter(GameState::Playing), spawn_cells);
        app.add_systems(
            FixedUpdate,
            handle_cell_hit.run_if(in_state(PlayingState::Active)),
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
        app.init_resource::<crate::shared::PlayfieldConfig>();
        // CellsPlugin reads BoltHitCell messages from physics domain
        app.add_message::<crate::physics::messages::BoltHitCell>();
        app.add_plugins(CellsPlugin);
        app.update();
    }
}
