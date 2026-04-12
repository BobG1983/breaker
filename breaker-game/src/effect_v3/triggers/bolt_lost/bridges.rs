//! Bolt lost trigger bridge system.
//!
//! Reads `BoltLost` messages and dispatches `BoltLostOccurred` triggers
//! to all entities with bound effects.

use bevy::prelude::*;

use crate::{
    bolt::messages::BoltLost,
    effect_v3::{
        storage::BoundEffects,
        types::{Trigger, TriggerContext},
        walking::walk_effects,
    },
};

/// Global bridge: fires `BoltLostOccurred` on all entities with bound effects
/// when a bolt is lost.
pub fn on_bolt_lost_occurred(
    mut reader: MessageReader<BoltLost>,
    bound_query: Query<(Entity, &BoundEffects)>,
    mut commands: Commands,
) {
    for _ in reader.read() {
        let context = TriggerContext::None;
        let trigger = Trigger::BoltLostOccurred;
        for (entity, bound) in bound_query.iter() {
            let trees = bound.0.clone();
            walk_effects(entity, &trigger, &context, &trees, &mut commands);
        }
    }
}
