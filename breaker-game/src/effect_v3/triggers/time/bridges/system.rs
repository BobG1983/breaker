//! Time trigger bridge system.
//!
//! Reads [`EffectTimerExpired`] messages and dispatches `TimeExpires` triggers
//! on the entity that owned the expired timer.

use bevy::prelude::*;

use super::super::messages::EffectTimerExpired;
use crate::effect_v3::{
    storage::{BoundEffects, StagedEffects},
    types::{Trigger, TriggerContext},
    walking::{walk_bound_effects, walk_staged_effects},
};

/// Self bridge: fires `TimeExpires(original_duration)` on the entity whose
/// timer expired.
pub fn on_time_expires(
    mut reader: MessageReader<EffectTimerExpired>,
    bound_query: Query<(&BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let trigger = Trigger::TimeExpires(msg.original_duration);
        let context = TriggerContext::None;

        if let Ok((bound, staged)) = bound_query.get(msg.entity) {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();
            walk_staged_effects(msg.entity, &trigger, &context, &staged_trees, &mut commands);
            walk_bound_effects(msg.entity, &trigger, &context, &bound_trees, &mut commands);
        }
    }
}
