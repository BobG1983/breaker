//! Hot-reload plugin — watches RON files and propagates changes to live game state.

use bevy::prelude::*;

use super::{
    sets::HotReloadSystems,
    systems::{
        propagate_archetype_changes, propagate_bolt_config, propagate_bolt_defaults,
        propagate_breaker_config, propagate_breaker_defaults, propagate_cell_defaults,
        propagate_cell_type_changes, propagate_chip_select_defaults, propagate_input_defaults,
        propagate_main_menu_defaults, propagate_node_layout_changes, propagate_playfield_defaults,
        propagate_timer_ui_defaults,
    },
};
use crate::{bolt::BoltConfig, breaker::BreakerConfig, shared::GameState};

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
                HotReloadSystems::PropagateDefaults.run_if(in_state(GameState::Playing)),
                HotReloadSystems::PropagateConfig
                    .after(HotReloadSystems::PropagateDefaults)
                    .run_if(in_state(GameState::Playing)),
            ),
        );

        // Layer 2: Defaults → Config (PropagateDefaults set)
        app.add_systems(
            Update,
            (
                propagate_bolt_defaults,
                propagate_breaker_defaults,
                propagate_cell_defaults,
                propagate_playfield_defaults,
                propagate_input_defaults,
                propagate_timer_ui_defaults,
                propagate_main_menu_defaults,
                propagate_chip_select_defaults,
                propagate_cell_type_changes,
                propagate_node_layout_changes,
                propagate_archetype_changes,
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
