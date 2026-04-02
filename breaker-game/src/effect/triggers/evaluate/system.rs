use bevy::prelude::*;

use crate::effect::{
    commands::{EffectCommandsExt, ResolveOnCommand},
    core::*,
};

/// Command that removes matching chains from an entity's `BoundEffects`.
pub(crate) struct RemoveChainsCommand {
    pub(crate) entity: Entity,
    pub(crate) chains: Vec<EffectNode>,
}

impl Command for RemoveChainsCommand {
    fn apply(self, world: &mut World) {
        if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
            bound.0.retain(|(_, node)| !self.chains.contains(node));
        }
    }
}

/// Walk `BoundEffects` for a trigger. Entries are NEVER consumed.
///
/// `context` carries the entities involved in the trigger event (e.g., the
/// specific cell and bolt in an `Impacted(Cell)` collision). When an inner
/// `On(target)` node finds a matching entity in the context, it resolves to
/// that single entity instead of querying all entities of that type.
pub(crate) fn evaluate_bound_effects(
    trigger: &Trigger,
    effect_owner: Entity,
    bound: &BoundEffects,
    staged: &mut StagedEffects,
    commands: &mut Commands,
    context: TriggerContext,
) {
    for (chip_name, node) in &bound.0 {
        walk_bound_node(
            trigger,
            effect_owner,
            chip_name,
            node,
            staged,
            commands,
            context,
        );
    }
}

/// Walk `StagedEffects` for a trigger. Matching entries ARE consumed.
///
/// See [`evaluate_bound_effects`] for `context` semantics.
pub(crate) fn evaluate_staged_effects(
    trigger: &Trigger,
    effect_owner: Entity,
    staged: &mut StagedEffects,
    commands: &mut Commands,
    context: TriggerContext,
) {
    let mut additions = Vec::new();
    staged.0.retain(|(chip_name, node)| {
        !walk_staged_node(
            trigger,
            effect_owner,
            chip_name,
            node,
            &mut additions,
            commands,
            context,
        )
    });
    staged.0.extend(additions);
}

fn walk_bound_node(
    trigger: &Trigger,
    effect_owner: Entity,
    chip_name: &str,
    node: &EffectNode,
    staged: &mut StagedEffects,
    commands: &mut Commands,
    _context: TriggerContext,
) {
    if let EffectNode::When { trigger: t, then } = node
        && t == trigger
    {
        for child in then {
            match child {
                EffectNode::Do(effect) => {
                    commands.fire_effect(effect_owner, effect.clone(), chip_name.to_string());
                }
                other => {
                    staged.0.push((chip_name.to_string(), other.clone()));
                }
            }
        }
    }
    // context is propagated through staged effects — On nodes queued from
    // bound evaluation will pick it up via walk_staged_node.
}

/// Returns true if the node was consumed (matched).
fn walk_staged_node(
    trigger: &Trigger,
    effect_owner: Entity,
    chip_name: &str,
    node: &EffectNode,
    additions: &mut Vec<(String, EffectNode)>,
    commands: &mut Commands,
    context: TriggerContext,
) -> bool {
    match node {
        EffectNode::When { trigger: t, then } if t == trigger => {
            for child in then {
                match child {
                    EffectNode::Do(effect) => {
                        commands.fire_effect(effect_owner, effect.clone(), chip_name.to_string());
                    }
                    EffectNode::Reverse { effects, chains } => {
                        for effect in effects {
                            commands.reverse_effect(
                                effect_owner,
                                effect.clone(),
                                chip_name.to_string(),
                            );
                        }
                        if !chains.is_empty() {
                            commands.queue(RemoveChainsCommand {
                                entity: effect_owner,
                                chains: chains.clone(),
                            });
                        }
                    }
                    other => {
                        additions.push((chip_name.to_string(), other.clone()));
                    }
                }
            }
            true // consumed
        }
        EffectNode::Once(children) => {
            let mut any_matched = false;
            for child in children {
                match child {
                    EffectNode::When { trigger: t, then } if t == trigger => {
                        any_matched = true;
                        for inner in then {
                            match inner {
                                EffectNode::Do(effect) => {
                                    commands.fire_effect(
                                        effect_owner,
                                        effect.clone(),
                                        chip_name.to_string(),
                                    );
                                }
                                other => {
                                    additions.push((chip_name.to_string(), other.clone()));
                                }
                            }
                        }
                    }
                    EffectNode::Do(effect) => {
                        // Bare Do always matches -- fire immediately
                        any_matched = true;
                        commands.fire_effect(effect_owner, effect.clone(), chip_name.to_string());
                    }
                    _ => {}
                }
            }
            any_matched
        }
        EffectNode::On {
            target,
            permanent,
            then: on_children,
        } => {
            commands.queue(ResolveOnCommand {
                target: *target,
                chip_name: chip_name.to_string(),
                children: on_children.clone(),
                permanent: *permanent,
                context,
            });
            true // consumed -- ResolveOnCommand resolves target asynchronously
        }
        _ => false,
    }
}
