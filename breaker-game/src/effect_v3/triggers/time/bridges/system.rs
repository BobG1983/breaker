//! Time trigger bridge system.
//!
//! Reads [`EffectTimerExpired`] messages and dispatches `TimeExpires` triggers
//! on the entity that owned the expired timer.

use bevy::prelude::*;

use super::super::messages::EffectTimerExpired;
use crate::{
    effect_v3::{
        storage::{BoundEffects, StagedEffects},
        types::{Tree, Trigger, TriggerContext},
        walking::{walk_bound_effects, walk_staged_effects},
    },
    prelude::SourceId,
};

/// Fires `TimeExpires(original_duration)` on the entity whose timer expired.
///
/// Filters both `BoundEffects` and `StagedEffects` by the message's `source`
/// before walking — only entries whose name (wrapped via
/// `SourceId::from(...)`) matches `msg.source` are forwarded to the walkers.
/// This disambiguates concurrent same-duration Untils on the same entity:
/// each timer entry is tagged with the source that armed it, and the
/// dispatch only reverses the matching Until.
pub fn on_time_expires(
    mut reader: MessageReader<EffectTimerExpired>,
    bound_query: Query<(&BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let trigger = Trigger::TimeExpires(msg.original_duration);
        let context = TriggerContext::None;

        if let Ok((bound, staged)) = bound_query.get(msg.entity) {
            let bound_trees: Vec<(String, Tree)> = bound
                .0
                .iter()
                .filter(|(name, _)| SourceId::from(name.clone()) == msg.source)
                .cloned()
                .collect();
            let staged_trees: Vec<(String, Tree)> = staged
                .map(|s| {
                    s.0.iter()
                        .filter(|(name, _)| SourceId::from(name.clone()) == msg.source)
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
            walk_staged_effects(msg.entity, &trigger, &context, &staged_trees, &mut commands);
            walk_bound_effects(msg.entity, &trigger, &context, &bound_trees, &mut commands);
        }
    }
}
