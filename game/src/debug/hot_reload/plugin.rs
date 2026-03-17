//! Hot-reload plugin — watches RON files and propagates changes to live game state.

use bevy::prelude::*;

use super::sets::HotReloadSystems;
use crate::shared::GameState;

/// Plugin that enables live hot-reload of RON configuration and content files.
///
/// Propagation runs in two ordered layers during `Update` while `GameState::Playing`:
/// 1. `PropagateDefaults` — asset changes re-seed Config resources
/// 2. `PropagateConfig` — Config changes force-overwrite entity components
pub struct HotReloadPlugin;

impl Plugin for HotReloadPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                HotReloadSystems::PropagateDefaults
                    .run_if(in_state(GameState::Playing)),
                HotReloadSystems::PropagateConfig
                    .after(HotReloadSystems::PropagateDefaults)
                    .run_if(in_state(GameState::Playing)),
            ),
        );
    }
}
