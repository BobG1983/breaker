//! Resources for the node trigger category.

use std::collections::HashSet;

use bevy::prelude::*;
use ordered_float::OrderedFloat;

/// Global resource storing registered node timer thresholds.
///
/// `thresholds` is populated by a scan system that runs after tree installation.
/// `fired` tracks which thresholds have already fired this node to avoid re-firing.
/// Reset on `OnEnter(NodeState::Playing)` by `reset_node_timer_thresholds`.
#[derive(Resource, Default, Debug)]
pub struct NodeTimerThresholdRegistry {
    /// All registered threshold ratios (0.0-1.0).
    pub thresholds: Vec<OrderedFloat<f32>>,
    /// Thresholds that have already fired this node.
    pub fired:      HashSet<OrderedFloat<f32>>,
}
