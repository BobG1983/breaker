//! `walk_effects` — outer loop for evaluating effect trees against a trigger.

use bevy::prelude::*;

use crate::effect_v3::types::{Tree, Trigger, TriggerContext};

/// Walk all effect trees on an entity, evaluating nodes against the given
/// trigger and context.
///
/// This is the main entry point for trigger dispatch. Bridge systems call
/// this after building a `TriggerContext` from a game event.
pub fn walk_effects(
    _entity: Entity,
    _trigger: &Trigger,
    _context: &TriggerContext,
    _trees: &[(String, Tree)],
    _commands: &mut Commands,
) {
    todo!()
}
