//! Shared helper functions for bridge systems.
//!
//! These helpers are used by multiple trigger files under `effect/triggers/`.

use bevy::prelude::*;

use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    effect::{
        armed::ArmedEffects,
        definition::{EffectChains, EffectNode, EffectTarget, Trigger},
        effect_nodes::until::{UntilTimers, UntilTriggers},
        evaluate::evaluate_node,
        typed_events::fire_typed_event,
    },
};

/// Evaluates armed triggers on all bolt entities that have `ArmedEffects`.
pub(in crate::effect) fn evaluate_armed_all(
    mut armed_query: Query<(Entity, &mut ArmedEffects)>,
    trigger_kind: Trigger,
    commands: &mut Commands,
) {
    for (bolt_entity, mut armed) in &mut armed_query {
        let targets = vec![EffectTarget::Entity(bolt_entity)];
        resolve_armed(&mut armed, trigger_kind, targets, commands);
    }
}

// FUTURE: may be used for upcoming phases
// /// Arms a bolt entity with a remaining trigger chain.
// ///
// /// If the bolt already has `ArmedEffects`, pushes to the existing vec.
// /// Otherwise, inserts a new `ArmedEffects` component.
// pub(in crate::effect) fn arm_bolt(
//     armed_query: &mut Query<&mut ArmedEffects>,
//     commands: &mut Commands,
//     bolt_entity: Entity,
//     chip_name: Option<String>,
//     remaining: EffectNode,
// ) {
//     if let Ok(mut armed) = armed_query.get_mut(bolt_entity) {
//         armed.0.push((chip_name, remaining));
//     } else {
//         commands
//             .entity(bolt_entity)
//             .insert(ArmedEffects(vec![(chip_name, remaining)]));
//     }
// }

/// Evaluates armed triggers on a specific bolt entity.
pub(in crate::effect) fn evaluate_armed(
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
pub(in crate::effect) fn resolve_armed(
    armed: &mut ArmedEffects,
    trigger_kind: Trigger,
    targets: Vec<EffectTarget>,
    commands: &mut Commands,
) {
    let mut new_armed = Vec::new();
    for (chip_name, chain) in armed.0.drain(..) {
        if let Some(children) = evaluate_node(trigger_kind, &chain) {
            for child in children {
                match child {
                    EffectNode::Do(effect) => {
                        fire_typed_event(
                            effect.clone(),
                            targets.clone(),
                            chip_name.clone(),
                            commands,
                        );
                    }
                    other => {
                        new_armed.push((chip_name.clone(), other.clone()));
                    }
                }
            }
        } else {
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
pub(in crate::effect) fn evaluate_entity_chains(
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
                    if let Some(grandchildren) = evaluate_node(trigger_kind, child) {
                        any_matched = true;
                        for gc in grandchildren {
                            if let EffectNode::Do(effect) = gc {
                                fire_typed_event(
                                    effect.clone(),
                                    targets.clone(),
                                    chip_name.clone(),
                                    commands,
                                );
                            }
                            // Non-Do grandchildren (Arm equivalent) — treat as matched
                        }
                    }
                }
                if any_matched {
                    consumed_indices.push(i);
                }
            }
            _ => {
                // Regular nodes — evaluate directly
                if let Some(children) = evaluate_node(trigger_kind, node) {
                    for child in children {
                        if let EffectNode::Do(effect) = child {
                            fire_typed_event(
                                effect.clone(),
                                targets.clone(),
                                chip_name.clone(),
                                commands,
                            );
                        }
                        // Non-Do children (Arm equivalent) — discarded in entity chains
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

/// Sweep all entities for a bump trigger, filtering by grade.
///
/// Reads `BumpPerformed` messages, optionally filters by `BumpGrade`, sweeps ALL
/// entities with `EffectChains`, and evaluates `ArmedEffects` on the specific bolt.
pub(in crate::effect) fn bridge_global_bump_inner(
    reader: &mut MessageReader<BumpPerformed>,
    chains_query: &mut Query<&mut EffectChains>,
    armed_query: &mut Query<&mut ArmedEffects>,
    commands: &mut Commands,
    grade: Option<BumpGrade>,
    trigger: Trigger,
) {
    for performed in reader.read() {
        if let Some(g) = grade
            && performed.grade != g
        {
            continue;
        }
        let targets = performed
            .bolt
            .map_or(vec![], |b| vec![EffectTarget::Entity(b)]);

        for mut chains in chains_query.iter_mut() {
            evaluate_entity_chains(&mut chains, trigger, targets.clone(), commands);
        }

        if let Some(bolt) = performed.bolt {
            evaluate_armed(armed_query, commands, bolt, trigger);
        }
    }
}

/// Evaluate a specific bolt for a bumped trigger, filtering by grade.
///
/// Reads `BumpPerformed` messages, optionally filters by `BumpGrade`, evaluates
/// ONLY the specific bolt entity's `EffectChains` and `ArmedEffects`.
pub(in crate::effect) fn bridge_targeted_bumped_inner(
    reader: &mut MessageReader<BumpPerformed>,
    chains_query: &mut Query<&mut EffectChains>,
    armed_query: &mut Query<&mut ArmedEffects>,
    commands: &mut Commands,
    grade: Option<BumpGrade>,
    trigger: Trigger,
) {
    for performed in reader.read() {
        if let Some(g) = grade
            && performed.grade != g
        {
            continue;
        }
        let Some(bolt) = performed.bolt else {
            continue;
        };
        let targets = vec![EffectTarget::Entity(bolt)];

        if let Ok(mut chains) = chains_query.get_mut(bolt) {
            evaluate_entity_chains(&mut chains, trigger, targets.clone(), commands);
        }

        evaluate_armed(armed_query, commands, bolt, trigger);
    }
}

/// Evaluates `When` children inside a bolt's `UntilTimers` and `UntilTriggers` entries.
///
/// Until entries contain children that may include `When` nodes. These `When` nodes
/// should be evaluated by bridge systems against the current trigger, firing effects
/// when matched. The Until entry itself is not consumed — only its children are
/// evaluated for the current trigger kind.
pub(in crate::effect) fn evaluate_until_children(
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
                if let Some(children) = evaluate_node(trigger_kind, child) {
                    for c in children {
                        if let EffectNode::Do(effect) = c {
                            fire_typed_event(effect.clone(), targets.to_vec(), None, commands);
                        }
                    }
                }
            }
        }
    }

    if let Some(triggers) = until_triggers {
        for entry in &triggers.0 {
            for child in &entry.children {
                if let Some(children) = evaluate_node(trigger_kind, child) {
                    for c in children {
                        if let EffectNode::Do(effect) = c {
                            fire_typed_event(effect.clone(), targets.to_vec(), None, commands);
                        }
                    }
                }
            }
        }
    }
}
