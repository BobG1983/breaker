//! When node evaluator — repeating trigger gate.

use bevy::prelude::*;

use super::walk_effects::evaluate_tree;
use crate::effect_v3::{
    commands::EffectCommandsExt,
    types::{Tree, Trigger, TriggerContext},
};

/// Evaluate a `Tree::When` node: if the trigger matches, evaluate the inner tree.
/// Repeats on every match.
///
/// When the inner tree's root is itself a trigger gate (`When`, `Once`,
/// `Until`), the outer `When` ARMS the inner by staging it under the
/// same source name instead of recursing. The staged entry is evaluated
/// on the *next* matching trigger — not the one that armed it. This
/// preserves the documented "new trigger event required" semantics of
/// nested gate types.
///
/// For non-gate inner trees (`Fire`, `Sequence`, `On`, `During`), the
/// previous recursion path is preserved.
pub fn evaluate_when(
    entity: Entity,
    gate_trigger: &Trigger,
    inner: &Tree,
    active_trigger: &Trigger,
    context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    if gate_trigger != active_trigger {
        return;
    }
    match inner {
        Tree::When(..) | Tree::Once(..) | Tree::Until(..) => {
            // Arm the inner gate by staging it under the same source.
            // The staged tree carries no context binding — when the
            // staged entry is later walked, it runs against the next
            // trigger's context, not the arming frame's context. This is
            // intentional.
            commands.stage_effect(entity, source.to_owned(), inner.clone());
        }
        _ => {
            evaluate_tree(entity, inner, active_trigger, context, source, commands);
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
        storage::{BoundEffects, StagedEffects},
        types::{
            BumpTarget, Condition, EffectType, ParticipantTarget, ReversibleEffectType, ScopedTree,
            Terminal,
        },
        walking::UntilApplied,
    };

    #[test]
    fn evaluate_when_matching_trigger_fires_inner() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let inner = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let gate = Trigger::Bumped;
        let active = Trigger::Bumped;
        let context = TriggerContext::None;

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &gate,
                &inner,
                &active,
                &context,
                "test_chip",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn evaluate_when_non_matching_trigger_does_nothing() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let inner = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let gate = Trigger::Bumped;
        let active = Trigger::BoltLostOccurred;
        let context = TriggerContext::None;

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &gate,
                &inner,
                &active,
                &context,
                "test_chip",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(stack.is_none());
    }

    // ================================================================
    // Wave C — When-arming for nested trigger gates
    // ================================================================

    // ----------------------------------------------------------------
    // Behavior 1: When(Bumped, When(Bumped, Fire(X))) — first matching
    //             trigger arms the inner When
    // ----------------------------------------------------------------
    #[test]
    fn when_arms_inner_when_on_first_matching_trigger() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.entity_mut(entity).insert(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                )),
            ),
        )]));

        let inner = Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        );

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &Trigger::Bumped,
                &inner,
                &Trigger::Bumped,
                &TriggerContext::None,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let staged = world
            .get::<StagedEffects>(entity)
            .expect("StagedEffects should be inserted when inner gate is armed");
        assert_eq!(staged.0.len(), 1);
        assert_eq!(staged.0[0].0, "chip_a");
        assert_eq!(
            staged.0[0].1,
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
            "staged entry must be exactly the inner When subtree"
        );

        assert!(
            world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
            "inner must NOT fire on the arming tick"
        );

        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1);
        assert_eq!(bound.0[0].0, "chip_a");
    }

    // ----------------------------------------------------------------
    // Behavior 2: Non-matching outer trigger arms nothing
    // ----------------------------------------------------------------
    #[test]
    fn when_arming_does_nothing_on_non_matching_outer_trigger() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let inner = Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        );

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &Trigger::Bumped,
                &inner,
                &Trigger::BoltLostOccurred,
                &TriggerContext::None,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        assert!(
            world.get::<StagedEffects>(entity).is_none(),
            "no StagedEffects should be inserted when outer trigger does not match"
        );
        assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
    }

    // ----------------------------------------------------------------
    // Behavior 3: When(Bumped, When(NoBumpOccurred, Fire(X))) — different
    //             inner trigger still arms
    // ----------------------------------------------------------------
    #[test]
    fn when_arms_inner_when_with_different_inner_trigger() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let inner = Tree::When(
            Trigger::NoBumpOccurred,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        );

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &Trigger::Bumped,
                &inner,
                &Trigger::Bumped,
                &TriggerContext::None,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 1);
        assert_eq!(
            staged.0[0],
            (
                "chip_a".to_string(),
                Tree::When(
                    Trigger::NoBumpOccurred,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                ),
            )
        );
        assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
    }

    // ----------------------------------------------------------------
    // Behavior 4: When(Bumped, Once(Bumped, Fire(X))) — arms the inner
    //             Once (preserves variant)
    // ----------------------------------------------------------------
    #[test]
    fn when_arms_inner_once_preserving_variant() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let inner = Tree::Once(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        );

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &Trigger::Bumped,
                &inner,
                &Trigger::Bumped,
                &TriggerContext::None,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 1);
        assert_eq!(
            staged.0[0],
            (
                "chip_a".to_string(),
                Tree::Once(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                ),
            )
        );
        assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
    }

    // ----------------------------------------------------------------
    // Behavior 5: When(Bumped, Until(Died, Fire(X))) — arms the inner
    //             Until (preserves variant, full value)
    // ----------------------------------------------------------------
    #[test]
    fn when_arms_inner_until_preserving_variant() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let inner = Tree::Until(
            Trigger::Died,
            Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                },
            ))),
        );

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &Trigger::Bumped,
                &inner,
                &Trigger::Bumped,
                &TriggerContext::None,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 1);
        assert_eq!(
            staged.0[0],
            (
                "chip_a".to_string(),
                Tree::Until(
                    Trigger::Died,
                    Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                        SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        },
                    ))),
                ),
            )
        );
        assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
        assert!(
            world.get::<UntilApplied>(entity).is_none(),
            "Until must NOT be evaluated during the arming walk"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 6: When(Bumped, Fire(X)) — non-gate Fire inner still
    //             evaluates recursively (regression)
    // ----------------------------------------------------------------
    #[test]
    fn when_non_gate_fire_inner_fires_immediately_not_armed() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let inner = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &Trigger::Bumped,
                &inner,
                &Trigger::Bumped,
                &TriggerContext::None,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("Fire inner should fire immediately");
        assert_eq!(stack.len(), 1);
        assert!(
            world.get::<StagedEffects>(entity).is_none(),
            "non-gate Fire inner must NOT be staged"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 7: When(Bumped, Sequence([Fire(X), Fire(Y)])) — non-gate
    //             Sequence inner still evaluates recursively
    // ----------------------------------------------------------------
    #[test]
    fn when_non_gate_sequence_inner_fires_immediately_not_armed() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let inner = Tree::Sequence(vec![
            Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            })),
            Terminal::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            })),
        ]);

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &Trigger::Bumped,
                &inner,
                &Trigger::Bumped,
                &TriggerContext::None,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let speed = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("SpeedBoost should have fired");
        assert_eq!(speed.len(), 1);
        let dmg = world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .expect("DamageBoost should have fired");
        assert_eq!(dmg.len(), 1);
        assert!(
            world.get::<StagedEffects>(entity).is_none(),
            "Sequence inner must not be staged"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 8: When(Bumped, On(Bump(Bolt), Fire(X))) — non-gate On
    //             inner still evaluates recursively
    // ----------------------------------------------------------------
    #[test]
    fn when_non_gate_on_inner_redirects_immediately_not_armed() {
        let mut world = World::new();
        let breaker = world.spawn_empty().id();
        let bolt = world.spawn_empty().id();

        let inner = Tree::On(
            ParticipantTarget::Bump(BumpTarget::Bolt),
            Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            })),
        );

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                breaker,
                &Trigger::Bumped,
                &inner,
                &Trigger::Bumped,
                &TriggerContext::Bump {
                    bolt: Some(bolt),
                    breaker,
                },
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let bolt_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .expect("On should have redirected effect to bolt");
        assert_eq!(bolt_stack.len(), 1);
        assert!(
            world
                .get::<EffectStack<SpeedBoostConfig>>(breaker)
                .is_none(),
            "breaker should not have received effect"
        );
        assert!(world.get::<StagedEffects>(bolt).is_none());
        assert!(world.get::<StagedEffects>(breaker).is_none());
    }

    // ----------------------------------------------------------------
    // Behavior 9: When(Bumped, During(cond, ...)) — non-gate During
    //             inner still recurses (arming regression guard)
    // ----------------------------------------------------------------
    #[test]
    fn when_non_gate_during_inner_is_not_armed() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let inner = Tree::During(
            Condition::NodeActive,
            Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                },
            ))),
        );

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &Trigger::Bumped,
                &inner,
                &Trigger::Bumped,
                &TriggerContext::None,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        assert!(
            world.get::<StagedEffects>(entity).is_none(),
            "During inner must NOT be armed — only trigger gates (When/Once/Until) are"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 12: Staging uses the same source name as the outer entry
    // ----------------------------------------------------------------
    #[test]
    fn when_arming_uses_same_source_name_as_outer_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let inner = Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        );

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &Trigger::Bumped,
                &inner,
                &Trigger::Bumped,
                &TriggerContext::None,
                "chip_xyz",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(
            staged.0[0].0, "chip_xyz",
            "staged name must be the exact source passed to evaluate_when"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 13: Two direct evaluate_when calls stage independent
    //              entries with distinct source names
    // ----------------------------------------------------------------
    #[test]
    fn when_arming_two_calls_append_independent_entries_in_order() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let inner_a = Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        );
        let inner_b = Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            }))),
        );

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &Trigger::Bumped,
                &inner_a,
                &Trigger::Bumped,
                &TriggerContext::None,
                "chip_a",
                &mut commands,
            );
            evaluate_when(
                entity,
                &Trigger::Bumped,
                &inner_b,
                &Trigger::Bumped,
                &TriggerContext::None,
                "chip_b",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 2);
        assert_eq!(
            staged.0[0],
            (
                "chip_a".to_string(),
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                ),
            )
        );
        assert_eq!(
            staged.0[1],
            (
                "chip_b".to_string(),
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    }))),
                ),
            )
        );
        assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
        assert!(
            world
                .get::<EffectStack<DamageBoostConfig>>(entity)
                .is_none()
        );
    }

    // ----------------------------------------------------------------
    // Behavior 14: Triple-nested When — a single evaluate_when call
    //              arms only one layer
    // ----------------------------------------------------------------
    #[test]
    fn when_arming_single_call_arms_only_one_layer() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let inner = Tree::When(
            Trigger::Bumped,
            Box::new(Tree::When(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            )),
        );

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &Trigger::Bumped,
                &inner,
                &Trigger::Bumped,
                &TriggerContext::None,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 1);
        assert_eq!(
            staged.0[0],
            (
                "chip_a".to_string(),
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::When(
                        Trigger::Bumped,
                        Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        }))),
                    )),
                ),
            )
        );
        assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
    }
}
