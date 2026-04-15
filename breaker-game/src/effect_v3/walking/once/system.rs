//! Once node evaluator — one-shot trigger gate.

use bevy::prelude::*;

use crate::effect_v3::{
    commands::EffectCommandsExt,
    types::{Tree, Trigger, TriggerContext},
    walking::walk_effects::evaluate_tree,
};

/// Evaluate a `Tree::Once` node: if the trigger matches, evaluate the inner tree
/// and then remove this node so it never fires again.
///
/// When the inner tree's root is itself a trigger gate (`When`, `Once`,
/// `Until`), the outer `Once` ARMS the inner by staging it under the
/// same source name BEFORE removing itself from `BoundEffects`. The
/// remove-then-stage ordering is load-bearing: `RemoveEffectCommand`
/// sweeps both `BoundEffects` and `StagedEffects` by name, so queuing
/// the remove first clears the outer without touching the later staged
/// entry.
///
/// `Once` always removes itself from `BoundEffects` on outer-trigger
/// match, regardless of whether the inner was armed or recursed —
/// preserving the existing one-shot invariant.
pub fn evaluate_once(
    entity: Entity,
    gate_trigger: &Trigger,
    inner: &Tree,
    active_trigger: &Trigger,
    context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    if gate_trigger != active_trigger {
        return;
    }
    match inner {
        Tree::When(..) | Tree::Once(..) | Tree::Until(..) => {
            // Remove the outer FIRST so the subsequent stage is not wiped
            // by `RemoveEffectCommand`'s name-sweep of `StagedEffects`.
            commands.remove_effect(entity, source);
            commands.stage_effect(entity, source.to_owned(), inner.clone());
        }
        _ => {
            evaluate_tree(entity, inner, active_trigger, context, source, commands);
            commands.remove_effect(entity, source);
        }
    }
}
