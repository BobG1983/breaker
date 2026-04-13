//! Bump trigger bridge systems.
//!
//! Each bridge reads a bump message, builds a [`TriggerContext`], and dispatches
//! the corresponding trigger to entities with bound effects.

use bevy::prelude::*;

use crate::{
    breaker::messages::{BumpGrade, BumpPerformed, BumpWhiffed, NoBump},
    effect_v3::{
        storage::BoundEffects,
        types::{Trigger, TriggerContext},
        walking::walk_effects,
    },
};

// ---------------------------------------------------------------------------
// Local bump bridges — fire on bolt + breaker involved in the bump
// ---------------------------------------------------------------------------

/// Local bridge: fires `Bumped` on the bolt and breaker entities involved in a
/// successful bump of any grade.
pub fn on_bumped(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<&BoundEffects>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_local_bump(
            &Trigger::Bumped,
            &context,
            msg.bolt,
            msg.breaker,
            &bound_query,
            &mut commands,
        );
    }
}

/// Local bridge: fires `PerfectBumped` on the bolt and breaker entities involved
/// in a perfect-timed bump.
pub fn on_perfect_bumped(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<&BoundEffects>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Perfect {
            continue;
        }
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_local_bump(
            &Trigger::PerfectBumped,
            &context,
            msg.bolt,
            msg.breaker,
            &bound_query,
            &mut commands,
        );
    }
}

/// Local bridge: fires `EarlyBumped` on the bolt and breaker entities involved
/// in an early-timed bump.
pub fn on_early_bumped(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<&BoundEffects>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Early {
            continue;
        }
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_local_bump(
            &Trigger::EarlyBumped,
            &context,
            msg.bolt,
            msg.breaker,
            &bound_query,
            &mut commands,
        );
    }
}

/// Local bridge: fires `LateBumped` on the bolt and breaker entities involved
/// in a late-timed bump.
pub fn on_late_bumped(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<&BoundEffects>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Late {
            continue;
        }
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_local_bump(
            &Trigger::LateBumped,
            &context,
            msg.bolt,
            msg.breaker,
            &bound_query,
            &mut commands,
        );
    }
}

// ---------------------------------------------------------------------------
// Global bump bridges — fire on ALL entities with BoundEffects
// ---------------------------------------------------------------------------

/// Global bridge: fires `BumpOccurred` on all entities with bound effects when
/// any successful bump happens.
pub fn on_bump_occurred(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<(Entity, &BoundEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_global(
            &Trigger::BumpOccurred,
            &context,
            &bound_query,
            &mut commands,
        );
    }
}

/// Global bridge: fires `PerfectBumpOccurred` on all entities with bound effects
/// when a perfect bump happens.
pub fn on_perfect_bump_occurred(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<(Entity, &BoundEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Perfect {
            continue;
        }
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_global(
            &Trigger::PerfectBumpOccurred,
            &context,
            &bound_query,
            &mut commands,
        );
    }
}

/// Global bridge: fires `EarlyBumpOccurred` on all entities with bound effects
/// when an early bump happens.
pub fn on_early_bump_occurred(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<(Entity, &BoundEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Early {
            continue;
        }
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_global(
            &Trigger::EarlyBumpOccurred,
            &context,
            &bound_query,
            &mut commands,
        );
    }
}

/// Global bridge: fires `LateBumpOccurred` on all entities with bound effects
/// when a late bump happens.
pub fn on_late_bump_occurred(
    mut reader: MessageReader<BumpPerformed>,
    bound_query: Query<(Entity, &BoundEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Late {
            continue;
        }
        let context = TriggerContext::Bump {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        walk_global(
            &Trigger::LateBumpOccurred,
            &context,
            &bound_query,
            &mut commands,
        );
    }
}

/// Global bridge: fires `BumpWhiffOccurred` on all entities with bound effects
/// when a bump timing window expires without contact.
pub fn on_bump_whiff_occurred(
    mut reader: MessageReader<BumpWhiffed>,
    bound_query: Query<(Entity, &BoundEffects)>,
    mut commands: Commands,
) {
    for _ in reader.read() {
        let context = TriggerContext::None;
        walk_global(
            &Trigger::BumpWhiffOccurred,
            &context,
            &bound_query,
            &mut commands,
        );
    }
}

/// Global bridge: fires `NoBumpOccurred` on all entities with bound effects
/// when a bolt hits the breaker without any bump input.
pub fn on_no_bump_occurred(
    mut reader: MessageReader<NoBump>,
    bound_query: Query<(Entity, &BoundEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext::Bump {
            bolt:    Some(msg.bolt),
            breaker: msg.breaker,
        };
        walk_global(
            &Trigger::NoBumpOccurred,
            &context,
            &bound_query,
            &mut commands,
        );
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Walk effects on both bolt and breaker (local dispatch).
fn walk_local_bump(
    trigger: &Trigger,
    context: &TriggerContext,
    bolt: Option<Entity>,
    breaker: Entity,
    bound_query: &Query<&BoundEffects>,
    commands: &mut Commands,
) {
    // Walk breaker effects
    if let Ok(bound) = bound_query.get(breaker) {
        let trees = bound.0.clone();
        walk_effects(breaker, trigger, context, &trees, commands);
    }
    // Walk bolt effects (if bolt entity exists)
    if let Some(bolt_entity) = bolt
        && let Ok(bound) = bound_query.get(bolt_entity)
    {
        let trees = bound.0.clone();
        walk_effects(bolt_entity, trigger, context, &trees, commands);
    }
}

/// Walk effects on all entities with `BoundEffects` (global dispatch).
fn walk_global(
    trigger: &Trigger,
    context: &TriggerContext,
    bound_query: &Query<(Entity, &BoundEffects)>,
    commands: &mut Commands,
) {
    for (entity, bound) in bound_query.iter() {
        let trees = bound.0.clone();
        walk_effects(entity, trigger, context, &trees, commands);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::on_no_bump_occurred;
    use crate::{
        breaker::messages::NoBump,
        effect_v3::{
            effects::SpeedBoostConfig,
            stacking::EffectStack,
            storage::BoundEffects,
            types::{BumpTarget, EffectType, ParticipantTarget, Terminal, Tree, Trigger},
        },
        shared::test_utils::TestAppBuilder,
    };

    // -- Helpers ----------------------------------------------------------

    /// Resource to inject `NoBump` messages into the test app.
    #[derive(Resource, Default)]
    struct TestNoBumpMessages(Vec<NoBump>);

    /// System that writes `NoBump` messages from the test resource.
    fn inject_no_bumps(messages: Res<TestNoBumpMessages>, mut writer: MessageWriter<NoBump>) {
        for msg in &messages.0 {
            writer.write(msg.clone());
        }
    }

    fn bridge_test_app() -> App {
        TestAppBuilder::new()
            .with_message::<NoBump>()
            .with_resource::<TestNoBumpMessages>()
            .with_system(
                FixedUpdate,
                (
                    inject_no_bumps.before(on_no_bump_occurred),
                    on_no_bump_occurred,
                ),
            )
            .build()
    }

    fn tick(app: &mut App) {
        crate::shared::test_utils::tick(app);
    }

    /// Helper to build a When(NoBumpOccurred, Fire(SpeedBoost)) tree.
    fn no_bump_speed_tree(name: &str, multiplier: f32) -> (String, Tree) {
        (
            name.to_string(),
            Tree::When(
                Trigger::NoBumpOccurred,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(multiplier),
                }))),
            ),
        )
    }

    /// Helper to build a When(NoBumpOccurred, On(Bump(target), Fire(SpeedBoost))) tree.
    fn no_bump_on_target_tree(name: &str, target: BumpTarget, multiplier: f32) -> (String, Tree) {
        (
            name.to_string(),
            Tree::When(
                Trigger::NoBumpOccurred,
                Box::new(Tree::On(
                    ParticipantTarget::Bump(target),
                    Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(multiplier),
                    })),
                )),
            ),
        )
    }

    // -- Behavior 11: walks effects with NoBumpOccurred on all entities ----

    #[test]
    fn on_no_bump_occurred_walks_all_entities_with_bound_effects() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity_a = app
            .world_mut()
            .spawn(BoundEffects(vec![no_bump_speed_tree("chip_a", 1.5)]))
            .id();

        let entity_b = app
            .world_mut()
            .spawn(BoundEffects(vec![no_bump_speed_tree("chip_b", 2.0)]))
            .id();

        app.insert_resource(TestNoBumpMessages(vec![NoBump {
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

    // -- Behavior 12: does not fire on non-matching trigger gates ----------

    #[test]
    fn on_no_bump_occurred_does_not_fire_on_non_matching_trigger() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        // BumpOccurred gate, NOT NoBumpOccurred
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

        app.insert_resource(TestNoBumpMessages(vec![NoBump {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "BumpOccurred gate should not match NoBumpOccurred trigger"
        );
    }

    // -- Behavior 13: provides Bump TriggerContext, On(Bump(Bolt)) resolves -

    #[test]
    fn on_no_bump_occurred_provides_bump_context_with_bolt() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![no_bump_on_target_tree(
                "chip_a",
                BumpTarget::Bolt,
                1.5,
            )]))
            .id();

        app.insert_resource(TestNoBumpMessages(vec![NoBump {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let bolt_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
            .expect("EffectStack should exist on bolt_entity via On(Bump(Bolt))");
        assert_eq!(bolt_stack.len(), 1);

        let owner_stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            owner_stack.is_none(),
            "On node should redirect to bolt, not fire on owner"
        );
    }

    // -- Behavior 14: On(Bump(Breaker)) resolves from context ----

    #[test]
    fn on_no_bump_occurred_resolves_bump_breaker_participant() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![no_bump_on_target_tree(
                "chip_a",
                BumpTarget::Breaker,
                1.5,
            )]))
            .id();

        app.insert_resource(TestNoBumpMessages(vec![NoBump {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let breaker_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("EffectStack should exist on breaker_entity via On(Bump(Breaker))");
        assert_eq!(breaker_stack.len(), 1);

        let owner_stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            owner_stack.is_none(),
            "On node should redirect to breaker, not fire on owner"
        );
    }

    // -- Behavior 15: no-op when no NoBump messages exist ------------------

    #[test]
    fn on_no_bump_occurred_is_no_op_without_messages() {
        let mut app = bridge_test_app();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![no_bump_speed_tree("chip_a", 1.5)]))
            .id();

        // No messages injected (default empty resource)
        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "no EffectStack should exist when no NoBump messages are sent"
        );
    }

    // -- Behavior 16: handles multiple NoBump messages in one frame --------

    #[test]
    fn on_no_bump_occurred_handles_multiple_messages_per_frame() {
        let mut app = bridge_test_app();

        let bolt_a = app.world_mut().spawn_empty().id();
        let breaker_a = app.world_mut().spawn_empty().id();
        let bolt_b = app.world_mut().spawn_empty().id();
        let breaker_b = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![no_bump_speed_tree("chip_a", 1.5)]))
            .id();

        app.insert_resource(TestNoBumpMessages(vec![
            NoBump {
                bolt:    bolt_a,
                breaker: breaker_a,
            },
            NoBump {
                bolt:    bolt_b,
                breaker: breaker_b,
            },
        ]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist after two NoBump messages");
        assert_eq!(
            stack.len(),
            2,
            "each NoBump message should trigger a separate walk"
        );
    }

    // -- Behavior 17: does not walk entities without BoundEffects ----------

    #[test]
    fn on_no_bump_occurred_does_not_walk_entities_without_bound_effects() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity_a = app
            .world_mut()
            .spawn(BoundEffects(vec![no_bump_speed_tree("chip_a", 1.5)]))
            .id();

        // entity_b has no BoundEffects
        let entity_b = app.world_mut().spawn_empty().id();

        app.insert_resource(TestNoBumpMessages(vec![NoBump {
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
}
