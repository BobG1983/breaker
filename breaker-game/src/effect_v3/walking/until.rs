//! Until node evaluator — event-scoped effect application.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::effect_v3::{
    dispatch::{fire_reversible_dispatch, reverse_dispatch},
    storage::BoundEffects,
    types::{ScopedTree, Trigger, TriggerContext},
};

/// Tracks which Until sources have already applied their inner effects
/// on this entity. Each entry in the `HashSet` is a source name string.
#[derive(Component, Default, Debug)]
pub struct UntilApplied(pub HashSet<String>);

/// Evaluate a `Tree::Until` node: apply inner effects immediately,
/// reverse them when the trigger fires.
pub fn evaluate_until(
    entity: Entity,
    gate_trigger: &Trigger,
    inner: &ScopedTree,
    active_trigger: &Trigger,
    context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    let _ = context;
    commands.queue(UntilEvaluateCommand {
        entity,
        gate_trigger: gate_trigger.clone(),
        active_trigger: active_trigger.clone(),
        inner: inner.clone(),
        source: source.to_owned(),
    });
}

/// Deferred command that performs the Until state-machine logic with world access.
struct UntilEvaluateCommand {
    entity:         Entity,
    gate_trigger:   Trigger,
    active_trigger: Trigger,
    inner:          ScopedTree,
    source:         String,
}

impl Command for UntilEvaluateCommand {
    fn apply(self, world: &mut World) {
        // Guard: entity must still exist
        if world.get_entity(self.entity).is_err() {
            return;
        }

        // Ensure UntilApplied component exists on entity
        if world.get::<UntilApplied>(self.entity).is_none() {
            world
                .entity_mut(self.entity)
                .insert(UntilApplied::default());
        }

        // Check if this source has already been applied
        let is_applied = world
            .get::<UntilApplied>(self.entity)
            .is_some_and(|ua| ua.0.contains(&self.source));

        if !is_applied {
            // NOT YET APPLIED: fire inner effects, mark as applied
            fire_scoped_tree(&self.inner, self.entity, &self.source, world);
            if let Some(mut ua) = world.get_mut::<UntilApplied>(self.entity) {
                ua.0.insert(self.source.clone());
            }

            // If gate matches on first walk, immediately reverse and clean up
            if self.gate_trigger == self.active_trigger {
                reverse_scoped_tree(&self.inner, self.entity, &self.source, world);
                if let Some(mut ua) = world.get_mut::<UntilApplied>(self.entity) {
                    ua.0.remove(&self.source);
                }
                if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
                    bound.0.retain(|(name, _)| name != &self.source);
                }
            }
        } else if self.gate_trigger == self.active_trigger {
            // APPLIED AND gate trigger matches: reverse and clean up
            reverse_scoped_tree(&self.inner, self.entity, &self.source, world);
            if let Some(mut ua) = world.get_mut::<UntilApplied>(self.entity) {
                ua.0.remove(&self.source);
            }
            if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
                bound.0.retain(|(name, _)| name != &self.source);
            }
        }
        // else: APPLIED but gate doesn't match — no-op
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
            // Nested When/On inside Until: conditional/redirected behavior that
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
            // Nested When/On inside Until: no explicit reversal needed.
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::CommandQueue;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        types::{EffectType, ReversibleEffectType, Tree},
        walking::walk_effects::walk_effects,
    };

    // ----------------------------------------------------------------
    // Behavior 1: Until fires inner ScopedTree::Fire effect on first
    //             evaluation (any trigger)
    // ----------------------------------------------------------------

    #[test]
    fn until_fires_inner_fire_effect_on_first_walk_with_non_matching_trigger() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist after Until fires on first walk");
        assert_eq!(
            stack.len(),
            1,
            "Until should fire inner effect on first walk regardless of trigger"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 2: Until does not fire inner effect a second time on
    //             subsequent non-matching walks
    // ----------------------------------------------------------------

    #[test]
    fn until_does_not_fire_inner_effect_on_subsequent_non_matching_walk() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);

        // First walk: fires inner effect
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
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
            walk_effects(
                entity,
                &Trigger::BoltLostOccurred,
                &TriggerContext::None,
                &trees_second,
                &mut commands,
            );
        }
        queue2.apply(&mut world);

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should still exist");
        assert_eq!(
            stack.len(),
            1,
            "Until should NOT fire inner effect a second time on subsequent non-matching walk"
        );
    }

    // Edge case for behavior 2: 5 non-matching walks still yields exactly 1 stack entry
    #[test]
    fn until_does_not_fire_on_five_subsequent_non_matching_walks() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);

        let non_matching_triggers = [
            Trigger::NodeStartOccurred,
            Trigger::BoltLostOccurred,
            Trigger::NodeEndOccurred,
            Trigger::BumpOccurred,
            Trigger::PerfectBumpOccurred,
        ];

        for trigger in &non_matching_triggers {
            let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
            let mut queue = CommandQueue::default();
            {
                let mut commands = Commands::new(&mut queue, &world);
                walk_effects(
                    entity,
                    trigger,
                    &TriggerContext::None,
                    &trees,
                    &mut commands,
                );
            }
            queue.apply(&mut world);
        }

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist after 5 walks");
        assert_eq!(
            stack.len(),
            1,
            "After 5 non-matching walks, stack should still have exactly 1 entry"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 3: Until reverses inner effect when gate trigger fires
    //             (after prior application)
    // ----------------------------------------------------------------

    #[test]
    fn until_reverses_inner_effect_when_gate_trigger_matches_after_application() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);

        // First walk: apply
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        // Second walk: gate trigger matches — should reverse
        let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue2 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue2, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees_second,
                &mut commands,
            );
        }
        queue2.apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
        let is_empty = stack.is_none() || stack.unwrap().is_empty();
        assert!(
            is_empty,
            "EffectStack should be empty after Until reverses on gate trigger match"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 4: Until self-removes from BoundEffects after reversal
    // ----------------------------------------------------------------

    #[test]
    fn until_removes_entry_from_bound_effects_after_reversal() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);

        // First walk: apply
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        // Second walk: gate trigger matches — should reverse and remove
        let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue2 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue2, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees_second,
                &mut commands,
            );
        }
        queue2.apply(&mut world);

        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert!(
            !remaining.iter().any(|(name, _)| name == "chip_a"),
            "chip_a should be removed from BoundEffects after Until reversal"
        );
    }

    // Edge case for behavior 4: BoundEffects with only Until entry becomes empty vec
    #[test]
    fn until_bound_effects_becomes_empty_after_sole_entry_reversal() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);

        // Apply
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        // Reverse
        let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue2 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue2, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees_second,
                &mut commands,
            );
        }
        queue2.apply(&mut world);

        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert!(
            remaining.is_empty(),
            "BoundEffects should be empty vec after sole Until entry is reversed and removed"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 5: Until with ScopedTree::Sequence fires all reversible
    //             effects on first walk
    // ----------------------------------------------------------------

    #[test]
    fn until_with_sequence_fires_all_reversible_effects_on_first_walk() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Sequence(vec![
                    ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }),
                    ReversibleEffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    }),
                ])),
            ),
        )]);
        world.entity_mut(entity).insert(bound);
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let speed_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("SpeedBoost EffectStack should exist");
        assert_eq!(speed_stack.len(), 1);

        let dmg_stack = world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .expect("DamageBoost EffectStack should exist");
        assert_eq!(dmg_stack.len(), 1);
    }

    // ----------------------------------------------------------------
    // Behavior 6: Until with ScopedTree::Sequence reverses ALL effects
    //             on gate trigger match
    // ----------------------------------------------------------------

    #[test]
    fn until_with_sequence_reverses_all_effects_on_gate_trigger_match() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Sequence(vec![
                    ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }),
                    ReversibleEffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    }),
                ])),
            ),
        )]);
        world.entity_mut(entity).insert(bound);

        // First walk: apply all
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        // Second walk: gate trigger — reverse all
        let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue2 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue2, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees_second,
                &mut commands,
            );
        }
        queue2.apply(&mut world);

        let speed_empty = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .is_none_or(EffectStack::is_empty);
        let dmg_empty = world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .is_none_or(EffectStack::is_empty);
        assert!(
            speed_empty,
            "SpeedBoost EffectStack should be empty after Sequence reversal"
        );
        assert!(
            dmg_empty,
            "DamageBoost EffectStack should be empty after Sequence reversal"
        );

        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert!(
            !remaining.iter().any(|(name, _)| name == "chip_a"),
            "chip_a should be removed from BoundEffects after Sequence reversal"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 7: Until does not reverse on non-matching trigger after
    //             application
    // ----------------------------------------------------------------

    #[test]
    fn until_does_not_reverse_on_non_matching_trigger_after_application() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);

        // First walk: apply
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        // Second walk: non-matching trigger — should not reverse
        let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue2 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue2, &world);
            walk_effects(
                entity,
                &Trigger::BoltLostOccurred,
                &TriggerContext::None,
                &trees_second,
                &mut commands,
            );
        }
        queue2.apply(&mut world);

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should still exist after non-matching walk");
        assert_eq!(
            stack.len(),
            1,
            "Effect should still be active after non-matching trigger"
        );

        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert!(
            remaining.iter().any(|(name, _)| name == "chip_a"),
            "chip_a should still be in BoundEffects after non-matching trigger"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 8: Until produces no additional effects or reversals
    //             after self-removal
    // ----------------------------------------------------------------

    #[test]
    fn until_produces_no_effects_after_self_removal() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);

        // First walk: apply
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        // Verify the first walk actually fired the effect (precondition)
        let stack_after_fire = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist after first walk fires Until inner effect");
        assert_eq!(
            stack_after_fire.len(),
            1,
            "Precondition: Until must fire inner effect on first walk"
        );

        // Second walk: gate trigger — reverse and remove
        let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue2 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue2, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees_second,
                &mut commands,
            );
        }
        queue2.apply(&mut world);

        // Third walk: walk with gate trigger again — should be a no-op
        let trees_third = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue3 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue3, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees_third,
                &mut commands,
            );
        }
        queue3.apply(&mut world);

        // Fourth walk: non-matching trigger — also no-op
        let trees_fourth = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue4 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue4, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees_fourth,
                &mut commands,
            );
        }
        queue4.apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
        let is_empty = stack.is_none() || stack.unwrap().is_empty();
        assert!(
            is_empty,
            "No new EffectStack entries should appear after Until self-removes"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 9: Until fires and immediately reverses when first walk
    //             matches gate trigger
    // ----------------------------------------------------------------

    #[test]
    fn until_fires_and_immediately_reverses_when_first_walk_matches_gate() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        // Effect should have been fired then immediately reversed
        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
        let is_empty = stack.is_none() || stack.unwrap().is_empty();
        assert!(
            is_empty,
            "EffectStack should be empty — Until fires then immediately reverses when first walk matches gate"
        );

        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert!(
            !remaining.iter().any(|(name, _)| name == "chip_a"),
            "chip_a should be removed from BoundEffects after immediate fire-and-reverse"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 10: Until removal does not affect other entries in
    //              BoundEffects
    // ----------------------------------------------------------------

    #[test]
    fn until_removal_does_not_affect_other_bound_effects_entries() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![
            (
                "chip_a".to_string(),
                Tree::Until(
                    Trigger::Bumped,
                    Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                        SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        },
                    ))),
                ),
            ),
            (
                "chip_b".to_string(),
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    }))),
                ),
            ),
        ]);
        world.entity_mut(entity).insert(bound);

        // First walk: NodeStartOccurred — Until fires, When does not match
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        // Second walk: Bumped — Until reverses, When fires
        let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue2 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue2, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees_second,
                &mut commands,
            );
        }
        queue2.apply(&mut world);

        // SpeedBoost should be reversed (empty)
        let speed_empty = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .is_none_or(EffectStack::is_empty);
        assert!(
            speed_empty,
            "SpeedBoost should be reversed after Until gate trigger match"
        );

        // DamageBoost should have 1 entry from When firing
        let dmg_stack = world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .expect("DamageBoost EffectStack should exist from When firing");
        assert_eq!(
            dmg_stack.len(),
            1,
            "When tree should have fired DamageBoost"
        );

        // BoundEffects should only contain chip_b
        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert!(
            !remaining.iter().any(|(name, _)| name == "chip_a"),
            "chip_a (Until) should be removed from BoundEffects"
        );
        assert!(
            remaining.iter().any(|(name, _)| name == "chip_b"),
            "chip_b (When) should remain in BoundEffects"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 11: Multiple Until entries with different sources track
    //              independently
    // ----------------------------------------------------------------

    #[test]
    fn multiple_until_entries_track_independently() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![
            (
                "chip_a".to_string(),
                Tree::Until(
                    Trigger::Bumped,
                    Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                        SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        },
                    ))),
                ),
            ),
            (
                "chip_b".to_string(),
                Tree::Until(
                    Trigger::BoltLostOccurred,
                    Box::new(ScopedTree::Fire(ReversibleEffectType::DamageBoost(
                        DamageBoostConfig {
                            multiplier: OrderedFloat(2.0),
                        },
                    ))),
                ),
            ),
        ]);
        world.entity_mut(entity).insert(bound);

        // First walk: NodeStartOccurred — both fire
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let speed_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("SpeedBoost should exist after both fire");
        assert_eq!(speed_stack.len(), 1);
        let dmg_stack = world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .expect("DamageBoost should exist after both fire");
        assert_eq!(dmg_stack.len(), 1);

        // Second walk: Bumped — only chip_a reverses
        let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue2 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue2, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees_second,
                &mut commands,
            );
        }
        queue2.apply(&mut world);

        let speed_empty = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .is_none_or(EffectStack::is_empty);
        assert!(
            speed_empty,
            "chip_a SpeedBoost should be reversed after Bumped trigger"
        );

        let dmg_stack_after = world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .expect("chip_b DamageBoost should still be active");
        assert_eq!(
            dmg_stack_after.len(),
            1,
            "chip_b DamageBoost should still have 1 entry (BoltLostOccurred is its gate)"
        );

        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert!(
            !remaining.iter().any(|(name, _)| name == "chip_a"),
            "chip_a should be removed from BoundEffects"
        );
        assert!(
            remaining.iter().any(|(name, _)| name == "chip_b"),
            "chip_b should still be in BoundEffects"
        );
    }

    // Edge case for behavior 11: second Until also reverses on its gate trigger
    #[test]
    fn multiple_until_entries_both_reverse_on_respective_gate_triggers() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![
            (
                "chip_a".to_string(),
                Tree::Until(
                    Trigger::Bumped,
                    Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                        SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        },
                    ))),
                ),
            ),
            (
                "chip_b".to_string(),
                Tree::Until(
                    Trigger::BoltLostOccurred,
                    Box::new(ScopedTree::Fire(ReversibleEffectType::DamageBoost(
                        DamageBoostConfig {
                            multiplier: OrderedFloat(2.0),
                        },
                    ))),
                ),
            ),
        ]);
        world.entity_mut(entity).insert(bound);

        // Walk 1: both fire
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        // Walk 2: chip_a reverses
        let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue2 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue2, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees_second,
                &mut commands,
            );
        }
        queue2.apply(&mut world);

        // Walk 3: chip_b reverses
        let trees_third = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue3 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue3, &world);
            walk_effects(
                entity,
                &Trigger::BoltLostOccurred,
                &TriggerContext::None,
                &trees_third,
                &mut commands,
            );
        }
        queue3.apply(&mut world);

        let speed_empty = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .is_none_or(EffectStack::is_empty);
        let dmg_empty = world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .is_none_or(EffectStack::is_empty);
        assert!(speed_empty, "SpeedBoost should be reversed");
        assert!(dmg_empty, "DamageBoost should be reversed");

        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert!(
            remaining.is_empty(),
            "BoundEffects should be empty after both Until entries reverse"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 12: UntilApplied component is created on first Until
    //              evaluation if not present
    // ----------------------------------------------------------------

    #[test]
    fn until_applied_component_created_on_first_evaluation() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);

        assert!(
            world.get::<UntilApplied>(entity).is_none(),
            "UntilApplied should not exist before first walk"
        );

        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let until_applied = world
            .get::<UntilApplied>(entity)
            .expect("UntilApplied component should be inserted after first evaluation");
        assert!(
            until_applied.0.contains("chip_a"),
            "UntilApplied should contain 'chip_a' after first evaluation"
        );
    }

    // Edge case for behavior 12: entity already has UntilApplied from different source
    #[test]
    fn until_applied_adds_to_existing_set_from_different_source() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Pre-insert UntilApplied with a different source
        let mut existing = UntilApplied::default();
        existing.0.insert("chip_x".to_string());
        world.entity_mut(entity).insert(existing);

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let until_applied = world
            .get::<UntilApplied>(entity)
            .expect("UntilApplied should exist");
        assert!(
            until_applied.0.contains("chip_a"),
            "chip_a should be added to existing UntilApplied"
        );
        assert!(
            until_applied.0.contains("chip_x"),
            "chip_x should remain in UntilApplied"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 13: UntilApplied source entry is removed after reversal
    // ----------------------------------------------------------------

    #[test]
    fn until_applied_source_removed_after_reversal() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);

        // Apply
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::NodeStartOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        // Verify it's in UntilApplied
        assert!(
            world
                .get::<UntilApplied>(entity)
                .expect("UntilApplied should exist")
                .0
                .contains("chip_a"),
            "chip_a should be in UntilApplied after first walk"
        );

        // Reverse
        let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue2 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue2, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees_second,
                &mut commands,
            );
        }
        queue2.apply(&mut world);

        let until_applied = world
            .get::<UntilApplied>(entity)
            .expect("UntilApplied component should still exist (may be empty)");
        assert!(
            !until_applied.0.contains("chip_a"),
            "chip_a should be removed from UntilApplied after reversal"
        );
    }
}
