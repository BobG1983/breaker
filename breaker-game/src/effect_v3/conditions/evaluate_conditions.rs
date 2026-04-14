//! `evaluate_conditions` — per-frame condition polling system for During nodes.

use std::collections::HashSet;

use bevy::prelude::*;

use super::{is_combo_active, is_node_active, is_shield_active};
use crate::effect_v3::{
    dispatch::{fire_reversible_dispatch, reverse_all_by_source_dispatch, reverse_dispatch},
    storage::BoundEffects,
    types::{Condition, ReversibleEffectType, ScopedTerminal, ScopedTree, Terminal, Tree},
};

/// Tracks which During sources have their effects currently applied
/// on this entity. Each entry in the `HashSet` is a source name string.
#[derive(Component, Default, Debug)]
pub struct DuringActive(pub HashSet<String>);

/// Poll all registered conditions each frame and fire/reverse During
/// entries on state transitions.
///
/// Runs in `EffectV3Systems::Conditions`.
pub fn evaluate_conditions(world: &mut World) {
    // Phase 1: Collect During entries (need immutable borrow first)
    let mut during_entries: Vec<(Entity, String, Condition, ScopedTree)> = Vec::new();
    let mut query = world.query::<(Entity, &BoundEffects)>();
    for (entity, bound) in query.iter(world) {
        for (source, tree) in &bound.0 {
            if let Tree::During(condition, inner) = tree {
                during_entries.push((entity, source.clone(), condition.clone(), (**inner).clone()));
            }
        }
    }

    // Phase 2: Evaluate conditions and manage transitions
    for (entity, source, condition, inner) in during_entries {
        if world.get_entity(entity).is_err() {
            continue;
        }

        let is_true = evaluate_condition(&condition, world);

        // Ensure DuringActive exists
        if world.get::<DuringActive>(entity).is_none() {
            world.entity_mut(entity).insert(DuringActive::default());
        }

        let was_active = world
            .get::<DuringActive>(entity)
            .is_some_and(|da| da.0.contains(&source));

        if !was_active && is_true {
            fire_scoped_tree(&inner, entity, &source, world);
            if let Some(mut da) = world.get_mut::<DuringActive>(entity) {
                da.0.insert(source);
            }
        } else if was_active && !is_true {
            reverse_scoped_tree(&inner, entity, &source, world);
            if let Some(mut da) = world.get_mut::<DuringActive>(entity) {
                da.0.remove(&source);
            }
        }
    }
}

/// Apply scoped tree effects (fire phase).
fn fire_scoped_tree(inner: &ScopedTree, entity: Entity, source: &str, world: &mut World) {
    match inner {
        ScopedTree::Fire(reversible) => {
            fire_reversible_dispatch(reversible, entity, source, world);
        }
        ScopedTree::Sequence(effects) => {
            for reversible in effects {
                fire_reversible_dispatch(reversible, entity, source, world);
            }
        }
        ScopedTree::When(trigger, inner_tree) => {
            let armed_key = format!("{source}#armed[0]");
            let armed_tree = Tree::When(trigger.clone(), inner_tree.clone());
            install_armed_entry(entity, armed_key, armed_tree, world);
        }
        ScopedTree::On(participant, scoped_terminal) => {
            let armed_key = format!("{source}#armed[0]");
            let terminal = Terminal::from(scoped_terminal.clone());
            let armed_tree = Tree::On(*participant, terminal);
            install_armed_entry(entity, armed_key, armed_tree, world);
        }
        ScopedTree::During(..) => {
            // Nested During inside During: handled by Shape A (wave 7b install pattern).
        }
    }
}

/// Reverse scoped tree effects (reversal phase).
fn reverse_scoped_tree(inner: &ScopedTree, entity: Entity, source: &str, world: &mut World) {
    match inner {
        ScopedTree::Fire(reversible) => {
            reverse_dispatch(reversible, entity, source, world);
        }
        ScopedTree::Sequence(effects) => {
            for reversible in effects {
                reverse_dispatch(reversible, entity, source, world);
            }
        }
        ScopedTree::When(_trigger, inner_tree) => {
            let armed_key = format!("{source}#armed[0]");
            // Remove armed entry from BoundEffects
            if let Some(mut bound) = world.get_mut::<BoundEffects>(entity) {
                bound.0.retain(|(name, _)| name != &armed_key);
            }
            // Reverse any effects the armed When may have fired
            reverse_armed_tree(inner_tree, entity, &armed_key, world);
        }
        ScopedTree::On(_participant, scoped_terminal) => {
            let armed_key = format!("{source}#armed[0]");
            // Remove armed entry from BoundEffects
            if let Some(mut bound) = world.get_mut::<BoundEffects>(entity) {
                bound.0.retain(|(name, _)| name != &armed_key);
            }
            // Reverse on OWNER entity per D18
            reverse_armed_scoped_terminal(scoped_terminal, entity, &armed_key, world);
        }
        ScopedTree::During(..) => {
            // Nested During: handled by Shape A reversal.
        }
    }
}

/// Helper to evaluate a single condition against world state.
///
/// Used by the During state machine to check condition transitions.
#[must_use]
pub fn evaluate_condition(condition: &Condition, world: &World) -> bool {
    match condition {
        Condition::NodeActive => is_node_active(world),
        Condition::ShieldActive => is_shield_active(world),
        Condition::ComboActive(threshold) => is_combo_active(world, *threshold),
    }
}

/// Install a tree into `BoundEffects` with idempotency guard.
fn install_armed_entry(entity: Entity, armed_key: String, tree: Tree, world: &mut World) {
    if let Some(mut bound) = world.get_mut::<BoundEffects>(entity) {
        if bound.0.iter().any(|(name, _)| name == &armed_key) {
            return; // Already installed — idempotent
        }
        bound.0.push((armed_key, tree));
    } else {
        world
            .entity_mut(entity)
            .insert(BoundEffects(vec![(armed_key, tree)]));
    }
}

/// Reverse all Fire effects in a `Tree` by armed source key.
///
/// Only handles `Fire` and `Sequence` — other node types are not expected
/// inside armed `ScopedTree::When` inner trees for current shapes.
fn reverse_armed_tree(tree: &Tree, entity: Entity, source: &str, world: &mut World) {
    match tree {
        Tree::Fire(et) => {
            if let Ok(reversible) = ReversibleEffectType::try_from(et.clone()) {
                reverse_all_by_source_dispatch(&reversible, entity, source, world);
            }
        }
        Tree::Sequence(terminals) => {
            for terminal in terminals {
                if let Terminal::Fire(et) = terminal
                    && let Ok(reversible) = ReversibleEffectType::try_from(et.clone())
                {
                    reverse_all_by_source_dispatch(&reversible, entity, source, world);
                }
            }
        }
        _ => {}
    }
}

/// Reverse effects from a `ScopedTerminal` by armed source key on the OWNER entity.
fn reverse_armed_scoped_terminal(
    st: &ScopedTerminal,
    entity: Entity,
    source: &str,
    world: &mut World,
) {
    match st {
        ScopedTerminal::Fire(reversible) => {
            reverse_all_by_source_dispatch(reversible, entity, source, world);
        }
        ScopedTerminal::Route(..) => {}
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::{
        effect_v3::{
            effects::{DamageBoostConfig, SpeedBoostConfig},
            stacking::EffectStack,
            storage::BoundEffects,
            types::{EffectType, ReversibleEffectType, ScopedTree, Tree, Trigger},
        },
        state::types::NodeState,
    };

    // ----------------------------------------------------------------
    // Helper: build a During tree with a single reversible Fire effect
    // ----------------------------------------------------------------

    fn during_node_speed_boost() -> (String, Tree) {
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

    fn during_shield_damage_boost() -> (String, Tree) {
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

    // ================================================================
    // Wave 7c — Shapes C & D: Armed Scoped Triggers
    // ================================================================

    use bevy::ecs::world::CommandQueue;

    use crate::effect_v3::{
        types::{
            BumpTarget, EntityKind, ImpactTarget, ParticipantTarget, ScopedTerminal, Terminal,
            TriggerContext,
        },
        walking::walk_effects,
    };

    // ----------------------------------------------------------------
    // Helper: build a Shape C During tree (When gate inside During)
    // ----------------------------------------------------------------

    fn during_when_bumped_speed_boost() -> (String, Tree) {
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

    fn during_on_bump_bolt_speed_boost() -> (String, Tree) {
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

    fn walk_entity_effects(
        world: &mut World,
        entity: Entity,
        trigger: &Trigger,
        context: &TriggerContext,
    ) {
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, world);
            walk_effects(entity, trigger, context, &trees, &mut commands);
        }
        queue.apply(world);
    }

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

    // ================================================================
    // Shape D: During(Cond, On(Participant, Fire(reversible)))
    // ================================================================

    // ----------------------------------------------------------------
    // Behavior 13: Cond entering true installs a Tree::On armed entry
    //              into BoundEffects with #armed[0] key
    // ----------------------------------------------------------------

    #[test]
    fn shape_d_cond_entering_true_installs_armed_on_entry() {
        let mut world = World::new();
        world.insert_resource(State::new(NodeState::Playing));

        let entity = world
            .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
            .id();

        evaluate_conditions(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should exist");
        assert_eq!(
            bound.0.len(),
            2,
            "BoundEffects should contain 2 entries: original During + armed On"
        );

        // Original During entry preserved
        assert_eq!(bound.0[0].0, "chip_redirect");
        assert!(
            matches!(bound.0[0].1, Tree::During(..)),
            "First entry should still be the original During tree"
        );

        // Armed On entry installed
        let armed = bound
            .0
            .iter()
            .find(|(name, _)| name == "chip_redirect#armed[0]");
        assert!(
            armed.is_some(),
            "Should find armed On with key 'chip_redirect#armed[0]'"
        );
        let (_, armed_tree) = armed.unwrap();
        // Verify it is Tree::On with widened Terminal::Fire(EffectType::...), not ScopedTerminal
        assert!(
            matches!(
                armed_tree,
                Tree::On(
                    ParticipantTarget::Bump(BumpTarget::Bolt),
                    Terminal::Fire(EffectType::SpeedBoost(..))
                )
            ),
            "Armed entry should be Tree::On(Bump(Bolt), Terminal::Fire(EffectType::SpeedBoost(...)))"
        );

        let da = world
            .get::<DuringActive>(entity)
            .expect("DuringActive should exist");
        assert!(
            da.0.contains("chip_redirect"),
            "DuringActive should contain 'chip_redirect'"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 14: Cond staying true does not re-install the armed On entry
    // ----------------------------------------------------------------

    #[test]
    fn shape_d_cond_staying_true_does_not_reinstall_armed_on_entry() {
        let mut world = World::new();
        world.insert_resource(State::new(NodeState::Playing));

        let entity = world
            .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
            .id();

        evaluate_conditions(&mut world);
        evaluate_conditions(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should exist");
        assert_eq!(
            bound.0.len(),
            2,
            "BoundEffects should contain exactly 2 entries after two evaluations"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 15: Firing trigger with matching participant context
    //              redirects effect to participant entity
    // ----------------------------------------------------------------

    #[test]
    fn shape_d_trigger_with_matching_context_redirects_to_participant() {
        let mut world = World::new();
        world.insert_resource(State::new(NodeState::Playing));

        let entity_a = world
            .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
            .id();
        let entity_b = world.spawn_empty().id();

        // Arm
        evaluate_conditions(&mut world);

        // Fire with matching context
        let context = TriggerContext::Bump {
            bolt:    Some(entity_b),
            breaker: entity_a,
        };
        walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);

        // Effect should be on entity_b (the bolt), not entity_a (the owner)
        let bolt_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity_b)
            .expect("EffectStack should exist on bolt entity (entity_b)");
        assert_eq!(bolt_stack.len(), 1);

        let entry = bolt_stack.iter().next().unwrap();
        assert_eq!(
            entry.0, "chip_redirect#armed[0]",
            "Source on bolt's stack must be the armed key"
        );

        assert!(
            world
                .get::<EffectStack<SpeedBoostConfig>>(entity_a)
                .is_none(),
            "Owner (entity_a) should have no EffectStack — effect goes to participant"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 16: Participant filter correctness — no bolt in context
    // ----------------------------------------------------------------

    #[test]
    fn shape_d_no_bolt_in_context_does_not_fire() {
        let mut world = World::new();
        world.insert_resource(State::new(NodeState::Playing));

        let entity_a = world
            .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
            .id();

        // Arm
        evaluate_conditions(&mut world);

        // Precondition: armed entry must be installed
        let bound = world.get::<BoundEffects>(entity_a).unwrap();
        assert_eq!(
            bound.0.len(),
            2,
            "Precondition: armed On entry must be installed before testing participant filter"
        );

        // Context with bolt = None
        let context = TriggerContext::Bump {
            bolt:    None,
            breaker: entity_a,
        };
        walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);

        assert!(
            world
                .get::<EffectStack<SpeedBoostConfig>>(entity_a)
                .is_none(),
            "No EffectStack should exist when bolt is None in context"
        );
    }

    // Edge case: TriggerContext::None also produces no stack entries
    #[test]
    fn shape_d_context_none_does_not_fire() {
        let mut world = World::new();
        world.insert_resource(State::new(NodeState::Playing));

        let entity_a = world
            .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
            .id();

        evaluate_conditions(&mut world);

        // Precondition: armed entry must be installed
        let bound = world.get::<BoundEffects>(entity_a).unwrap();
        assert_eq!(
            bound.0.len(),
            2,
            "Precondition: armed On entry must be installed before testing TriggerContext::None"
        );

        walk_entity_effects(
            &mut world,
            entity_a,
            &Trigger::Bumped,
            &TriggerContext::None,
        );

        assert!(
            world
                .get::<EffectStack<SpeedBoostConfig>>(entity_a)
                .is_none(),
            "No EffectStack should exist when context is None"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 17: Participant filter — Impact context does not match
    //              Bump(Bolt) target
    // ----------------------------------------------------------------

    #[test]
    fn shape_d_impact_context_does_not_match_bump_bolt_target() {
        let mut world = World::new();
        world.insert_resource(State::new(NodeState::Playing));

        let entity_a = world
            .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
            .id();
        let entity_b = world.spawn_empty().id();
        let entity_c = world.spawn_empty().id();

        // Arm
        evaluate_conditions(&mut world);

        // Precondition: armed entry must be installed
        let bound = world.get::<BoundEffects>(entity_a).unwrap();
        assert_eq!(
            bound.0.len(),
            2,
            "Precondition: armed On entry must be installed before testing mismatched context"
        );

        // Fire with Impact context (wrong context type for Bump(Bolt))
        let context = TriggerContext::Impact {
            impactor: entity_b,
            impactee: entity_c,
        };
        walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);

        assert!(
            world
                .get::<EffectStack<SpeedBoostConfig>>(entity_a)
                .is_none(),
            "No EffectStack on owner"
        );
        assert!(
            world
                .get::<EffectStack<SpeedBoostConfig>>(entity_b)
                .is_none(),
            "No EffectStack on impactor"
        );
        assert!(
            world
                .get::<EffectStack<SpeedBoostConfig>>(entity_c)
                .is_none(),
            "No EffectStack on impactee"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 18: Firing multiple times with matching context stacks
    //              effects on participant entity
    // ----------------------------------------------------------------

    #[test]
    fn shape_d_multiple_fires_stack_on_participant() {
        let mut world = World::new();
        world.insert_resource(State::new(NodeState::Playing));

        let entity_a = world
            .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
            .id();
        let entity_b = world.spawn_empty().id();

        // Arm
        evaluate_conditions(&mut world);

        let context = TriggerContext::Bump {
            bolt:    Some(entity_b),
            breaker: entity_a,
        };

        // Fire 3 times
        for _ in 0..3 {
            walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);
        }

        let bolt_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity_b)
            .expect("EffectStack should exist on bolt entity");
        assert_eq!(
            bolt_stack.len(),
            3,
            "Bolt entity should have 3 stack entries from 3 fires"
        );

        for entry in bolt_stack.iter() {
            assert_eq!(entry.0, "chip_redirect#armed[0]");
        }
    }

    // ----------------------------------------------------------------
    // Behavior 19: Cond exiting true removes armed On entry AND reverses
    //              stacked effects on the OWNER entity only (D18)
    // ----------------------------------------------------------------

    #[test]
    fn shape_d_disarm_removes_armed_on_and_reverses_stack_on_owner() {
        let mut world = World::new();
        world.insert_resource(State::new(NodeState::Playing));

        let entity_a = world
            .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
            .id();

        // Arm
        evaluate_conditions(&mut world);

        // Use degenerate context where bolt = owner (entity_a) so reversal is observable
        let context = TriggerContext::Bump {
            bolt:    Some(entity_a),
            breaker: entity_a,
        };

        // Fire 2x — effects go to entity_a (since bolt = entity_a)
        for _ in 0..2 {
            walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);
        }

        // Verify precondition: entity_a has 2 stack entries
        assert_eq!(
            world
                .get::<EffectStack<SpeedBoostConfig>>(entity_a)
                .unwrap()
                .len(),
            2,
            "Precondition: owner should have 2 stack entries"
        );

        // Disarm
        world.insert_resource(State::new(NodeState::Loading));
        evaluate_conditions(&mut world);

        // Armed entry removed
        let bound = world.get::<BoundEffects>(entity_a).unwrap();
        assert_eq!(
            bound.0.len(),
            1,
            "BoundEffects should have 1 entry (original During only)"
        );
        assert_eq!(bound.0[0].0, "chip_redirect");

        // Stack reversed on owner
        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .expect("Stack should still exist");
        assert!(
            stack.is_empty(),
            "Stack on owner should be empty after reversal"
        );

        // DuringActive cleared
        let da = world.get::<DuringActive>(entity_a).unwrap();
        assert!(
            !da.0.contains("chip_redirect"),
            "DuringActive should not contain 'chip_redirect'"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 20: Firing the trigger after disarm is a no-op
    // ----------------------------------------------------------------

    #[test]
    fn shape_d_trigger_after_disarm_is_noop() {
        let mut world = World::new();
        world.insert_resource(State::new(NodeState::Playing));

        let entity_a = world
            .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
            .id();

        // Arm
        evaluate_conditions(&mut world);

        // Fire once with degenerate context
        let context = TriggerContext::Bump {
            bolt:    Some(entity_a),
            breaker: entity_a,
        };
        walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);

        // Disarm
        world.insert_resource(State::new(NodeState::Loading));
        evaluate_conditions(&mut world);

        // Fire again after disarm
        walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .expect("Stack should still exist");
        assert!(
            stack.is_empty(),
            "Stack should remain empty after trigger fires post-disarm"
        );

        let bound = world.get::<BoundEffects>(entity_a).unwrap();
        assert_eq!(
            bound.0.len(),
            1,
            "BoundEffects should have only 1 entry after disarm"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 21: Re-entering Cond true re-arms with a fresh On entry
    // ----------------------------------------------------------------

    #[test]
    fn shape_d_re_entering_true_rearms_with_fresh_on_entry() {
        let mut world = World::new();
        world.insert_resource(State::new(NodeState::Playing));

        let entity_a = world
            .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
            .id();

        // Arm
        evaluate_conditions(&mut world);

        // Fire 2x with degenerate context
        let context = TriggerContext::Bump {
            bolt:    Some(entity_a),
            breaker: entity_a,
        };
        for _ in 0..2 {
            walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);
        }

        // Disarm
        world.insert_resource(State::new(NodeState::Loading));
        evaluate_conditions(&mut world);

        // Re-arm
        world.insert_resource(State::new(NodeState::Playing));
        evaluate_conditions(&mut world);

        let bound = world.get::<BoundEffects>(entity_a).unwrap();
        assert_eq!(
            bound.0.len(),
            2,
            "BoundEffects should have 2 entries (original During + fresh armed On)"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 22: Shape D On(Impact(Impactee)) with Impact context
    //              redirects to impactee
    // ----------------------------------------------------------------

    #[test]
    fn shape_d_on_impact_impactee_redirects_to_impactee() {
        let mut world = World::new();
        world.insert_resource(State::new(NodeState::Playing));

        let entity_owner = world
            .spawn(BoundEffects(vec![(
                "chip_reflect".to_string(),
                Tree::During(
                    Condition::NodeActive,
                    Box::new(ScopedTree::On(
                        ParticipantTarget::Impact(ImpactTarget::Impactee),
                        ScopedTerminal::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                            multiplier: OrderedFloat(2.0),
                        })),
                    )),
                ),
            )]))
            .id();
        let entity_cell = world.spawn_empty().id();

        // Arm
        evaluate_conditions(&mut world);

        // Fire with Impact context
        let context = TriggerContext::Impact {
            impactor: entity_owner,
            impactee: entity_cell,
        };
        walk_entity_effects(
            &mut world,
            entity_owner,
            &Trigger::Impacted(EntityKind::Cell),
            &context,
        );

        let cell_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity_cell)
            .expect("EffectStack should exist on impactee (cell) entity");
        assert_eq!(cell_stack.len(), 1);

        let entry = cell_stack.iter().next().unwrap();
        assert_eq!(
            entry.0, "chip_reflect#armed[0]",
            "Source on impactee's stack must be the armed key"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 23: Shape D armed entry with Bump(Breaker) redirects
    //              to breaker entity
    // ----------------------------------------------------------------

    #[test]
    fn shape_d_on_bump_breaker_redirects_to_breaker_entity() {
        let mut world = World::new();
        world.insert_resource(State::new(NodeState::Playing));

        let entity_a = world
            .spawn(BoundEffects(vec![(
                "chip_breaker_buff".to_string(),
                Tree::During(
                    Condition::NodeActive,
                    Box::new(ScopedTree::On(
                        ParticipantTarget::Bump(BumpTarget::Breaker),
                        ScopedTerminal::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        })),
                    )),
                ),
            )]))
            .id();
        let entity_bolt = world.spawn_empty().id();
        let entity_breaker = world.spawn_empty().id();

        // Arm
        evaluate_conditions(&mut world);

        // Fire with Bump context
        let context = TriggerContext::Bump {
            bolt:    Some(entity_bolt),
            breaker: entity_breaker,
        };
        walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);

        let breaker_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity_breaker)
            .expect("EffectStack should exist on breaker entity");
        assert_eq!(breaker_stack.len(), 1);

        assert!(
            world
                .get::<EffectStack<SpeedBoostConfig>>(entity_bolt)
                .is_none(),
            "Bolt entity should have no EffectStack"
        );
        assert!(
            world
                .get::<EffectStack<SpeedBoostConfig>>(entity_a)
                .is_none(),
            "Owner entity should have no EffectStack"
        );
    }

    // ================================================================
    // Behavior 27: ScopedTerminal::Fire(reversible) converts to
    //              Terminal::Fire(EffectType) for armed On entry
    // ================================================================

    #[test]
    fn scoped_terminal_fire_converts_to_terminal_fire_with_widened_type() {
        let scoped = ScopedTerminal::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let terminal = Terminal::from(scoped);
        assert_eq!(
            terminal,
            Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            })),
            "ScopedTerminal::Fire should convert to Terminal::Fire with widened EffectType"
        );
    }
}
