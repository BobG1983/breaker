//! Sequence node evaluator — ordered multi-execute.

use bevy::prelude::*;

use crate::effect_v3::types::{Terminal, TriggerContext};

/// Evaluate a `Tree::Sequence` node: run children left to right.
pub fn evaluate_sequence(
    _entity: Entity,
    _terminals: &[Terminal],
    _context: &TriggerContext,
    _source: &str,
    _commands: &mut Commands,
) {
    todo!()
}
