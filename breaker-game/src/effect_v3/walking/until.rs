//! Until node evaluator — event-scoped effect application.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::effect_v3::{
    conditions::DuringActive,
    dispatch::{fire_reversible_dispatch, reverse_all_by_source_dispatch, reverse_dispatch},
    storage::BoundEffects,
    types::{ScopedTree, Tree, Trigger, TriggerContext},
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

        match &self.inner {
            ScopedTree::During(condition, inner_scoped) => {
                // Shape B: Until wraps a During
                if !is_applied {
                    // Install the During into BoundEffects for the condition poller
                    let install_key = format!("{}#installed[0]", self.source);
                    if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity)
                        && !bound.0.iter().any(|(name, _)| name == &install_key)
                    {
                        bound.0.push((
                            install_key,
                            Tree::During(condition.clone(), Box::new((**inner_scoped).clone())),
                        ));
                    }
                    // Mark as applied
                    if let Some(mut ua) = world.get_mut::<UntilApplied>(self.entity) {
                        ua.0.insert(self.source.clone());
                    }

                    // If gate matches on first walk, immediately tear down
                    if self.gate_trigger == self.active_trigger {
                        teardown_installed_during(self.entity, &self.source, inner_scoped, world);
                        // Clean up Until tracking
                        if let Some(mut ua) = world.get_mut::<UntilApplied>(self.entity) {
                            ua.0.remove(&self.source);
                        }
                        if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
                            bound.0.retain(|(name, _)| name != &self.source);
                        }
                    }
                } else if self.gate_trigger == self.active_trigger {
                    // Applied and gate fires: tear down
                    teardown_installed_during(self.entity, &self.source, inner_scoped, world);
                    if let Some(mut ua) = world.get_mut::<UntilApplied>(self.entity) {
                        ua.0.remove(&self.source);
                    }
                    if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
                        bound.0.retain(|(name, _)| name != &self.source);
                    }
                }
                // else: APPLIED but gate doesn't match — no-op (During polls normally)
            }
            _ => {
                // Existing logic: fire/reverse for Fire/Sequence/When/On
                if !is_applied {
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
        ScopedTree::When(..) | ScopedTree::On(..) | ScopedTree::During(..) => {
            // Nested When/On/During inside Until: conditional/redirected behavior
            // that fires during future walks, not during initial application.
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
        ScopedTree::When(..) | ScopedTree::On(..) | ScopedTree::During(..) => {
            // Nested When/On/During inside Until: no explicit reversal needed.
        }
    }
}

/// Tear down an installed During from `BoundEffects` and reverse any
/// active effects it fired. Used by Shape B (Until wrapping During).
fn teardown_installed_during(
    entity: Entity,
    source: &str,
    inner_scoped: &ScopedTree,
    world: &mut World,
) {
    let install_key = format!("{source}#installed[0]");

    // Remove installed During from BoundEffects
    if let Some(mut bound) = world.get_mut::<BoundEffects>(entity) {
        bound.0.retain(|(name, _)| name != &install_key);
    }

    // Check if the During was active and reverse its effects
    let was_active = world
        .get::<DuringActive>(entity)
        .is_some_and(|da| da.0.contains(&install_key));

    if was_active {
        reverse_scoped_tree_by_source(inner_scoped, entity, &install_key, world);
        if let Some(mut da) = world.get_mut::<DuringActive>(entity) {
            da.0.remove(&install_key);
        }
    }
}

/// Reverse scoped tree effects using `reverse_all_by_source_dispatch`,
/// which removes all instances fired from the given source. Used during
/// Shape B teardown where the install key is the source.
fn reverse_scoped_tree_by_source(
    tree: &ScopedTree,
    entity: Entity,
    source: &str,
    world: &mut World,
) {
    match tree {
        ScopedTree::Fire(effect) => {
            reverse_all_by_source_dispatch(effect, entity, source, world);
        }
        ScopedTree::Sequence(effects) => {
            for effect in effects {
                reverse_all_by_source_dispatch(effect, entity, source, world);
            }
        }
        ScopedTree::When(..) | ScopedTree::On(..) | ScopedTree::During(..) => {}
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::CommandQueue;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::{
        effect_v3::{
            conditions::{DuringActive, evaluate_conditions},
            effects::{DamageBoostConfig, SpeedBoostConfig},
            stacking::EffectStack,
            types::{Condition, EffectType, ReversibleEffectType, Tree},
            walking::walk_effects::walk_effects,
        },
        state::types::NodeState,
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
            walk_effects(
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
            walk_effects(
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
            walk_effects(
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
            walk_effects(
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
            walk_effects(
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
            walk_effects(
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
            walk_effects(
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
            walk_effects(
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
            walk_effects(
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
            walk_effects(
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
            walk_effects(
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
            walk_effects(
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
            walk_effects(
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
}
