//! Time trigger bridge system.
//!
//! Reads [`EffectTimerExpired`] messages and dispatches `TimeExpires` triggers
//! on the entity that owned the expired timer.

use bevy::prelude::*;

use super::messages::EffectTimerExpired;
use crate::effect_v3::{
    storage::BoundEffects,
    types::{Trigger, TriggerContext},
    walking::walk_effects,
};

/// Self bridge: fires `TimeExpires(original_duration)` on the entity whose
/// timer expired.
pub fn on_time_expires(
    mut reader: MessageReader<EffectTimerExpired>,
    bound_query: Query<&BoundEffects>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let trigger = Trigger::TimeExpires(msg.original_duration);
        let context = TriggerContext::None;

        if let Ok(bound) = bound_query.get(msg.entity) {
            let trees = bound.0.clone();
            walk_effects(msg.entity, &trigger, &context, &trees, &mut commands);
        }
    }
}
