//! Fire node evaluator — immediate effect execution.

use bevy::prelude::*;

use crate::effect_v3::types::{EffectType, TriggerContext};

/// Evaluate a `Tree::Fire` node: execute the effect immediately on the entity.
pub fn evaluate_fire(
    _entity: Entity,
    _effect: &EffectType,
    _source: &str,
    _context: &TriggerContext,
    _commands: &mut Commands,
) {
    todo!()
}
