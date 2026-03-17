//! Hot-reload system sets for ordering propagation layers.

use bevy::prelude::*;

/// System sets for hot-reload propagation ordering.
///
/// `PropagateDefaults` runs first (RON asset → Config resource),
/// then `PropagateConfig` (Config resource → entity components).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum HotReloadSystems {
    /// Defaults asset changed → re-seed Config resource.
    PropagateDefaults,
    /// Config resource changed → force-overwrite entity components.
    PropagateConfig,
}
