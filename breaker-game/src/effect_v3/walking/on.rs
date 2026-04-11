//! On node evaluator — participant redirection.

use bevy::prelude::*;

use crate::effect_v3::types::{ParticipantTarget, Terminal, TriggerContext};

/// Evaluate a `Tree::On` node: redirect the terminal to the entity
/// identified by the participant target in the trigger context.
///
/// NOTE: Participant resolution is deferred to a future wave. On nodes
/// need to resolve `ParticipantTarget` to an `Entity` from the
/// `TriggerContext`, then evaluate the terminal on that entity.
#[allow(
    clippy::missing_const_for_fn,
    reason = "deferred implementation will use &mut Commands"
)]
pub fn evaluate_on(
    _entity: Entity,
    _target: ParticipantTarget,
    _terminal: &Terminal,
    _context: &TriggerContext,
    _source: &str,
    _commands: &mut Commands,
) {
    // TODO: implement On participant resolution.
    // Needs:
    // - Match ParticipantTarget against TriggerContext to get target entity
    // - Evaluate terminal on the resolved target entity instead of the owner
}
