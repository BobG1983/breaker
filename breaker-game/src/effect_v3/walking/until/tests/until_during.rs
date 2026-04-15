use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use super::super::system::*;
use crate::{
    effect_v3::{
        conditions::{DuringActive, evaluate_conditions},
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::BoundEffects,
        types::{Condition, ReversibleEffectType, ScopedTree, Tree, Trigger, TriggerContext},
        walking::walk_effects::walk_bound_effects,
    },
    state::types::NodeState,
};

// ================================================================
// Wave 7b — Part C: Shape B — Until(X, During(Cond, inner))
// ================================================================

// ----------------------------------------------------------------
// Behavior 13: Until with During inner installs the During into
//              BoundEffects on first walk
// ----------------------------------------------------------------

#[test]
fn until_with_during_inner_installs_during_on_first_walk() {
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

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should contain 2 entries: original Until + installed During"
    );

    let installed = bound
        .0
        .iter()
        .find(|(name, _)| name == "chip_shield#installed[0]");
    assert!(
        installed.is_some(),
        "Should find installed During with key 'chip_shield#installed[0]'"
    );
    let (_, installed_tree) = installed.unwrap();
    assert!(
        matches!(installed_tree, Tree::During(Condition::NodeActive, _)),
        "Installed entry should be a Tree::During(NodeActive, ...)"
    );

    let ua = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should exist");
    assert!(
        ua.0.contains("chip_shield"),
        "UntilApplied should contain 'chip_shield'"
    );
}

// ----------------------------------------------------------------
// Behavior 14: Installed During from Until polls normally via
//              condition poller
// ----------------------------------------------------------------

#[test]
fn installed_during_from_until_polls_normally_via_condition_poller() {
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

    // Walk with non-matching trigger — installs During
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

    // Poll conditions — should fire installed During
    evaluate_conditions(&mut world);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after condition poller fires installed During");
    assert_eq!(
        stack.len(),
        1,
        "Installed During from Until should fire on condition poll"
    );

    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should exist");
    assert!(
        da.0.contains("chip_shield#installed[0]"),
        "DuringActive should contain the installed key"
    );
}

// ----------------------------------------------------------------
// Behavior 15: Installed During from Until cycles normally
//              (fire + reverse on condition transitions)
// ----------------------------------------------------------------

#[test]
fn installed_during_from_until_cycles_normally_on_condition_transitions() {
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

    // Condition true: fire
    evaluate_conditions(&mut world);

    // Condition false: reverse
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Condition true: re-fire
    world.insert_resource(State::new(NodeState::Playing));
    evaluate_conditions(&mut world);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Stack should exist after re-fire");
    assert_eq!(
        stack.len(),
        1,
        "After full condition cycle, stack should have 1 entry"
    );

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should still exist");
    assert!(
        bound
            .0
            .iter()
            .any(|(name, _)| name == "chip_shield#installed[0]"),
        "Installed During should still be in BoundEffects after condition cycling"
    );

    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should exist");
    assert!(
        da.0.contains("chip_shield#installed[0]"),
        "DuringActive should contain the installed key after re-fire"
    );
}

// ----------------------------------------------------------------
// Behavior 16: Until gate trigger fires — uninstalls During from
//              BoundEffects
// ----------------------------------------------------------------

#[test]
fn until_gate_trigger_uninstalls_during_from_bound_effects() {
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
    let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

    // First walk: install During (non-matching trigger)
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

    // Verify installation precondition
    assert_eq!(
        world.get::<BoundEffects>(entity).unwrap().0.len(),
        2,
        "Precondition: BoundEffects should have 2 entries (original Until + installed During)"
    );

    // Gate trigger fires: should uninstall both
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

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should still exist");
    assert!(
        !bound
            .0
            .iter()
            .any(|(name, _)| name == "chip_shield#installed[0]"),
        "Installed During should be removed after gate trigger fires"
    );
    assert!(
        !bound.0.iter().any(|(name, _)| name == "chip_shield"),
        "Original Until entry should also be removed after gate trigger fires"
    );

    let ua = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should still exist");
    assert!(
        !ua.0.contains("chip_shield"),
        "UntilApplied should no longer contain 'chip_shield'"
    );
}

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

// ----------------------------------------------------------------
// Behavior 20: Until fires on same trigger as installation walk —
//              net result is full cleanup
// ----------------------------------------------------------------

#[test]
fn until_with_during_fires_on_same_trigger_as_installation_walk_full_cleanup() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_instant".to_string(),
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

    // Walk with gate trigger on first walk
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        walk_bound_effects(
            entity,
            &Trigger::Bumped,
            &TriggerContext::None,
            &trees,
            &mut commands,
        );
    }
    queue.apply(&mut world);

    // Everything should be cleaned up
    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should still exist");
    assert!(
        !bound
            .0
            .iter()
            .any(|(name, _)| name == "chip_instant#installed[0]"),
        "Installed During should not exist after immediate gate match"
    );
    assert!(
        !bound.0.iter().any(|(name, _)| name == "chip_instant"),
        "Original Until entry should not exist after immediate gate match"
    );

    let ua = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should exist");
    assert!(
        !ua.0.contains("chip_instant"),
        "UntilApplied should not contain 'chip_instant'"
    );

    // No effects should have been fired (During was never polled by evaluate_conditions)
    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
        "No EffectStack should exist — the During was never polled by evaluate_conditions"
    );
}

// ----------------------------------------------------------------
// Behavior 21: Until with During — entity despawned between walks
//              does not panic
// ----------------------------------------------------------------

#[test]
fn until_with_during_entity_despawned_between_walks_does_not_panic() {
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
    let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

    // First walk: install During
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

    // Snapshot trees before despawn
    let trees_before_despawn = world.get::<BoundEffects>(entity).unwrap().0.clone();

    // Despawn entity
    world.despawn(entity);

    // Walk with gate trigger on despawned entity — should not panic
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::Bumped,
            &TriggerContext::None,
            &trees_before_despawn,
            &mut commands,
        );
    }
    queue2.apply(&mut world);
    // No panic = test passes
}

// ----------------------------------------------------------------
// Behavior 22: Until with During + Sequence inner installs the
//              During correctly
// ----------------------------------------------------------------

#[test]
fn until_with_during_sequence_inner_installs_correctly() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_dual".to_string(),
        Tree::Until(
            Trigger::Bumped,
            Box::new(ScopedTree::During(
                Condition::NodeActive,
                Box::new(ScopedTree::Sequence(vec![
                    ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }),
                    ReversibleEffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    }),
                ])),
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

    // Verify installation
    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    let installed = bound
        .0
        .iter()
        .find(|(name, _)| name == "chip_dual#installed[0]");
    assert!(
        installed.is_some(),
        "Installed During with Sequence inner should exist"
    );

    // Poll conditions — should fire both effects
    evaluate_conditions(&mut world);

    let speed_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("SpeedBoost stack should exist");
    assert_eq!(
        speed_stack.len(),
        1,
        "SpeedBoost should have 1 entry from Sequence inner"
    );

    let damage_stack = world
        .get::<EffectStack<DamageBoostConfig>>(entity)
        .expect("DamageBoost stack should exist");
    assert_eq!(
        damage_stack.len(),
        1,
        "DamageBoost should have 1 entry from Sequence inner"
    );
}
