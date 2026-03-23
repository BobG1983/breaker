//! Cells plugin registration.

use bevy::prelude::*;

use crate::{
    cells::{
        messages::{CellDestroyed, DamageCell},
        resources::CellConfig,
        systems::{
            check_lock_release::check_lock_release, handle_cell_hit,
            tick_cell_regen::tick_cell_regen,
        },
    },
    shared::PlayingState,
};

/// Plugin for the cells domain.
///
/// Owns cell components, damage handling, and destruction logic.
pub(crate) struct CellsPlugin;

impl Plugin for CellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CellDestroyed>()
            .add_message::<DamageCell>()
            .init_resource::<CellConfig>()
            .add_systems(
                FixedUpdate,
                (
                    handle_cell_hit,
                    check_lock_release.after(handle_cell_hit),
                    tick_cell_regen,
                )
                    .run_if(in_state(PlayingState::Active)),
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
            // CellsPlugin reads BoltHitCell messages from bolt domain
            .add_message::<crate::bolt::messages::BoltHitCell>()
            .add_plugins(CellsPlugin)
            .update();
    }
}
