//! Hot-reload system sets for ordering propagation layers.

use bevy::prelude::*;

/// System sets for hot-reload propagation ordering.
///
/// `PropagateDefaults` runs when registry or content changes need to be
/// propagated to live game state.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum HotReloadSystems {
    /// Defaults asset changed → re-seed Config resource / re-stamp entity components.
    PropagateDefaults,
}
