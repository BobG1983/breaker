//! On node evaluator — participant redirection.

use bevy::prelude::*;

use crate::effect_v3::types::{ParticipantTarget, Terminal, TriggerContext};

/// Evaluate a `Tree::On` node: redirect the terminal to the entity
/// identified by the participant target in the trigger context.
pub fn evaluate_on(
    _entity: Entity,
    _target: ParticipantTarget,
    _terminal: &Terminal,
    _context: &TriggerContext,
    _source: &str,
    _commands: &mut Commands,
) {
    todo!()
}
