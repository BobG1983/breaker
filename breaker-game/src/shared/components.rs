//! Shared components used across multiple domain plugins.

use bevy::prelude::*;

/// Scale factor applied to breaker and bolt dimensions per layout.
///
/// Set at node entry from [`ActiveNodeLayout`]. Multiplies visual size and
/// collision hitboxes — speed is unaffected. Defaults to 1.0 (no scaling).
#[derive(Component, Debug, Clone, Copy)]
pub struct NodeScalingFactor(pub f32);

/// Marker component for entities that should be despawned when exiting a node.
///
/// Added to bolt, cells, and other node-scoped entities. Node exit is modeled
/// as exiting [`GameState::Playing`] — any new transitions out of `Playing`
/// must account for the fact that all `CleanupOnNodeExit` entities will be
/// despawned.
#[derive(Component, Default)]
pub struct CleanupOnNodeExit;

/// Marker component for entities that should be despawned when a run ends.
///
/// Added to breaker, run-scoped chips, and accumulated state.
#[derive(Component)]
pub struct CleanupOnRunEnd;
