use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use super::super::super::system::*;
use crate::{
    effect_v3::{
        conditions::evaluate_conditions,
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::BoundEffects,
        types::{Condition, ReversibleEffectType, ScopedTree, Tree, Trigger, TriggerContext},
        walking::walk_effects::walk_bound_effects,
    },
    state::types::NodeState,
};

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
        walk_bound_effects(
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
        walk_bound_effects(
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
        walk_bound_effects(
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
        walk_bound_effects(
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
