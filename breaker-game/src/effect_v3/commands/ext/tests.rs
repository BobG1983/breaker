use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use super::system::*;
use crate::effect_v3::{
    effects::{DamageBoostConfig, PiercingConfig, SizeBoostConfig, SpeedBoostConfig},
    stacking::EffectStack,
    storage::{BoundEffects, StagedEffects},
    traits::Fireable,
    types::{EffectType, ReversibleEffectType, RouteType, Tree},
};

// ── fire_effect ───────────────────────────────────────────────────

#[test]
fn fire_effect_queues_fire_command() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        commands.fire_effect(
            entity,
            EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }),
            "ext_test".to_owned(),
        );
    }
    queue.apply(&mut world);

    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
    assert_eq!(stack.len(), 1);
}

#[test]
fn fire_effect_twice_stacks_both_entries() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        commands.fire_effect(
            entity,
            EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }),
            "ext_test".to_owned(),
        );
        commands.fire_effect(
            entity,
            EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            }),
            "ext_test".to_owned(),
        );
    }
    queue.apply(&mut world);

    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
    assert_eq!(stack.len(), 2);
}

// ── reverse_effect ────────────────────────────────────────────────

#[test]
fn reverse_effect_queues_reverse_command() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let config = SpeedBoostConfig {
        multiplier: OrderedFloat(1.5),
    };

    // Fire first to create the stack.
    config.fire(entity, "ext_test", &mut world);
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .unwrap()
            .len(),
        1
    );

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        commands.reverse_effect(
            entity,
            ReversibleEffectType::SpeedBoost(config),
            "ext_test".to_owned(),
        );
    }
    queue.apply(&mut world);

    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
    assert!(stack.is_empty());
}

#[test]
fn reverse_effect_does_not_panic_for_nonexistent_effect() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        commands.reverse_effect(
            entity,
            ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }),
            "ext_test".to_owned(),
        );
    }
    queue.apply(&mut world);

    // No panic — pass.
}

// ── stamp_effect ──────────────────────────────────────────────────

#[test]
fn stamp_effect_queues_stamp_command() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        commands.stamp_effect(
            entity,
            "stamp_ext".to_owned(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 2 })),
        );
    }
    queue.apply(&mut world);

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, "stamp_ext");
}

#[test]
fn stamp_effect_twice_appends_both_entries() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        commands.stamp_effect(
            entity,
            "stamp_ext".to_owned(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 2 })),
        );
        commands.stamp_effect(
            entity,
            "stamp_ext".to_owned(),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 3 })),
        );
    }
    queue.apply(&mut world);

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(bound.0.len(), 2);
}

// ── stage_effect ──────────────────────────────────────────────────

#[test]
fn stage_effect_queues_stage_command() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        commands.stage_effect(
            entity,
            "stage_ext".to_owned(),
            Tree::Fire(EffectType::SizeBoost(SizeBoostConfig {
                multiplier: OrderedFloat(1.3),
            })),
        );
    }
    queue.apply(&mut world);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(staged.0.len(), 1);
    assert_eq!(staged.0[0].0, "stage_ext");
}

#[test]
fn stage_effect_twice_with_different_names_appends_both() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        commands.stage_effect(
            entity,
            "stage_a".to_owned(),
            Tree::Fire(EffectType::SizeBoost(SizeBoostConfig {
                multiplier: OrderedFloat(1.3),
            })),
        );
        commands.stage_effect(
            entity,
            "stage_b".to_owned(),
            Tree::Fire(EffectType::SizeBoost(SizeBoostConfig {
                multiplier: OrderedFloat(1.5),
            })),
        );
    }
    queue.apply(&mut world);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(staged.0.len(), 2);
}

// ── route_effect ──────────────────────────────────────────────────

#[test]
fn route_effect_bound_queues_route_to_bound_effects() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        commands.route_effect(
            entity,
            "route_bound".to_owned(),
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(1.7),
            })),
            RouteType::Bound,
        );
    }
    queue.apply(&mut world);

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, "route_bound");
}

#[test]
fn route_effect_staged_queues_route_to_staged_effects() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        commands.route_effect(
            entity,
            "route_staged".to_owned(),
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(1.7),
            })),
            RouteType::Staged,
        );
    }
    queue.apply(&mut world);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(staged.0.len(), 1);
    assert_eq!(staged.0[0].0, "route_staged");
}

// ── remove_effect ─────────────────────────────────────────────────

#[test]
fn remove_effect_queues_remove_command() {
    let mut world = World::new();
    let tree = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
        multiplier: OrderedFloat(1.5),
    }));
    let entity = world
        .spawn(BoundEffects(vec![("remove_me".to_owned(), tree)]))
        .id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        commands.remove_effect(entity, "remove_me");
    }
    queue.apply(&mut world);

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(bound.0.len(), 0);
}

#[test]
fn remove_effect_with_no_matching_name_leaves_bound_unchanged() {
    let mut world = World::new();
    let tree = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
        multiplier: OrderedFloat(1.5),
    }));
    let entity = world
        .spawn(BoundEffects(vec![("keep_me".to_owned(), tree)]))
        .id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        commands.remove_effect(entity, "nonexistent");
    }
    queue.apply(&mut world);

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(bound.0.len(), 1);
}
