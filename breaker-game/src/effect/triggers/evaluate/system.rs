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
pub(crate) fn evaluate_bound_effects(
    trigger: &Trigger,
    entity: Entity,
    bound: &BoundEffects,
    staged: &mut StagedEffects,
    commands: &mut Commands,
) {
    for (chip_name, node) in &bound.0 {
        walk_bound_node(trigger, entity, chip_name, node, staged, commands);
    }
}

/// Walk `StagedEffects` for a trigger. Matching entries ARE consumed.
pub(crate) fn evaluate_staged_effects(
    trigger: &Trigger,
    entity: Entity,
    staged: &mut StagedEffects,
    commands: &mut Commands,
) {
    let mut additions = Vec::new();
    staged.0.retain(|(chip_name, node)| {
        !walk_staged_node(trigger, entity, chip_name, node, &mut additions, commands)
    });
    staged.0.extend(additions);
}

fn walk_bound_node(
    trigger: &Trigger,
    entity: Entity,
    chip_name: &str,
    node: &EffectNode,
    staged: &mut StagedEffects,
    commands: &mut Commands,
) {
    if let EffectNode::When { trigger: t, then } = node
        && t == trigger
    {
        for child in then {
            match child {
                EffectNode::Do(effect) => {
                    commands.fire_effect(entity, effect.clone(), chip_name.to_string());
                }
                other => {
                    staged.0.push((chip_name.to_string(), other.clone()));
                }
            }
        }
    }
}

/// Returns true if the node was consumed (matched).
fn walk_staged_node(
    trigger: &Trigger,
    entity: Entity,
    chip_name: &str,
    node: &EffectNode,
    additions: &mut Vec<(String, EffectNode)>,
    commands: &mut Commands,
) -> bool {
    match node {
        EffectNode::When { trigger: t, then } if t == trigger => {
            for child in then {
                match child {
                    EffectNode::Do(effect) => {
                        commands.fire_effect(entity, effect.clone(), chip_name.to_string());
                    }
                    EffectNode::Reverse { effects, chains } => {
                        for effect in effects {
                            commands.reverse_effect(entity, effect.clone(), chip_name.to_string());
                        }
                        if !chains.is_empty() {
                            commands.queue(RemoveChainsCommand {
                                entity,
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
                                        entity,
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
                        commands.fire_effect(entity, effect.clone(), chip_name.to_string());
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
            });
            true // consumed -- ResolveOnCommand resolves target asynchronously
        }
        _ => false,
    }
}
