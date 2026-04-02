//! Hot-reload plugin — watches RON files and propagates changes to live game state.

use bevy::prelude::*;

use super::{
    sets::HotReloadSystems,
    systems::{
        propagate_bolt_definition, propagate_breaker_changes, propagate_cell_type_changes,
        propagate_node_layout_changes,
    },
};
use crate::{chips::systems::propagate_chip_catalog, shared::GameState};

/// Plugin that enables live hot-reload of RON configuration and content files.
///
/// Handles registry rebuilds and content-type changes in the
/// `PropagateDefaults` system set.
pub(crate) struct HotReloadPlugin;

impl Plugin for HotReloadPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            HotReloadSystems::PropagateDefaults.run_if(in_state(GameState::Playing)),
        );

        // Layer 2: Content/registry changes (PropagateDefaults set)
        app.add_systems(
            Update,
            (
                propagate_cell_type_changes.before(propagate_node_layout_changes),
                propagate_node_layout_changes,
                propagate_breaker_changes,
                propagate_bolt_definition,
                propagate_chip_catalog,
            )
                .in_set(HotReloadSystems::PropagateDefaults),
        );
    }
}
