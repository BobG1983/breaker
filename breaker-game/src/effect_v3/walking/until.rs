//! Until node evaluator — event-scoped effect application.

use bevy::prelude::*;

use crate::effect_v3::types::{ScopedTree, Trigger, TriggerContext};

/// Evaluate a `Tree::Until` node: apply inner effects immediately,
/// reverse them when the trigger fires.
///
/// NOTE: Reversal logic is deferred to a future wave. Until nodes need
/// state tracking to know whether inner effects have been applied, and
/// reversal dispatch for scoped trees.
#[allow(
    clippy::missing_const_for_fn,
    reason = "deferred implementation will use &mut Commands"
)]
pub fn evaluate_until(
    _entity: Entity,
    _trigger: &Trigger,
    _inner: &ScopedTree,
    _context: &TriggerContext,
    _source: &str,
    _commands: &mut Commands,
) {
    // TODO: implement Until state tracking and reversal.
    // Needs:
    // - Track whether inner effects have been applied
    // - On first evaluation: fire inner scoped tree
    // - When trigger matches: reverse inner scoped tree and remove node
}
