//! Node trigger bridge systems.
//!
//! Bridges node lifecycle events to effect tree evaluation triggers.

use bevy::prelude::*;

use crate::effect_v3::types::{Trigger, TriggerContext};

/// Global bridge: fires `NodeStartOccurred` on all entities with bound effects
/// when a new node starts.
pub fn on_node_start_occurred() {
    todo!()
}

/// Global bridge: fires `NodeEndOccurred` on all entities with bound effects
/// when the current node ends.
pub fn on_node_end_occurred() {
    todo!()
}

/// Global bridge: fires `NodeTimerThresholdOccurred(ratio)` on all entities
/// with bound effects when a timer threshold crossing is detected.
pub fn on_node_timer_threshold_occurred() {
    todo!()
}
