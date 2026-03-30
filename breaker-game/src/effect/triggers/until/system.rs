use bevy::prelude::*;

use crate::effect::{commands::EffectCommandsExt, core::*};

fn desugar_until(
    mut query: Query<(Entity, &mut BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for (entity, mut bound, mut staged) in &mut query {
        let mut new_bound = Vec::new();
        let mut new_staged = Vec::new();

        // Extract Until nodes from BoundEffects
        let mut bound_untils = Vec::new();
        bound.0.retain(|(chip_name, node)| {
            if matches!(node, EffectNode::Until { .. }) {
                bound_untils.push((chip_name.clone(), node.clone()));
                false
            } else {
                true
            }
        });

        // Extract Until nodes from StagedEffects
        let mut staged_untils = Vec::new();
        staged.0.retain(|(chip_name, node)| {
            if matches!(node, EffectNode::Until { .. }) {
                staged_untils.push((chip_name.clone(), node.clone()));
                false
            } else {
                true
            }
        });

        // Desugar each Until
        for (chip_name, node) in bound_untils.into_iter().chain(staged_untils) {
            if let EffectNode::Until { trigger, then } = node {
                let mut fired_effects = Vec::new();
                let mut pushed_chains = Vec::new();

                for child in then {
                    match child {
                        EffectNode::Do(effect) => {
                            commands.fire_effect(entity, effect.clone(), chip_name.clone());
                            fired_effects.push(effect);
                        }
                        other => {
                            new_bound.push((chip_name.clone(), other.clone()));
                            pushed_chains.push(other);
                        }
                    }
                }

                // Replace with When+Reverse in StagedEffects
                new_staged.push((
                    chip_name,
                    EffectNode::When {
                        trigger,
                        then: vec![EffectNode::Reverse {
                            effects: fired_effects,
                            chains: pushed_chains,
                        }],
                    },
                ));
            }
        }

        bound.0.extend(new_bound);
        staged.0.extend(new_staged);
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(FixedUpdate, desugar_until);
}
