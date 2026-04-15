//! Node timer threshold checking system.
//!
//! Checks the node timer ratio against registered thresholds each frame
//! and sends [`NodeTimerThresholdCrossed`] when a new threshold is crossed.

use bevy::prelude::*;

use super::{messages::NodeTimerThresholdCrossed, resources::NodeTimerThresholdRegistry};
use crate::prelude::*;

/// Checks the node timer ratio against registered thresholds.
///
/// When the ratio crosses a threshold in the registry that hasn't already
/// fired, sends [`NodeTimerThresholdCrossed`] and marks the threshold as fired.
pub fn check_node_timer_thresholds(
    timer: Option<Res<NodeTimer>>,
    mut registry: ResMut<NodeTimerThresholdRegistry>,
    mut writer: MessageWriter<NodeTimerThresholdCrossed>,
) {
    let Some(timer) = timer else {
        return;
    };
    if timer.total <= 0.0 {
        return;
    }

    // Ratio is elapsed / total (0.0 at start, approaches 1.0 at end)
    let elapsed = timer.total - timer.remaining;
    let ratio = (elapsed / timer.total).clamp(0.0, 1.0);

    let newly_crossed: Vec<_> = registry
        .thresholds
        .iter()
        .copied()
        .filter(|t| ratio >= **t && !registry.fired.contains(t))
        .collect();

    for threshold in newly_crossed {
        registry.fired.insert(threshold);
        writer.write(NodeTimerThresholdCrossed { ratio: threshold });
    }
}
