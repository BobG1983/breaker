use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{super::system::*, helpers::*};
use crate::{
    effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::BoundEffects,
        types::{Condition, EffectType, ScopedTree, Tree, Trigger, TriggerContext},
    },
    state::types::NodeState,
};

// ================================================================
// Shape C: During(Cond, When(Trigger, Fire(reversible)))
// ================================================================

// ----------------------------------------------------------------
// Behavior 1: Cond entering true installs a Tree::When armed entry
//             into BoundEffects with #armed[0] key
// ----------------------------------------------------------------

#[test]
fn shape_c_cond_entering_true_installs_armed_when_entry() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should contain 2 entries: original During + armed When"
    );

    // Original During entry preserved
    assert_eq!(bound.0[0].0, "chip_siege");
    assert!(
        matches!(bound.0[0].1, Tree::During(..)),
        "First entry should still be the original During tree"
    );

    // Armed When entry installed
    let armed = bound
        .0
        .iter()
        .find(|(name, _)| name == "chip_siege#armed[0]");
    assert!(
        armed.is_some(),
        "Should find armed When with key 'chip_siege#armed[0]'"
    );
    let (_, armed_tree) = armed.unwrap();
    assert!(
        matches!(armed_tree, Tree::When(Trigger::Bumped, _)),
        "Armed entry should be a Tree::When(Bumped, ...)"
    );

    // DuringActive should contain the source
    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should exist");
    assert!(
        da.0.contains("chip_siege"),
        "DuringActive should contain 'chip_siege'"
    );
}

// Edge case: idempotent — running evaluate_conditions twice does not duplicate armed entry
#[test]
fn shape_c_armed_entry_not_duplicated_on_second_evaluation() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);
    evaluate_conditions(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should contain exactly 2 entries after two evaluations (not 3)"
    );
}

// ----------------------------------------------------------------
// Behavior 2: Cond staying true does not re-install the armed entry
// ----------------------------------------------------------------

#[test]
fn shape_c_cond_staying_true_does_not_reinstall_armed_entry() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    // Run 5 times with condition staying true
    for _ in 0..5 {
        evaluate_conditions(&mut world);
    }

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        2,
        "After 5 evaluations with condition staying true, BoundEffects should still have exactly 2 entries"
    );
}

// ----------------------------------------------------------------
// Behavior 3: Firing the trigger while Cond is true dispatches
//             the inner fire
// ----------------------------------------------------------------

#[test]
fn shape_c_trigger_while_armed_dispatches_inner_fire() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    // Arm the When entry
    evaluate_conditions(&mut world);

    // Fire the trigger
    walk_entity_effects(&mut world, entity, &Trigger::Bumped, &TriggerContext::None);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack<SpeedBoostConfig> should exist after trigger fires");
    assert_eq!(stack.len(), 1, "Stack should have exactly 1 entry");

    // Verify source is the armed key
    let entry = stack.iter().next().unwrap();
    assert_eq!(
        entry.0, "chip_siege#armed[0]",
        "Stack entry source must be the armed key, not the original source"
    );
    assert_eq!(
        entry.1,
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
        "Config should match the original"
    );
}

// ----------------------------------------------------------------
// Behavior 4: Firing the trigger multiple times stacks multiple pushes
// ----------------------------------------------------------------

#[test]
fn shape_c_multiple_trigger_fires_stack_entries() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire 3 times
    for _ in 0..3 {
        walk_entity_effects(&mut world, entity, &Trigger::Bumped, &TriggerContext::None);
    }

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist");
    assert_eq!(
        stack.len(),
        3,
        "Stack should have 3 entries from 3 trigger fires"
    );

    for entry in stack.iter() {
        assert_eq!(
            entry.0, "chip_siege#armed[0]",
            "All stack entries should have source 'chip_siege#armed[0]'"
        );
    }
}

// Edge case: zero triggers fired while armed — no stack
#[test]
fn shape_c_zero_triggers_while_armed_no_stack() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);

    // Precondition: armed entry must be installed
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Precondition: armed entry must be installed before testing zero-trigger case"
    );

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
        "No EffectStack should exist when no triggers have been fired"
    );
}

// ----------------------------------------------------------------
// Behavior 5: Non-matching trigger does not fire the armed entry
// ----------------------------------------------------------------

#[test]
fn shape_c_non_matching_trigger_does_not_fire_armed_entry() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);

    // Precondition: armed entry must exist for the non-matching test to be meaningful
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Precondition: armed entry must be installed before testing non-matching trigger"
    );

    // Fire with non-matching trigger
    walk_entity_effects(
        &mut world,
        entity,
        &Trigger::BoltLostOccurred,
        &TriggerContext::None,
    );

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
        "No EffectStack should exist when trigger does not match"
    );
}

// Edge case: PerfectBumped does not match Bumped gate
#[test]
fn shape_c_perfect_bumped_does_not_match_bumped_gate() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);

    // Precondition: armed entry must exist
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Precondition: armed entry must be installed before testing non-matching trigger"
    );

    walk_entity_effects(
        &mut world,
        entity,
        &Trigger::PerfectBumped,
        &TriggerContext::None,
    );

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
        "PerfectBumped should not match Bumped gate"
    );
}

// ----------------------------------------------------------------
// Behavior 6: Cond exiting true removes armed entry AND reverses
//             all stacked effects
// ----------------------------------------------------------------

#[test]
fn shape_c_cond_exiting_true_removes_armed_entry_and_reverses_stack() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire trigger twice to stack 2 entries
    for _ in 0..2 {
        walk_entity_effects(&mut world, entity, &Trigger::Bumped, &TriggerContext::None);
    }

    // Verify precondition: stack has 2 entries
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .unwrap()
            .len(),
        2,
        "Precondition: stack should have 2 entries before disarm"
    );

    // Condition becomes false — disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Armed entry should be removed
    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have 1 entry (original During only)"
    );
    assert_eq!(bound.0[0].0, "chip_siege");
    assert!(
        matches!(bound.0[0].1, Tree::During(..)),
        "Remaining entry must be the original During tree"
    );

    // Stack should be empty (reversed)
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Stack component should still exist");
    assert!(
        stack.is_empty(),
        "Stack should be empty after disarm reversal"
    );

    // DuringActive should no longer contain the source
    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should still exist");
    assert!(
        !da.0.contains("chip_siege"),
        "DuringActive should no longer contain 'chip_siege'"
    );
}

// Edge case: original During tree is NOT removed
#[test]
fn shape_c_original_during_tree_not_removed_on_disarm() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    // Arm
    evaluate_conditions(&mut world);

    // Precondition: armed entry must have been installed (proves arm phase worked)
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Precondition: armed entry must be installed before testing disarm"
    );

    // Disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert!(
        bound
            .0
            .iter()
            .any(|(name, tree)| name == "chip_siege" && matches!(tree, Tree::During(..))),
        "Original During tree entry must NOT be removed during disarm"
    );
}

// ----------------------------------------------------------------
// Behavior 7: Firing the trigger after disarm is a no-op
// ----------------------------------------------------------------

#[test]
fn shape_c_trigger_after_disarm_is_noop() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire once to create a stack entry
    walk_entity_effects(&mut world, entity, &Trigger::Bumped, &TriggerContext::None);

    // Disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Fire trigger 3 times after disarm
    for _ in 0..3 {
        walk_entity_effects(&mut world, entity, &Trigger::Bumped, &TriggerContext::None);
    }

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Stack component should still exist");
    assert!(
        stack.is_empty(),
        "Stack should remain empty after trigger fires post-disarm"
    );

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have only 1 entry (original During) after disarm"
    );
}

// ----------------------------------------------------------------
// Behavior 8: Re-entering Cond true re-arms with a fresh armed entry
// ----------------------------------------------------------------

#[test]
fn shape_c_re_entering_true_rearms_with_fresh_entry() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire trigger twice
    for _ in 0..2 {
        walk_entity_effects(&mut world, entity, &Trigger::Bumped, &TriggerContext::None);
    }

    // Disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Re-arm
    world.insert_resource(State::new(NodeState::Playing));
    evaluate_conditions(&mut world);

    // Verify re-armed
    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should have 2 entries (original During + fresh armed When)"
    );

    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should exist");
    assert!(
        da.0.contains("chip_siege"),
        "DuringActive should contain 'chip_siege' after re-arm"
    );

    // Stack should still be empty (no new triggers fired)
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Stack component should still exist");
    assert!(
        stack.is_empty(),
        "Stack should be empty after re-arm (no new triggers)"
    );

    // Fire once after re-arm — should create 1 entry (not accumulate with old ones)
    walk_entity_effects(&mut world, entity, &Trigger::Bumped, &TriggerContext::None);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Stack should exist after fire");
    assert_eq!(
        stack.len(),
        1,
        "After re-arm and 1 trigger fire, stack should have exactly 1 entry"
    );
}

// ----------------------------------------------------------------
// Behavior 9: Full lifecycle: arm -> fire 2x -> disarm -> re-arm
//             -> fire 1x -> disarm
// ----------------------------------------------------------------

#[test]
fn shape_c_full_lifecycle() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    // Step 1: evaluate_conditions (arm)
    evaluate_conditions(&mut world);
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(bound.0.len(), 2, "Step 1: armed entry should be installed");

    // Step 2: walk_effects Bumped 2x
    for _ in 0..2 {
        walk_entity_effects(&mut world, entity, &Trigger::Bumped, &TriggerContext::None);
    }
    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
    assert_eq!(stack.len(), 2, "Step 2: stack should have 2 entries");

    // Step 3: set Loading + evaluate_conditions (disarm)
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Step 3: armed entry removed, only During remains"
    );
    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
    assert!(stack.is_empty(), "Step 3: stack should be empty");

    // Step 4: set Playing + evaluate_conditions (re-arm)
    world.insert_resource(State::new(NodeState::Playing));
    evaluate_conditions(&mut world);
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Step 4: armed entry should be re-installed"
    );

    // Step 5: walk_effects Bumped 1x
    walk_entity_effects(&mut world, entity, &Trigger::Bumped, &TriggerContext::None);
    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
    assert_eq!(stack.len(), 1, "Step 5: stack should have 1 entry");

    // Step 6: set Loading + evaluate_conditions (disarm)
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(bound.0.len(), 1, "Step 6: armed entry removed again");
    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
    assert!(stack.is_empty(), "Step 6: stack should be empty again");
}

// ----------------------------------------------------------------
// Behavior 10: During tree persists through arm/disarm cycles
// ----------------------------------------------------------------

#[test]
fn shape_c_during_tree_persists_through_arm_disarm_cycles() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    // Arm
    evaluate_conditions(&mut world);

    // Precondition: armed entry must have been installed
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Precondition: armed entry must be installed before lifecycle test"
    );

    // Fire -> disarm -> re-arm -> fire -> disarm
    walk_entity_effects(&mut world, entity, &Trigger::Bumped, &TriggerContext::None);
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);
    world.insert_resource(State::new(NodeState::Playing));
    evaluate_conditions(&mut world);
    walk_entity_effects(&mut world, entity, &Trigger::Bumped, &TriggerContext::None);
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should still exist");
    assert_eq!(
        bound.0.len(),
        1,
        "After full lifecycle, BoundEffects should contain exactly 1 entry"
    );
    assert_eq!(bound.0[0].0, "chip_siege");
    assert!(
        matches!(bound.0[0].1, Tree::During(..)),
        "The remaining entry should still be the original During tree"
    );
}

// ----------------------------------------------------------------
// Behavior 11: Despawned entity during armed phase does not panic
// ----------------------------------------------------------------

#[test]
fn shape_c_despawned_entity_during_armed_phase_does_not_panic() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    // Arm
    evaluate_conditions(&mut world);

    // Precondition: armed entry must be installed
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Precondition: armed entry must be installed before despawn test"
    );

    // Fire to create stack entry
    walk_entity_effects(&mut world, entity, &Trigger::Bumped, &TriggerContext::None);

    // Despawn
    world.despawn(entity);

    // Should not panic
    evaluate_conditions(&mut world);
}

// ----------------------------------------------------------------
// Behavior 12: Multiple Shape C entries with different triggers
//              on same entity are independent
// ----------------------------------------------------------------

#[test]
fn shape_c_multiple_entries_with_different_triggers_are_independent() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![
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
            ),
            (
                "chip_rush".to_string(),
                Tree::During(
                    Condition::NodeActive,
                    Box::new(ScopedTree::When(
                        Trigger::BoltLostOccurred,
                        Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                            multiplier: OrderedFloat(2.0),
                        }))),
                    )),
                ),
            ),
        ]))
        .id();

    // Arm both
    evaluate_conditions(&mut world);

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        4,
        "BoundEffects should have 4 entries: 2 During originals + 2 armed entries"
    );

    // Fire only Bumped trigger
    walk_entity_effects(&mut world, entity, &Trigger::Bumped, &TriggerContext::None);

    // SpeedBoost should have 1 entry from chip_siege#armed[0]
    let speed_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("SpeedBoost stack should exist");
    assert_eq!(
        speed_stack.len(),
        1,
        "SpeedBoost stack should have 1 entry from Bumped trigger"
    );

    // DamageBoost should not exist (BoltLostOccurred trigger not fired)
    assert!(
        world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .is_none(),
        "DamageBoost stack should not exist (BoltLostOccurred not fired)"
    );
}
