//! Until node evaluator — event-scoped effect application.

use bevy::prelude::*;

use crate::effect_v3::types::{ScopedTree, Trigger, TriggerContext};

/// Evaluate a `Tree::Until` node: apply inner effects immediately,
/// reverse them when the trigger fires.
pub fn evaluate_until(
    _entity: Entity,
    _trigger: &Trigger,
    _inner: &ScopedTree,
    _context: &TriggerContext,
    _source: &str,
    _commands: &mut Commands,
) {
    todo!()
}
