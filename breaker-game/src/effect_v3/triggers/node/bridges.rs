//! Node trigger bridge systems.
//!
//! Bridges node lifecycle events to effect tree evaluation triggers.

use bevy::prelude::*;

use crate::effect_v3::{
    storage::BoundEffects,
    types::{Trigger, TriggerContext},
    walking::walk_effects,
};

/// Global bridge: fires `NodeStartOccurred` on all entities with bound effects
/// when a new node starts (enters `NodeState::Playing`).
pub fn on_node_start_occurred(bound_query: Query<(Entity, &BoundEffects)>, mut commands: Commands) {
    let context = TriggerContext::None;
    let trigger = Trigger::NodeStartOccurred;
    for (entity, bound) in bound_query.iter() {
        let trees = bound.0.clone();
        walk_effects(entity, &trigger, &context, &trees, &mut commands);
    }
}

/// Global bridge: fires `NodeEndOccurred` on all entities with bound effects
/// when the current node ends (exits `NodeState::Playing`).
pub fn on_node_end_occurred(bound_query: Query<(Entity, &BoundEffects)>, mut commands: Commands) {
    let context = TriggerContext::None;
    let trigger = Trigger::NodeEndOccurred;
    for (entity, bound) in bound_query.iter() {
        let trees = bound.0.clone();
        walk_effects(entity, &trigger, &context, &trees, &mut commands);
    }
}

/// Global bridge: fires `NodeTimerThresholdOccurred(ratio)` on all entities
/// with bound effects when a timer threshold crossing is detected.
///
/// Reads `NodeTimerThresholdCrossed` messages sent by `check_node_timer_thresholds`.
pub fn on_node_timer_threshold_occurred(
    mut reader: MessageReader<super::messages::NodeTimerThresholdCrossed>,
    bound_query: Query<(Entity, &BoundEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext::None;
        let trigger = Trigger::NodeTimerThresholdOccurred(msg.ratio);
        for (entity, bound) in bound_query.iter() {
            let trees = bound.0.clone();
            walk_effects(entity, &trigger, &context, &trees, &mut commands);
        }
    }
}
