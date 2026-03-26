//! Shared helper functions for bridge systems.
//!
//! These helpers are used by multiple trigger files under `effect/triggers/`.

use bevy::prelude::*;

use super::{
    armed::ArmedEffects,
    definition::{EffectChains, EffectNode, EffectTarget, Trigger},
    effect_nodes::until::{UntilTimers, UntilTriggers},
    evaluate::{NodeEvalResult, evaluate_node},
    typed_events::fire_typed_event,
};

/// Evaluates armed triggers on all bolt entities that have `ArmedEffects`.
pub(super) fn evaluate_armed_all(
    mut armed_query: Query<(Entity, &mut ArmedEffects)>,
    trigger_kind: Trigger,
    commands: &mut Commands,
) {
    for (bolt_entity, mut armed) in &mut armed_query {
        let targets = vec![EffectTarget::Entity(bolt_entity)];
        resolve_armed(&mut armed, trigger_kind, targets, commands);
    }
}

/// Arms a bolt entity with a remaining trigger chain.
///
/// If the bolt already has `ArmedEffects`, pushes to the existing vec.
/// Otherwise, inserts a new `ArmedEffects` component.
pub(super) fn arm_bolt(
    armed_query: &mut Query<&mut ArmedEffects>,
    commands: &mut Commands,
    bolt_entity: Entity,
    chip_name: Option<String>,
    remaining: EffectNode,
) {
    if let Ok(mut armed) = armed_query.get_mut(bolt_entity) {
        armed.0.push((chip_name, remaining));
    } else {
        commands
            .entity(bolt_entity)
            .insert(ArmedEffects(vec![(chip_name, remaining)]));
    }
}

/// Evaluates armed triggers on a specific bolt entity.
pub(super) fn evaluate_armed(
    armed_query: &mut Query<&mut ArmedEffects>,
    commands: &mut Commands,
    bolt_entity: Entity,
    trigger_kind: Trigger,
) {
    if let Ok(mut armed) = armed_query.get_mut(bolt_entity) {
        let targets = vec![EffectTarget::Entity(bolt_entity)];
        resolve_armed(&mut armed, trigger_kind, targets, commands);
    }
}

/// Resolves armed trigger chains: fires leaves, re-arms non-leaves, retains non-matches.
pub(super) fn resolve_armed(
    armed: &mut ArmedEffects,
    trigger_kind: Trigger,
    targets: Vec<EffectTarget>,
    commands: &mut Commands,
) {
    let mut new_armed = Vec::new();
    for (chip_name, chain) in armed.0.drain(..) {
        let results = evaluate_node(trigger_kind, &chain);
        let mut matched = false;
        for result in results {
            match result {
                NodeEvalResult::Fire(effect) => {
                    matched = true;
                    fire_typed_event(effect, targets.clone(), chip_name.clone(), commands);
                }
                NodeEvalResult::Arm(next) => {
                    matched = true;
                    new_armed.push((chip_name.clone(), next));
                }
                NodeEvalResult::NoMatch => {}
            }
        }
        if !matched {
            new_armed.push((chip_name, chain));
        }
    }
    armed.0 = new_armed;
}

/// Evaluates entity-local `EffectChains` against a trigger kind.
///
/// Handles `Once` node unwrapping: if a `Once` wraps children that match the
/// trigger, the effects are fired and the `Once` is consumed (removed).
/// If no children match, the `Once` is preserved.
pub(super) fn evaluate_entity_chains(
    chains: &mut EffectChains,
    trigger_kind: Trigger,
    targets: Vec<EffectTarget>,
    commands: &mut Commands,
) {
    let mut consumed_indices = Vec::new();

    for (i, (chip_name, node)) in chains.0.iter().enumerate() {
        match node {
            EffectNode::Once(children) => {
                // Unwrap the Once: evaluate inner children against the trigger
                let mut any_matched = false;
                for child in children {
                    for result in evaluate_node(trigger_kind, child) {
                        match result {
                            NodeEvalResult::Fire(effect) => {
                                any_matched = true;
                                fire_typed_event(
                                    effect,
                                    targets.clone(),
                                    chip_name.clone(),
                                    commands,
                                );
                            }
                            NodeEvalResult::Arm(_remaining) => {
                                // Arming from Once children — treat as matched
                                any_matched = true;
                            }
                            NodeEvalResult::NoMatch => {}
                        }
                    }
                }
                if any_matched {
                    consumed_indices.push(i);
                }
            }
            _ => {
                // Regular nodes — evaluate directly
                for result in evaluate_node(trigger_kind, node) {
                    match result {
                        NodeEvalResult::Fire(effect) => {
                            fire_typed_event(effect, targets.clone(), chip_name.clone(), commands);
                        }
                        NodeEvalResult::Arm(_) | NodeEvalResult::NoMatch => {}
                    }
                }
            }
        }
    }

    // Remove consumed Once nodes in reverse order
    for &i in consumed_indices.iter().rev() {
        chains.0.remove(i);
    }
}

/// Evaluates `When` children inside a bolt's `UntilTimers` and `UntilTriggers` entries.
///
/// Until entries contain children that may include `When` nodes. These `When` nodes
/// should be evaluated by bridge systems against the current trigger, firing effects
/// when matched. The Until entry itself is not consumed — only its children are
/// evaluated for the current trigger kind.
pub(super) fn evaluate_until_children(
    until_query: &Query<(Option<&UntilTimers>, Option<&UntilTriggers>)>,
    bolt_entity: Entity,
    trigger_kind: Trigger,
    targets: &[EffectTarget],
    commands: &mut Commands,
) {
    let Ok((until_timers, until_triggers)) = until_query.get(bolt_entity) else {
        return;
    };

    if let Some(timers) = until_timers {
        for entry in &timers.0 {
            for child in &entry.children {
                for result in evaluate_node(trigger_kind, child) {
                    if let NodeEvalResult::Fire(effect) = result {
                        fire_typed_event(effect, targets.to_vec(), None, commands);
                    }
                }
            }
        }
    }

    if let Some(triggers) = until_triggers {
        for entry in &triggers.0 {
            for child in &entry.children {
                for result in evaluate_node(trigger_kind, child) {
                    if let NodeEvalResult::Fire(effect) = result {
                        fire_typed_event(effect, targets.to_vec(), None, commands);
                    }
                }
            }
        }
    }
}
