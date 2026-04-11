//! `ComboActive` condition evaluator.

use bevy::prelude::*;

/// Evaluate whether the `ComboActive(threshold)` condition is currently true.
///
/// Returns true while the consecutive perfect bump streak is at or above
/// the given count.
pub fn is_combo_active(_world: &World, _threshold: u32) -> bool {
    todo!()
}
