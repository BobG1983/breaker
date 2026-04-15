use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{super::system::*, helpers::*};
use crate::{
    effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::BoundEffects,
        types::{Condition, EffectType, ReversibleEffectType, ScopedTree, Tree, Trigger},
    },
    state::types::NodeState,
};

// ----------------------------------------------------------------
// Behavior 1: During fires inner effects when condition becomes true
// ----------------------------------------------------------------

#[test]
fn during_fires_inner_effect_when_node_condition_true() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_node_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack<SpeedBoostConfig> should exist after During fires");
    assert_eq!(stack.len(), 1, "Stack should have exactly 1 entry");
}

// ----------------------------------------------------------------
// Behavior 2: During does not fire when condition is false
// ----------------------------------------------------------------

#[test]
fn during_does_not_fire_when_node_condition_false() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Loading));

    let entity = world
        .spawn(BoundEffects(vec![during_node_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
        "No EffectStack should exist when condition is false"
    );
    // DuringActive should exist on entity but NOT contain "chip_a"
    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive component should be inserted even when condition is false");
    assert!(
        !da.0.contains("chip_a"),
        "chip_a should NOT be in DuringActive when condition is false"
    );
}

// ----------------------------------------------------------------
// Behavior 3: During reverses effects when condition becomes false
// ----------------------------------------------------------------

#[test]
fn during_reverses_effects_when_condition_becomes_false() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_node_speed_boost()]))
        .id();

    // First call: condition true, effects should fire
    evaluate_conditions(&mut world);

    // Verify effects are active
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Stack should exist after first evaluation");
    assert_eq!(stack.len(), 1, "Stack should have 1 entry after fire");

    // Change condition to false
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Effects should be reversed (stack empty)
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Stack component should still exist");
    assert!(
        stack.is_empty(),
        "Stack should be empty after condition becomes false"
    );
}

// ----------------------------------------------------------------
// Behavior 4: During does not double-fire when condition stays true
// ----------------------------------------------------------------

#[test]
fn during_does_not_double_fire_when_condition_stays_true() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_node_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);
    evaluate_conditions(&mut world);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Stack should exist");
    assert_eq!(
        stack.len(),
        1,
        "Stack should still have exactly 1 entry, not 2"
    );
}

// ----------------------------------------------------------------
// Behavior 5: During cycles — re-fires when condition becomes true
//             again after being false
// ----------------------------------------------------------------

#[test]
fn during_cycles_refires_when_condition_becomes_true_again() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_node_speed_boost()]))
        .id();

    // Fire
    evaluate_conditions(&mut world);

    // Toggle off
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Toggle back on
    world.insert_resource(State::new(NodeState::Playing));
    evaluate_conditions(&mut world);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Stack should exist after re-fire");
    assert_eq!(
        stack.len(),
        1,
        "Stack should have 1 entry after condition cycles back to true"
    );
}

// ----------------------------------------------------------------
// Behavior 6: During with ScopedTree::Sequence fires all effects
// ----------------------------------------------------------------

#[test]
fn during_sequence_fires_all_effects_when_condition_true() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::During(
                Condition::NodeActive,
                Box::new(ScopedTree::Sequence(vec![
                    ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }),
                    ReversibleEffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    }),
                ])),
            ),
        )]))
        .id();

    evaluate_conditions(&mut world);

    let speed_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("SpeedBoost stack should exist");
    assert_eq!(speed_stack.len(), 1, "SpeedBoost stack should have 1 entry");

    let damage_stack = world
        .get::<EffectStack<DamageBoostConfig>>(entity)
        .expect("DamageBoost stack should exist");
    assert_eq!(
        damage_stack.len(),
        1,
        "DamageBoost stack should have 1 entry"
    );
}

// ----------------------------------------------------------------
// Behavior 7: During with ScopedTree::Sequence reverses all effects
// ----------------------------------------------------------------

#[test]
fn during_sequence_reverses_all_effects_when_condition_becomes_false() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::During(
                Condition::NodeActive,
                Box::new(ScopedTree::Sequence(vec![
                    ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }),
                    ReversibleEffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    }),
                ])),
            ),
        )]))
        .id();

    // Fire
    evaluate_conditions(&mut world);

    // Verify both stacks active
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .unwrap()
            .len(),
        1
    );

    // Toggle condition off
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    let speed_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("SpeedBoost stack should still exist as component");
    assert!(
        speed_stack.is_empty(),
        "SpeedBoost stack should be empty after reversal"
    );

    let damage_stack = world
        .get::<EffectStack<DamageBoostConfig>>(entity)
        .expect("DamageBoost stack should still exist as component");
    assert!(
        damage_stack.is_empty(),
        "DamageBoost stack should be empty after reversal"
    );
}

// ----------------------------------------------------------------
// Behavior 8: DuringActive component tracks applied state
// ----------------------------------------------------------------

#[test]
fn during_active_tracks_applied_source() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_node_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);

    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should exist after evaluation");
    assert!(
        da.0.contains("chip_a"),
        "DuringActive should contain 'chip_a' when condition is true"
    );
}

// ----------------------------------------------------------------
// Behavior 9: DuringActive source removed when condition goes false
// ----------------------------------------------------------------

#[test]
fn during_active_source_removed_when_condition_goes_false() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_node_speed_boost()]))
        .id();

    // Fire
    evaluate_conditions(&mut world);
    assert!(
        world
            .get::<DuringActive>(entity)
            .unwrap()
            .0
            .contains("chip_a"),
        "chip_a should be active"
    );

    // Toggle off
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should still exist");
    assert!(
        !da.0.contains("chip_a"),
        "chip_a should be removed from DuringActive when condition becomes false"
    );
}

// ----------------------------------------------------------------
// Behavior 10: Multiple During entries with different conditions
//              track independently
// ----------------------------------------------------------------

#[test]
fn multiple_during_entries_with_different_conditions_track_independently() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));
    // No ShieldWall entity — Shield condition is false

    let entity = world
        .spawn(BoundEffects(vec![
            during_node_speed_boost(),
            during_shield_damage_boost(),
        ]))
        .id();

    evaluate_conditions(&mut world);

    // Node condition is true → SpeedBoost should be active
    let speed_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("SpeedBoost stack should exist (Node condition true)");
    assert_eq!(
        speed_stack.len(),
        1,
        "SpeedBoost should have 1 entry (chip_a active)"
    );

    // Shield condition is false → DamageBoost should NOT be active
    assert!(
        world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .is_none(),
        "DamageBoost stack should not exist (Shield condition false)"
    );
}

// ----------------------------------------------------------------
// Behavior 11: During does NOT self-remove from BoundEffects
//              (persists permanently through condition cycling)
// ----------------------------------------------------------------

#[test]
fn during_does_not_self_remove_from_bound_effects() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_node_speed_boost()]))
        .id();

    // true: effects should fire
    evaluate_conditions(&mut world);

    // Verify effects fired (proves the system actually processed the entity)
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Stack should exist after first fire");
    assert_eq!(stack.len(), 1, "Stack should have 1 entry");

    // false: effects should reverse
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // true again: effects should re-fire
    world.insert_resource(State::new(NodeState::Playing));
    evaluate_conditions(&mut world);

    // BoundEffects should still contain the During entry (not self-removed)
    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should still exist");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should still contain the chip_a During entry"
    );
    assert_eq!(bound.0[0].0, "chip_a");
    assert!(
        matches!(bound.0[0].1, Tree::During(..)),
        "Entry should still be a During variant"
    );
}

// ----------------------------------------------------------------
// Behavior 12: System skips entities with no During entries
// ----------------------------------------------------------------

#[test]
fn system_skips_entities_with_no_during_entries() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    // Entity A: has a Fire tree (not During) — should be skipped
    let entity_fire = world
        .spawn(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            })),
        )]))
        .id();

    // Entity B: has a During tree — should be processed (proves system runs)
    let entity_during = world
        .spawn(BoundEffects(vec![during_node_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);

    // Entity A: no EffectStack should be created — During processing only
    assert!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity_fire)
            .is_none(),
        "System should not fire non-During entries"
    );

    // Entity B: EffectStack should exist — proves the system actually ran
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity_during)
        .expect("During entity should have EffectStack (proves system ran)");
    assert_eq!(stack.len(), 1, "During entity stack should have 1 entry");
}

// ----------------------------------------------------------------
// Behavior 13: System handles despawned entities without panic
// ----------------------------------------------------------------

#[test]
fn system_handles_despawned_entities_without_panic() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_node_speed_boost()]))
        .id();

    // First evaluation: condition true, should fire and create DuringActive
    evaluate_conditions(&mut world);

    // Verify the system actually processed the entity
    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should exist after first evaluation");
    assert!(
        da.0.contains("chip_a"),
        "chip_a should be active before despawn"
    );

    // Despawn entity
    world.despawn(entity);

    // Second evaluation should not panic despite entity being gone
    evaluate_conditions(&mut world);
}

// ================================================================
// Wave 7b — Part A: Condition Poller Recursive Walk
// ================================================================

// ----------------------------------------------------------------
// Behavior 1 (regression lock): Nested During inside When is NOT
//            polled by condition poller
// ----------------------------------------------------------------

#[test]
fn nested_during_inside_when_is_not_polled_by_condition_poller() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![(
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
        )]))
        .id();

    evaluate_conditions(&mut world);

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
        "No EffectStack should exist — the During is nested inside a When gate and should not be polled"
    );
}

// ----------------------------------------------------------------
// Behavior 2: Installed nested During (from Shape A) is polled
//             by condition poller
// ----------------------------------------------------------------

#[test]
fn installed_nested_during_is_polled_by_condition_poller() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![
            (
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
            ),
            (
                "chip_siege#installed[0]".to_string(),
                Tree::During(
                    Condition::NodeActive,
                    Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                        SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        },
                    ))),
                ),
            ),
        ]))
        .id();

    evaluate_conditions(&mut world);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack<SpeedBoostConfig> should exist after installed During is polled");
    assert_eq!(
        stack.len(),
        1,
        "Installed During should fire exactly 1 entry"
    );

    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should exist");
    assert!(
        da.0.contains("chip_siege#installed[0]"),
        "DuringActive should contain the install key 'chip_siege#installed[0]'"
    );
}

// ----------------------------------------------------------------
// Behavior 3: Installed nested During reverses when condition
//             becomes false
// ----------------------------------------------------------------

#[test]
fn installed_nested_during_reverses_when_condition_becomes_false() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![(
            "chip_siege#installed[0]".to_string(),
            Tree::During(
                Condition::NodeActive,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]))
        .id();

    // Fire: condition true
    evaluate_conditions(&mut world);
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("Stack should exist after fire")
            .len(),
        1,
        "Precondition: stack should have 1 entry after fire"
    );

    // Reverse: condition false
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Stack component should still exist");
    assert!(
        stack.is_empty(),
        "Stack should be empty after condition becomes false"
    );

    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should still exist");
    assert!(
        !da.0.contains("chip_siege#installed[0]"),
        "DuringActive should no longer contain 'chip_siege#installed[0]'"
    );
}

// ----------------------------------------------------------------
// Behavior 4: Multiple installed Durings with different install
//             keys track independently
// ----------------------------------------------------------------

#[test]
fn multiple_installed_durings_with_different_keys_track_independently() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));
    // No ShieldWall entity — Shield condition is false

    let entity = world
        .spawn(BoundEffects(vec![
            (
                "chip_siege#installed[0]".to_string(),
                Tree::During(
                    Condition::NodeActive,
                    Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                        SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        },
                    ))),
                ),
            ),
            (
                "chip_guard#installed[0]".to_string(),
                Tree::During(
                    Condition::ShieldActive,
                    Box::new(ScopedTree::Fire(ReversibleEffectType::DamageBoost(
                        DamageBoostConfig {
                            multiplier: OrderedFloat(2.0),
                        },
                    ))),
                ),
            ),
            (
                "chip_other".to_string(),
                Tree::During(
                    Condition::NodeActive,
                    Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                        SpeedBoostConfig {
                            multiplier: OrderedFloat(1.3),
                        },
                    ))),
                ),
            ),
        ]))
        .id();

    evaluate_conditions(&mut world);

    // NodeActive is true: chip_siege#installed[0] and chip_other should fire SpeedBoost
    let speed_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("SpeedBoost stack should exist");
    assert_eq!(
        speed_stack.len(),
        2,
        "SpeedBoost stack should have 2 entries (from chip_siege#installed[0] and chip_other)"
    );

    // ShieldActive is false: chip_guard#installed[0] should NOT fire DamageBoost
    assert!(
        world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .is_none(),
        "DamageBoost stack should not exist (ShieldActive is false)"
    );

    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should exist");
    assert!(
        da.0.contains("chip_siege#installed[0]"),
        "DuringActive should contain 'chip_siege#installed[0]'"
    );
    assert!(
        da.0.contains("chip_other"),
        "DuringActive should contain 'chip_other'"
    );
    assert!(
        !da.0.contains("chip_guard#installed[0]"),
        "DuringActive should NOT contain 'chip_guard#installed[0]' (Shield condition false)"
    );
}

// ----------------------------------------------------------------
// Behavior 5: Installed During persists through condition cycling
//             (does not self-remove)
// ----------------------------------------------------------------

#[test]
fn installed_during_persists_through_condition_cycling() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![(
            "chip_siege#installed[0]".to_string(),
            Tree::During(
                Condition::NodeActive,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]))
        .id();

    // Fire
    evaluate_conditions(&mut world);

    // Toggle off
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Toggle back on
    world.insert_resource(State::new(NodeState::Playing));
    evaluate_conditions(&mut world);

    // BoundEffects should still contain the entry
    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should still exist");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should still contain the chip_siege#installed[0] entry"
    );
    assert_eq!(bound.0[0].0, "chip_siege#installed[0]");
    assert!(
        matches!(bound.0[0].1, Tree::During(..)),
        "Entry should still be a During variant"
    );

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Stack should exist after re-fire");
    assert_eq!(
        stack.len(),
        1,
        "Stack should have 1 entry after full true->false->true cycle"
    );
}
