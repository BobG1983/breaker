//! When node evaluator — repeating trigger gate.

use bevy::prelude::*;

use crate::effect_v3::types::{Tree, Trigger, TriggerContext};

/// Evaluate a `Tree::When` node: if the trigger matches, evaluate the inner tree.
/// Repeats on every match.
pub fn evaluate_when(
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
