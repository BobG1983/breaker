use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use super::system::*;
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
// Wave 7b — Part B: Shape A — When(X, During(Cond, inner))
// ================================================================

// ----------------------------------------------------------------
// Behavior 6: evaluate_during installs Tree::During into BoundEffects
//             with install key
// ----------------------------------------------------------------

#[test]
fn evaluate_during_installs_tree_during_into_bound_effects_with_install_key() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_siege".to_string(),
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::During(
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
            &Trigger::Bumped,
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
        "BoundEffects should contain 2 entries: original When + installed During"
    );

    // Original When entry preserved
    assert_eq!(bound.0[0].0, "chip_siege");
    assert!(
        matches!(bound.0[0].1, Tree::When(..)),
        "First entry should still be the original When tree"
    );

    // Installed During entry
    let installed = bound
        .0
        .iter()
        .find(|(name, _)| name == "chip_siege#installed[0]");
    assert!(
        installed.is_some(),
        "Should find installed During with key 'chip_siege#installed[0]'"
    );
    let (_, installed_tree) = installed.unwrap();
    assert!(
        matches!(installed_tree, Tree::During(Condition::NodeActive, _)),
        "Installed entry should be a Tree::During(NodeActive, ...)"
    );
}

// ----------------------------------------------------------------
// Behavior 7: Installed During is picked up by condition poller on
//             next evaluation
// ----------------------------------------------------------------

#[test]
fn installed_during_is_picked_up_by_condition_poller() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_siege".to_string(),
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::During(
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

    // Walk with matching trigger — installs the During
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

    // Poll conditions — should find installed During and fire it
    evaluate_conditions(&mut world);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after condition poller fires installed During");
    assert_eq!(
        stack.len(),
        1,
        "Installed During should fire on first poll after installation"
    );

    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should exist");
    assert!(
        da.0.contains("chip_siege#installed[0]"),
        "DuringActive should contain the exact installed key"
    );
}

// ----------------------------------------------------------------
// Behavior 8: Second fire of When trigger is idempotent — does not
//             duplicate installed During
// ----------------------------------------------------------------

#[test]
fn second_when_fire_does_not_duplicate_installed_during() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_siege".to_string(),
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::During(
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

    // First walk with matching trigger
    let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
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

    // Second walk with matching trigger
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
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should contain exactly 2 entries (original When + 1 installed During), not 3"
    );
}

// Edge case: 5 fires of When trigger still yields exactly 2 entries
#[test]
fn five_when_fires_still_yield_exactly_two_bound_effects_entries() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_siege".to_string(),
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::During(
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

    for _ in 0..5 {
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
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
    }

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        2,
        "After 5 When fires, BoundEffects should still have exactly 2 entries"
    );
}

// ----------------------------------------------------------------
// Behavior 9: Installed During persists after reversal cycles of
//             condition polling
// ----------------------------------------------------------------

#[test]
fn installed_during_persists_after_reversal_cycles() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_siege".to_string(),
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::During(
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

    // Walk: installs the During
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

    // Condition true: fire
    evaluate_conditions(&mut world);

    // Condition false: reverse
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Condition true: re-fire
    world.insert_resource(State::new(NodeState::Playing));
    evaluate_conditions(&mut world);

    // Both the When and installed During should still be in BoundEffects
    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should still exist");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should contain both original When and installed During after condition cycling"
    );

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Stack should exist after re-fire");
    assert_eq!(
        stack.len(),
        1,
        "Stack should have 1 entry after true->false->true cycle"
    );
}

// ----------------------------------------------------------------
// Behavior 10: evaluate_during with non-matching trigger does nothing
// ----------------------------------------------------------------

#[test]
fn non_matching_trigger_does_not_install_during() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_siege".to_string(),
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::During(
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
            &Trigger::BoltLostOccurred,
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
        1,
        "BoundEffects should still have exactly 1 entry (no installed During)"
    );
    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
        "No EffectStack should exist when trigger does not match When gate"
    );
}

// ----------------------------------------------------------------
// Behavior 11: evaluate_during on entity without BoundEffects
//              does not panic
// ----------------------------------------------------------------

#[test]
fn evaluate_during_on_entity_without_bound_effects_does_not_panic() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let condition = Condition::NodeActive;
    let inner = ScopedTree::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
        multiplier: OrderedFloat(1.5),
    }));

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        evaluate_during(
            entity,
            &condition,
            &inner,
            &TriggerContext::None,
            "chip_test",
            &mut commands,
        );
    }
    queue.apply(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be created by evaluate_during even on bare entity");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should contain exactly 1 installed During"
    );
    let (key, _) = &bound.0[0];
    assert_eq!(
        key, "chip_test#installed[0]",
        "Install key should be 'chip_test#installed[0]'"
    );
}

// ----------------------------------------------------------------
// Behavior 12: Multiple different When gates with nested Durings
//              produce distinct installed entries
// ----------------------------------------------------------------

#[test]
fn multiple_when_gates_produce_distinct_installed_during_entries() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![
        (
            "chip_a".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::During(
                    Condition::NodeActive,
                    Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                        SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        },
                    ))),
                )),
            ),
        ),
        (
            "chip_b".to_string(),
            Tree::When(
                Trigger::PerfectBumped,
                Box::new(Tree::During(
                    Condition::ShieldActive,
                    Box::new(ScopedTree::Fire(ReversibleEffectType::DamageBoost(
                        DamageBoostConfig {
                            multiplier: OrderedFloat(2.0),
                        },
                    ))),
                )),
            ),
        ),
    ]);
    world.entity_mut(entity).insert(bound);

    // Walk with Trigger::Bumped — installs chip_a's During
    let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
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

    // Walk with Trigger::PerfectBumped — installs chip_b's During
    let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::PerfectBumped,
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
        4,
        "BoundEffects should contain 4 entries: 2 original When + 2 installed Durings"
    );

    assert!(
        bound
            .0
            .iter()
            .any(|(name, _)| name == "chip_a#installed[0]"),
        "Should find installed During for chip_a"
    );
    assert!(
        bound
            .0
            .iter()
            .any(|(name, _)| name == "chip_b#installed[0]"),
        "Should find installed During for chip_b"
    );
}
