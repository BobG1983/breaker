//! Shared components used across multiple domain plugins.

use bevy::prelude::*;

/// Full width of an entity in world units.
#[derive(Component, Debug, Clone, Copy)]
pub struct BaseWidth(pub f32);

impl BaseWidth {
    /// Returns half the width.
    #[must_use]
    pub fn half_width(&self) -> f32 {
        self.0 / 2.0
    }
}

/// Full height of an entity in world units.
#[derive(Component, Debug, Clone, Copy)]
pub struct BaseHeight(pub f32);

impl BaseHeight {
    /// Returns half the height.
    #[must_use]
    pub fn half_height(&self) -> f32 {
        self.0 / 2.0
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_width_half_width() {
        let w = BaseWidth(120.0);
        assert!((w.half_width() - 60.0).abs() < f32::EPSILON);
    }

    #[test]
    fn base_height_half_height() {
        let h = BaseHeight(20.0);
        assert!((h.half_height() - 10.0).abs() < f32::EPSILON);
    }
}
