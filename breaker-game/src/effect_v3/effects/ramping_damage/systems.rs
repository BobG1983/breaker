//! Ramping damage systems — reset accumulator on node start.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::components::RampingDamageAccumulator;

/// Resets all `RampingDamageAccumulator` components to zero at the start of each node.
pub fn reset_ramping_damage(mut query: Query<&mut RampingDamageAccumulator>) {
    for mut acc in &mut query {
        acc.0 = OrderedFloat(0.0);
    }
}
