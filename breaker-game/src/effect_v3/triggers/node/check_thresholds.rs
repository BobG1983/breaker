//! Node timer threshold checking system.
//!
//! Checks the node timer ratio against registered thresholds each frame
//! and sends [`NodeTimerThresholdCrossed`] when a new threshold is crossed.

use bevy::prelude::*;

use super::{messages::NodeTimerThresholdCrossed, resources::NodeTimerThresholdRegistry};

/// Checks the node timer ratio against registered thresholds.
///
/// When the ratio crosses a threshold in the registry that hasn't already
/// fired, sends [`NodeTimerThresholdCrossed`] and marks the threshold as fired.
pub fn check_node_timer_thresholds() {
    todo!()
}
