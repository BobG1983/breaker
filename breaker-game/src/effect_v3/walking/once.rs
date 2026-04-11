//! Once node evaluator — one-shot trigger gate.

use bevy::prelude::*;

use crate::effect_v3::types::{Tree, Trigger, TriggerContext};

/// Evaluate a `Tree::Once` node: if the trigger matches, evaluate the inner tree
/// and then remove this node so it never fires again.
pub fn evaluate_once(
    _entity: Entity,
    _trigger: &Trigger,
    _inner: &Tree,
    _active_trigger: &Trigger,
    _context: &TriggerContext,
    _source: &str,
    _commands: &mut Commands,
) {
    todo!()
}
