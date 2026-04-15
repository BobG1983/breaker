//! Node trigger bridge systems.
//!
//! Bridges node lifecycle events to effect tree evaluation triggers.

use bevy::prelude::*;

use crate::effect_v3::{
    storage::{BoundEffects, StagedEffects},
    types::{Trigger, TriggerContext},
    walking::{walk_bound_effects, walk_staged_effects},
};

/// Global bridge: fires `NodeStartOccurred` on all entities with bound effects
/// when a new node starts (enters `NodeState::Playing`).
pub fn on_node_start_occurred(
    bound_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    let context = TriggerContext::None;
    let trigger = Trigger::NodeStartOccurred;
    for (entity, bound, staged) in bound_query.iter() {
        let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
        let bound_trees = bound.0.clone();
        walk_staged_effects(entity, &trigger, &context, &staged_trees, &mut commands);
        walk_bound_effects(entity, &trigger, &context, &bound_trees, &mut commands);
    }
}

/// Global bridge: fires `NodeEndOccurred` on all entities with bound effects
/// when the current node ends (exits `NodeState::Playing`).
pub fn on_node_end_occurred(
    bound_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    let context = TriggerContext::None;
    let trigger = Trigger::NodeEndOccurred;
    for (entity, bound, staged) in bound_query.iter() {
        let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
        let bound_trees = bound.0.clone();
        walk_staged_effects(entity, &trigger, &context, &staged_trees, &mut commands);
        walk_bound_effects(entity, &trigger, &context, &bound_trees, &mut commands);
    }
}

/// Global bridge: fires `NodeTimerThresholdOccurred(ratio)` on all entities
/// with bound effects when a timer threshold crossing is detected.
///
/// Reads `NodeTimerThresholdCrossed` messages sent by `check_node_timer_thresholds`.
pub fn on_node_timer_threshold_occurred(
    mut reader: MessageReader<super::super::messages::NodeTimerThresholdCrossed>,
    bound_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext::None;
        let trigger = Trigger::NodeTimerThresholdOccurred(msg.ratio);
        for (entity, bound, staged) in bound_query.iter() {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();
            walk_staged_effects(entity, &trigger, &context, &staged_trees, &mut commands);
            walk_bound_effects(entity, &trigger, &context, &bound_trees, &mut commands);
        }
    }
}
