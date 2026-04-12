//! During node evaluator — state-scoped effect application.

use bevy::prelude::*;

use crate::effect_v3::types::{Condition, ScopedTree, TriggerContext};

/// Evaluate a `Tree::During` node: apply inner effects while the condition
/// is true, reverse them when it becomes false.
///
/// NOTE: State tracking (was-true/was-false transitions) is deferred to a
/// future wave. This node type requires per-entity, per-source condition
/// state that is not yet implemented.
pub const fn evaluate_during(
    _entity: Entity,
    _condition: &Condition,
    _inner: &ScopedTree,
    _context: &TriggerContext,
    _source: &str,
    _commands: &mut Commands,
) {
    // TODO: implement During state tracking.
    // Needs per-entity state to track previous condition value:
    // - If condition just became true: fire inner scoped tree
    // - If condition just became false: reverse inner scoped tree
    // - If condition unchanged: no-op
}
