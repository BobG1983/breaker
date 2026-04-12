//! `ComboActive` condition evaluator.

use bevy::prelude::*;

use crate::state::run::resources::HighlightTracker;

/// Evaluate whether the `ComboActive(threshold)` condition is currently true.
///
/// Returns true while the consecutive perfect bump streak is at or above
/// the given count.
pub fn is_combo_active(world: &World, threshold: u32) -> bool {
    world
        .get_resource::<HighlightTracker>()
        .is_some_and(|tracker| tracker.consecutive_perfect_bumps >= threshold)
}
