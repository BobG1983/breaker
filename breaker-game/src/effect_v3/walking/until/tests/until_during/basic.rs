use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use super::super::super::system::*;
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
