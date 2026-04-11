//! During node evaluator — state-scoped effect application.

use bevy::prelude::*;

use crate::effect_v3::types::{Condition, ScopedTree, TriggerContext};

/// Evaluate a `Tree::During` node: apply inner effects while the condition
/// is true, reverse them when it becomes false.
pub fn evaluate_during(
    _entity: Entity,
    _condition: &Condition,
    _inner: &ScopedTree,
    _context: &TriggerContext,
    _source: &str,
    _commands: &mut Commands,
) {
    todo!()
}
