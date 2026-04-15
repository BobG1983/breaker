//! Bolt lost trigger bridge system.
//!
//! Reads `BoltLost` messages and dispatches `BoltLostOccurred` triggers
//! to all entities with bound effects.

use bevy::prelude::*;

use crate::{
    bolt::messages::BoltLost,
    effect_v3::{
        storage::{BoundEffects, StagedEffects},
        types::{Trigger, TriggerContext},
        walking::{walk_bound_effects, walk_staged_effects},
    },
};

/// Global bridge: fires `BoltLostOccurred` on all entities with bound effects
/// when a bolt is lost.
pub fn on_bolt_lost_occurred(
    mut reader: MessageReader<BoltLost>,
    bound_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext::BoltLost {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        let trigger = Trigger::BoltLostOccurred;
        for (entity, bound, staged) in bound_query.iter() {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();
            walk_staged_effects(entity, &trigger, &context, &staged_trees, &mut commands);
            walk_bound_effects(entity, &trigger, &context, &bound_trees, &mut commands);
        }
    }
}
