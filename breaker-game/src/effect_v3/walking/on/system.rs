//! On node evaluator — participant redirection.

use bevy::prelude::*;

use crate::effect_v3::{
    commands::EffectCommandsExt,
    conditions::is_armed_source,
    types::{
        BoltLostTarget, BumpTarget, DeathTarget, ImpactTarget, ParticipantTarget, Terminal,
        TriggerContext,
    },
    walking::sequence::evaluate_terminal,
};

/// Evaluate a `Tree::On` node: redirect the terminal to the entity
/// identified by the participant target in the trigger context.
///
/// When the source string encodes an armed-entry key (see
/// `is_armed_source`), this additionally queues a
/// `TrackArmedFireCommand` on the owner so the Shape D disarm path can
/// reverse effects on the exact participants they were fired on.
pub fn evaluate_on(
    owner: Entity,
    target: ParticipantTarget,
    terminal: &Terminal,
    context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    if let Some(resolved) = resolve_participant(target, context) {
        evaluate_terminal(resolved, terminal, source, commands);
        if is_armed_source(source) {
            commands.track_armed_fire(owner, source.to_owned(), resolved);
        }
    }
}

const fn resolve_participant(
    target: ParticipantTarget,
    context: &TriggerContext,
) -> Option<Entity> {
    match (target, context) {
        (ParticipantTarget::Bump(BumpTarget::Bolt), TriggerContext::Bump { bolt, .. }) => *bolt,
        (ParticipantTarget::Bump(BumpTarget::Breaker), TriggerContext::Bump { breaker, .. }) => {
            Some(*breaker)
        }
        (
            ParticipantTarget::Impact(ImpactTarget::Impactor),
            TriggerContext::Impact { impactor, .. },
        ) => Some(*impactor),
        (
            ParticipantTarget::Impact(ImpactTarget::Impactee),
            TriggerContext::Impact { impactee, .. },
        ) => Some(*impactee),
        (ParticipantTarget::Death(DeathTarget::Victim), TriggerContext::Death { victim, .. }) => {
            Some(*victim)
        }
        (ParticipantTarget::Death(DeathTarget::Killer), TriggerContext::Death { killer, .. }) => {
            *killer
        }
        (
            ParticipantTarget::BoltLost(BoltLostTarget::Bolt),
            TriggerContext::BoltLost { bolt, .. },
        ) => Some(*bolt),
        (
            ParticipantTarget::BoltLost(BoltLostTarget::Breaker),
            TriggerContext::BoltLost { breaker, .. },
        ) => Some(*breaker),
        _ => None,
    }
}
