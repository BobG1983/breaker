use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use super::super::system::*;
use crate::{
    effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::StagedEffects,
        types::{
            BumpTarget, Condition, EffectType, ParticipantTarget, ReversibleEffectType, ScopedTree,
            Terminal, Tree, Trigger, TriggerContext,
        },
    },
    prelude::DamageBoostStack,
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
        .get::<DamageBoostStack>(entity)
        .expect("DamageBoost should have fired");
    assert!(!dmg.is_empty());
    assert!(
        (dmg.aggregate_persistent() - 2.0).abs() < 1e-5,
        "DamageBoostStack aggregate should be 2.0, got {}",
        dmg.aggregate_persistent()
    );
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
        world.get::<DamageBoostStack>(entity).is_none(),
        "DamageBoostStack must not be inserted — inner When is staged, not fired"
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
