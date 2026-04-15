use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use crate::effect_v3::{
    effects::{DamageBoostConfig, SpeedBoostConfig},
    storage::BoundEffects,
    types::{
        BumpTarget, Condition, EffectType, ParticipantTarget, ReversibleEffectType, ScopedTerminal,
        ScopedTree, Tree, Trigger, TriggerContext,
    },
    walking::walk_bound_effects,
};

// ----------------------------------------------------------------
// Helper: build a During tree with a single reversible Fire effect
// ----------------------------------------------------------------

pub(super) fn during_node_speed_boost() -> (String, Tree) {
    (
        "chip_a".to_string(),
        Tree::During(
            Condition::NodeActive,
            Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                },
            ))),
        ),
    )
}

pub(super) fn during_shield_damage_boost() -> (String, Tree) {
    (
        "chip_b".to_string(),
        Tree::During(
            Condition::ShieldActive,
            Box::new(ScopedTree::Fire(ReversibleEffectType::DamageBoost(
                DamageBoostConfig {
                    multiplier: OrderedFloat(2.0),
                },
            ))),
        ),
    )
}

// ----------------------------------------------------------------
// Helper: build a Shape C During tree (When gate inside During)
// ----------------------------------------------------------------

pub(super) fn during_when_bumped_speed_boost() -> (String, Tree) {
    (
        "chip_siege".to_string(),
        Tree::During(
            Condition::NodeActive,
            Box::new(ScopedTree::When(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            )),
        ),
    )
}

// ----------------------------------------------------------------
// Helper: build a Shape D During tree (On inside During)
// ----------------------------------------------------------------

pub(super) fn during_on_bump_bolt_speed_boost() -> (String, Tree) {
    (
        "chip_redirect".to_string(),
        Tree::During(
            Condition::NodeActive,
            Box::new(ScopedTree::On(
                ParticipantTarget::Bump(BumpTarget::Bolt),
                ScopedTerminal::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                })),
            )),
        ),
    )
}

// ----------------------------------------------------------------
// Helper: walk effects on an entity using its BoundEffects
// ----------------------------------------------------------------

pub(super) fn walk_entity_effects(
    world: &mut World,
    entity: Entity,
    trigger: &Trigger,
    context: &TriggerContext,
) {
    let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, world);
        walk_bound_effects(entity, trigger, context, &trees, &mut commands);
    }
    queue.apply(world);
}
