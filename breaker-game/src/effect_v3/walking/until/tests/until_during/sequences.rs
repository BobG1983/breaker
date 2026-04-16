use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use crate::{
    effect_v3::{
        conditions::{DuringActive, evaluate_conditions},
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::BoundEffects,
        types::{Condition, ReversibleEffectType, ScopedTree, Tree, Trigger, TriggerContext},
        walking::walk_effects::walk_bound_effects,
    },
    state::types::NodeState,
};

// ----------------------------------------------------------------
// Behavior 17: Until gate trigger fires — reverses currently-active
//              During effects
// ----------------------------------------------------------------

#[test]
fn until_gate_trigger_reverses_active_during_effects() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_shield".to_string(),
        Tree::Until(
            Trigger::Bumped,
            Box::new(ScopedTree::During(
                Condition::NodeActive,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            )),
        ),
    )]);
    world.entity_mut(entity).insert(bound);
    let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

    // Install During
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        walk_bound_effects(
            entity,
            &Trigger::NodeStartOccurred,
            &TriggerContext::None,
            &trees,
            &mut commands,
        );
    }
    queue.apply(&mut world);

    // Fire inner effects via condition poller
    evaluate_conditions(&mut world);

    // Precondition: effects are active
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("Stack should exist")
            .len(),
        1,
        "Precondition: SpeedBoost should be active before gate fires"
    );
    assert!(
        world
            .get::<DuringActive>(entity)
            .expect("DuringActive should exist")
            .0
            .contains("chip_shield#installed[0]"),
        "Precondition: DuringActive should contain installed key"
    );

    // Gate trigger fires: should reverse effects and clean up
    let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::Bumped,
            &TriggerContext::None,
            &trees_second,
            &mut commands,
        );
    }
    queue2.apply(&mut world);

    let stack_empty = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(
        stack_empty,
        "SpeedBoost EffectStack should be empty after Until gate trigger reverses effects"
    );

    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should still exist");
    assert!(
        !da.0.contains("chip_shield#installed[0]"),
        "DuringActive should no longer contain installed key after reversal"
    );

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should still exist");
    assert!(
        !bound
            .0
            .iter()
            .any(|(name, _)| name == "chip_shield#installed[0]"),
        "Installed During should be removed from BoundEffects"
    );
}

// ----------------------------------------------------------------
// Behavior 18: Until gate trigger fires — does NOT double-reverse
//              effects already reversed by condition transition
// ----------------------------------------------------------------

#[test]
fn until_gate_trigger_does_not_double_reverse_already_reversed_effects() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_shield".to_string(),
        Tree::Until(
            Trigger::Bumped,
            Box::new(ScopedTree::During(
                Condition::NodeActive,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            )),
        ),
    )]);
    world.entity_mut(entity).insert(bound);
    let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

    // Install During
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        walk_bound_effects(
            entity,
            &Trigger::NodeStartOccurred,
            &TriggerContext::None,
            &trees,
            &mut commands,
        );
    }
    queue.apply(&mut world);

    // Fire inner effects via condition poller (condition true)
    evaluate_conditions(&mut world);

    // Reverse via condition transition (condition false)
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Precondition: effects already reversed by condition
    let stack_empty = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(
        stack_empty,
        "Precondition: effects should already be reversed by condition transition"
    );
    assert!(
        !world
            .get::<DuringActive>(entity)
            .expect("DuringActive should exist")
            .0
            .contains("chip_shield#installed[0]"),
        "Precondition: DuringActive should not contain installed key after condition reversal"
    );

    // Gate trigger fires: should clean up without double-reverse
    let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::Bumped,
            &TriggerContext::None,
            &trees_second,
            &mut commands,
        );
    }
    queue2.apply(&mut world);

    // Effects should still be empty (no double-reverse)
    let stack_still_empty = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(
        stack_still_empty,
        "EffectStack should still be empty — no double-reverse"
    );

    // Installed During should be removed from BoundEffects
    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should still exist");
    assert!(
        !bound
            .0
            .iter()
            .any(|(name, _)| name == "chip_shield#installed[0]"),
        "Installed During should be removed from BoundEffects after gate fire"
    );
    assert!(
        !bound.0.iter().any(|(name, _)| name == "chip_shield"),
        "Original Until entry should also be removed after gate fire"
    );
}

// ----------------------------------------------------------------
// Behavior 19: Until with During inner — second non-matching walk
//              does not duplicate installation
// ----------------------------------------------------------------

#[test]
fn until_with_during_inner_second_non_matching_walk_does_not_duplicate() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_shield".to_string(),
        Tree::Until(
            Trigger::Bumped,
            Box::new(ScopedTree::During(
                Condition::NodeActive,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            )),
        ),
    )]);
    world.entity_mut(entity).insert(bound);

    // First walk: installs During
    let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        walk_bound_effects(
            entity,
            &Trigger::NodeStartOccurred,
            &TriggerContext::None,
            &trees,
            &mut commands,
        );
    }
    queue.apply(&mut world);

    // Second walk: different non-matching trigger
    let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::BoltLostOccurred,
            &TriggerContext::None,
            &trees_second,
            &mut commands,
        );
    }
    queue2.apply(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should have exactly 2 entries (original Until + 1 installed During), not more"
    );
}
