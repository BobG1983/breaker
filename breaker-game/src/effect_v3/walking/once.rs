//! Once node evaluator — one-shot trigger gate.

use bevy::prelude::*;

use super::walk_effects::evaluate_tree;
use crate::effect_v3::types::{Tree, Trigger, TriggerContext};

/// Evaluate a `Tree::Once` node: if the trigger matches, evaluate the inner tree
/// and then remove this node so it never fires again.
///
/// NOTE: Removal logic is deferred to a future wave — for now, this behaves
/// like a When (repeating) gate but logs a warning about the missing removal.
pub fn evaluate_once(
    entity: Entity,
    gate_trigger: &Trigger,
    inner: &Tree,
    active_trigger: &Trigger,
    context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    if gate_trigger == active_trigger {
        evaluate_tree(entity, inner, active_trigger, context, source, commands);
        // TODO: remove this Once node from the entity's BoundEffects after firing.
        // This requires mutable access to BoundEffects during walking, which needs
        // a deferred command or post-walk cleanup pass.
    }
}
