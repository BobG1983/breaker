//! Messages for the node trigger category.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

/// Sent by [`check_node_timer_thresholds`] when the node timer ratio crosses
/// a registered threshold.
///
/// Read by the `on_node_timer_threshold_occurred` bridge which dispatches
/// `NodeTimerThresholdOccurred(ratio)` globally.
#[derive(Message, Clone, Debug)]
pub struct NodeTimerThresholdCrossed {
    /// The threshold ratio (0.0-1.0) that was crossed.
    pub ratio: OrderedFloat<f32>,
}
