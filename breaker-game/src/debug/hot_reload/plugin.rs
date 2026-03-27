//! Hot-reload plugin — watches RON files and propagates changes to live game state.

use bevy::prelude::*;

use super::{
    sets::HotReloadSystems,
    systems::{
        propagate_bolt_config, propagate_breaker_changes, propagate_breaker_config,
        propagate_cell_type_changes, propagate_node_layout_changes,
    },
};
use crate::{
    bolt::BoltConfig, breaker::BreakerConfig, chips::systems::propagate_chip_catalog,
    shared::GameState,
};

/// Plugin that enables live hot-reload of RON configuration and content files.
///
/// Simple config types (bolt, breaker, cells, input, etc.) are propagated
/// automatically by `rantzsoft_defaults::systems::propagate_defaults`. This
/// plugin handles:
/// 1. `PropagateDefaults` — registry rebuilds and content-type changes
/// 2. `PropagateConfig` — `Config` changes force-overwrite entity components
pub(crate) struct HotReloadPlugin;

impl Plugin for HotReloadPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                HotReloadSystems::PropagateDefaults.run_if(in_state(GameState::Playing)),
                HotReloadSystems::PropagateConfig
                    .after(HotReloadSystems::PropagateDefaults)
                    .run_if(in_state(GameState::Playing)),
            ),
        );

        // Layer 2: Content/registry changes (PropagateDefaults set)
        app.add_systems(
            Update,
            (
                propagate_cell_type_changes.before(propagate_node_layout_changes),
                propagate_node_layout_changes,
                propagate_breaker_changes,
                propagate_chip_catalog,
            )
                .in_set(HotReloadSystems::PropagateDefaults),
        );

        // Layer 3: Config → Components (PropagateConfig set)
        app.add_systems(
            Update,
            (
                propagate_bolt_config.run_if(resource_changed::<BoltConfig>),
                propagate_breaker_config.run_if(resource_changed::<BreakerConfig>),
            )
                .in_set(HotReloadSystems::PropagateConfig),
        );
    }
}
