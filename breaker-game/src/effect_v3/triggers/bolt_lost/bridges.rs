//! Bolt lost trigger bridge system.
//!
//! Reads `BoltLost` messages and dispatches `BoltLostOccurred` triggers
//! to all entities with bound effects.

use bevy::prelude::*;

use crate::{
    bolt::messages::BoltLost,
    effect_v3::{
        storage::{BoundEffects, StagedEffects},
        types::{Trigger, TriggerContext},
        walking::{walk_bound_effects, walk_staged_effects},
    },
};

/// Global bridge: fires `BoltLostOccurred` on all entities with bound effects
/// when a bolt is lost.
pub fn on_bolt_lost_occurred(
    mut reader: MessageReader<BoltLost>,
    bound_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext::BoltLost {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        let trigger = Trigger::BoltLostOccurred;
        for (entity, bound, staged) in bound_query.iter() {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();
            walk_staged_effects(entity, &trigger, &context, &staged_trees, &mut commands);
            walk_bound_effects(entity, &trigger, &context, &bound_trees, &mut commands);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::on_bolt_lost_occurred;
    use crate::{
        bolt::messages::BoltLost,
        effect_v3::{
            effects::SpeedBoostConfig,
            stacking::EffectStack,
            storage::{BoundEffects, StagedEffects},
            types::{BoltLostTarget, EffectType, ParticipantTarget, Terminal, Tree, Trigger},
        },
        shared::test_utils::TestAppBuilder,
    };

    // -- Helpers ----------------------------------------------------------

    /// Resource to inject `BoltLost` messages into the test app.
    #[derive(Resource, Default)]
    struct TestBoltLostMessages(Vec<BoltLost>);

    /// System that writes `BoltLost` messages from the test resource.
    fn inject_bolt_lost(messages: Res<TestBoltLostMessages>, mut writer: MessageWriter<BoltLost>) {
        for msg in &messages.0 {
            writer.write(msg.clone());
        }
    }

    fn bridge_test_app() -> App {
        TestAppBuilder::new()
            .with_message::<BoltLost>()
            .with_resource::<TestBoltLostMessages>()
            .with_system(
                FixedUpdate,
                (
                    inject_bolt_lost.before(on_bolt_lost_occurred),
                    on_bolt_lost_occurred,
                ),
            )
            .build()
    }

    fn tick(app: &mut App) {
        crate::shared::test_utils::tick(app);
    }

    /// Helper to build a When(BoltLostOccurred, Fire(SpeedBoost)) tree.
    fn bolt_lost_speed_tree(name: &str, multiplier: f32) -> (String, Tree) {
        (
            name.to_string(),
            Tree::When(
                Trigger::BoltLostOccurred,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(multiplier),
                }))),
            ),
        )
    }

    /// Helper to build a When(BoltLostOccurred, On(BoltLost(target), Fire(SpeedBoost))) tree.
    fn bolt_lost_on_target_tree(
        name: &str,
        target: BoltLostTarget,
        multiplier: f32,
    ) -> (String, Tree) {
        (
            name.to_string(),
            Tree::When(
                Trigger::BoltLostOccurred,
                Box::new(Tree::On(
                    ParticipantTarget::BoltLost(target),
                    Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(multiplier),
                    })),
                )),
            ),
        )
    }

    // -- Behavior 6: dispatches BoltLostOccurred trigger to entities with BoundEffects --

    #[test]
    fn on_bolt_lost_occurred_dispatches_to_entity_with_bound_effects() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![bolt_lost_speed_tree("chip_a", 1.5)]))
            .id();

        app.insert_resource(TestBoltLostMessages(vec![BoltLost {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist on entity with BoundEffects");
        assert_eq!(stack.len(), 1, "entity should have one effect fired");
    }

    // -- Behavior 7: On(BoltLost(Bolt)) participant resolution to bolt entity --

    #[test]
    fn on_bolt_lost_occurred_resolves_bolt_lost_bolt_participant() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let owner = app
            .world_mut()
            .spawn(BoundEffects(vec![bolt_lost_on_target_tree(
                "chip_a",
                BoltLostTarget::Bolt,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBoltLostMessages(vec![BoltLost {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let bolt_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
            .expect("EffectStack should exist on bolt_entity via On(BoltLost(Bolt))");
        assert_eq!(bolt_stack.len(), 1);

        let owner_stack = app.world().get::<EffectStack<SpeedBoostConfig>>(owner);
        assert!(
            owner_stack.is_none(),
            "On node should redirect to bolt, not fire on owner"
        );
    }

    // -- Behavior 8: On(BoltLost(Breaker)) participant resolution to breaker entity --

    #[test]
    fn on_bolt_lost_occurred_resolves_bolt_lost_breaker_participant() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let owner = app
            .world_mut()
            .spawn(BoundEffects(vec![bolt_lost_on_target_tree(
                "chip_a",
                BoltLostTarget::Breaker,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBoltLostMessages(vec![BoltLost {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let breaker_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("EffectStack should exist on breaker_entity via On(BoltLost(Breaker))");
        assert_eq!(breaker_stack.len(), 1);

        let owner_stack = app.world().get::<EffectStack<SpeedBoostConfig>>(owner);
        assert!(
            owner_stack.is_none(),
            "On node should redirect to breaker, not fire on owner"
        );
    }

    // -- Behavior 9: walks all entities with BoundEffects (global dispatch) --

    #[test]
    fn on_bolt_lost_occurred_walks_all_entities_with_bound_effects() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity_a = app
            .world_mut()
            .spawn(BoundEffects(vec![bolt_lost_speed_tree("chip_a", 1.5)]))
            .id();

        let entity_b = app
            .world_mut()
            .spawn(BoundEffects(vec![bolt_lost_speed_tree("chip_b", 2.0)]))
            .id();

        app.insert_resource(TestBoltLostMessages(vec![BoltLost {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack_a = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .expect("EffectStack should exist on entity_a");
        assert_eq!(stack_a.len(), 1, "entity_a should have one effect fired");

        let stack_b = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_b)
            .expect("EffectStack should exist on entity_b");
        assert_eq!(stack_b.len(), 1, "entity_b should have one effect fired");
    }

    // -- Behavior 10: does not fire on non-matching trigger gates --

    #[test]
    fn on_bolt_lost_occurred_does_not_fire_on_non_matching_trigger() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        // BumpOccurred gate, NOT BoltLostOccurred
        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![(
                "chip_a".to_string(),
                Tree::When(
                    Trigger::BumpOccurred,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                ),
            )]))
            .id();

        app.insert_resource(TestBoltLostMessages(vec![BoltLost {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "BumpOccurred gate should not match BoltLostOccurred trigger"
        );
    }

    // -- Behavior 11: no-op when no BoltLost messages exist --

    #[test]
    fn on_bolt_lost_occurred_is_no_op_without_messages() {
        let mut app = bridge_test_app();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![bolt_lost_speed_tree("chip_a", 1.5)]))
            .id();

        // No messages injected (default empty resource)
        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "no EffectStack should exist when no BoltLost messages are sent"
        );
    }

    // -- Behavior 12: handles multiple BoltLost messages in one frame --

    #[test]
    fn on_bolt_lost_occurred_handles_multiple_messages_per_frame() {
        let mut app = bridge_test_app();

        let bolt_a = app.world_mut().spawn_empty().id();
        let bolt_b = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![bolt_lost_speed_tree("chip_a", 1.5)]))
            .id();

        app.insert_resource(TestBoltLostMessages(vec![
            BoltLost {
                bolt:    bolt_a,
                breaker: breaker_entity,
            },
            BoltLost {
                bolt:    bolt_b,
                breaker: breaker_entity,
            },
        ]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist after two BoltLost messages");
        assert_eq!(
            stack.len(),
            2,
            "each BoltLost message should trigger a separate walk"
        );
    }

    // -- Behavior 13: does not walk entities without BoundEffects --

    #[test]
    fn on_bolt_lost_occurred_does_not_walk_entities_without_bound_effects() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity_a = app
            .world_mut()
            .spawn(BoundEffects(vec![bolt_lost_speed_tree("chip_a", 1.5)]))
            .id();

        // entity_b has no BoundEffects
        let entity_b = app.world_mut().spawn_empty().id();

        app.insert_resource(TestBoltLostMessages(vec![BoltLost {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack_a = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .expect("EffectStack should exist on entity_a (has BoundEffects)");
        assert_eq!(stack_a.len(), 1);

        let stack_b = app.world().get::<EffectStack<SpeedBoostConfig>>(entity_b);
        assert!(
            stack_b.is_none(),
            "entity_b (no BoundEffects) should not have an EffectStack"
        );
    }

    // ================================================================
    // Wave C — Staged effect dispatch on bolt_lost bridge
    // ================================================================

    // ----------------------------------------------------------------
    // Behavior 23: staged entry whose inner trigger matches fires and
    //              is consumed on the same tick
    // ----------------------------------------------------------------
    #[test]
    fn on_bolt_lost_occurred_staged_entry_fires_and_is_consumed_on_match() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let owner = app
            .world_mut()
            .spawn((
                BoundEffects(vec![]),
                StagedEffects(vec![(
                    "chip_a".to_string(),
                    Tree::When(
                        Trigger::BoltLostOccurred,
                        Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        }))),
                    ),
                )]),
            ))
            .id();

        app.insert_resource(TestBoltLostMessages(vec![BoltLost {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(owner)
            .expect("staged When should have fired on matching trigger");
        assert_eq!(stack.len(), 1);

        let staged = app
            .world()
            .get::<StagedEffects>(owner)
            .expect("StagedEffects should still exist (empty)");
        assert!(
            staged.0.is_empty(),
            "staged entry must be consumed via commands.remove_effect"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 24: entity with StagedEffects always also has
    //              BoundEffects by construction — the bridge query
    //              picks it up via &BoundEffects
    // ----------------------------------------------------------------
    #[test]
    fn on_bolt_lost_occurred_staged_entity_with_empty_bound_still_fires() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let owner = app
            .world_mut()
            .spawn((
                BoundEffects(vec![]),
                StagedEffects(vec![(
                    "chip_a".to_string(),
                    Tree::When(
                        Trigger::BoltLostOccurred,
                        Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        }))),
                    ),
                )]),
            ))
            .id();

        app.insert_resource(TestBoltLostMessages(vec![BoltLost {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(owner)
            .expect("staged entry should fire even though BoundEffects is empty");
        assert_eq!(stack.len(), 1);

        let staged = app.world().get::<StagedEffects>(owner).unwrap();
        assert!(staged.0.is_empty(), "staged entry must be consumed");
    }

    // ----------------------------------------------------------------
    // Behavior 25: staged entries walk BEFORE bound entries
    //              (snapshot semantics — no same-tick arming fire)
    // ----------------------------------------------------------------
    #[test]
    fn on_bolt_lost_occurred_staged_walks_before_bound_no_same_tick_fire() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let owner = app
            .world_mut()
            .spawn(BoundEffects(vec![(
                "chip_a".to_string(),
                Tree::When(
                    Trigger::BoltLostOccurred,
                    Box::new(Tree::When(
                        Trigger::BoltLostOccurred,
                        Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        }))),
                    )),
                ),
            )]))
            .id();

        app.insert_resource(TestBoltLostMessages(vec![BoltLost {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let staged = app
            .world()
            .get::<StagedEffects>(owner)
            .expect("outer When should have armed the inner");
        assert_eq!(staged.0.len(), 1);

        assert!(
            app.world()
                .get::<EffectStack<SpeedBoostConfig>>(owner)
                .is_none(),
            "freshly armed staged entry MUST NOT fire in the same tick it was armed"
        );
    }
}
