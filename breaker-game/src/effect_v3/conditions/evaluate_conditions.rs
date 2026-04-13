//! `evaluate_conditions` — per-frame condition polling system for During nodes.

use std::collections::HashSet;

use bevy::prelude::*;

use super::{is_combo_active, is_node_active, is_shield_active};
use crate::effect_v3::{
    dispatch::{fire_reversible_dispatch, reverse_dispatch},
    storage::BoundEffects,
    types::{Condition, ScopedTree, Tree},
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
        ScopedTree::When(..) | ScopedTree::On(..) => {
            // Nested When/On inside During: conditional/redirected behavior that
            // fires during future walks, not during initial application.
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
        ScopedTree::When(..) | ScopedTree::On(..) => {
            // Nested When/On inside During: no explicit reversal needed.
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

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::{
        effect_v3::{
            effects::{DamageBoostConfig, SpeedBoostConfig},
            stacking::EffectStack,
            storage::BoundEffects,
            types::{EffectType, ReversibleEffectType, ScopedTree, Tree},
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
}
