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

    use super::{
        on_bump_occurred, on_bump_whiff_occurred, on_bumped, on_early_bump_occurred,
        on_early_bumped, on_late_bump_occurred, on_late_bumped, on_no_bump_occurred,
        on_perfect_bump_occurred, on_perfect_bumped,
    };
    use crate::{
        breaker::messages::{BumpGrade, BumpPerformed, BumpWhiffed, NoBump},
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

    // ====================================================================
    // BumpPerformed bridge helpers
    // ====================================================================

    /// Resource to inject `BumpPerformed` messages into the test app.
    #[derive(Resource, Default)]
    struct TestBumpPerformedMessages(Vec<BumpPerformed>);

    /// System that writes `BumpPerformed` messages from the test resource.
    fn inject_bump_performed(
        messages: Res<TestBumpPerformedMessages>,
        mut writer: MessageWriter<BumpPerformed>,
    ) {
        for msg in &messages.0 {
            writer.write(msg.clone());
        }
    }

    /// Resource to inject `BumpWhiffed` messages into the test app.
    #[derive(Resource, Default)]
    struct TestBumpWhiffedMessages(Vec<BumpWhiffed>);

    /// System that writes `BumpWhiffed` messages from the test resource.
    fn inject_bump_whiffed(
        messages: Res<TestBumpWhiffedMessages>,
        mut writer: MessageWriter<BumpWhiffed>,
    ) {
        for msg in &messages.0 {
            writer.write(msg.clone());
        }
    }

    // -- Per-bridge app builders ------------------------------------------

    fn bump_performed_test_app<M>(
        systems: impl IntoScheduleConfigs<bevy::ecs::system::ScheduleSystem, M>,
    ) -> App {
        TestAppBuilder::new()
            .with_message::<BumpPerformed>()
            .with_resource::<TestBumpPerformedMessages>()
            .with_system(FixedUpdate, systems)
            .build()
    }

    fn bump_whiff_occurred_test_app() -> App {
        TestAppBuilder::new()
            .with_message::<BumpWhiffed>()
            .with_resource::<TestBumpWhiffedMessages>()
            .with_system(
                FixedUpdate,
                (
                    inject_bump_whiffed.before(on_bump_whiff_occurred),
                    on_bump_whiff_occurred,
                ),
            )
            .build()
    }

    // -- Tree helpers for bump bridges ------------------------------------

    fn speed_tree(name: &str, trigger: Trigger, multiplier: f32) -> (String, Tree) {
        (
            name.to_string(),
            Tree::When(
                trigger,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(multiplier),
                }))),
            ),
        )
    }

    fn on_target_tree(
        name: &str,
        trigger: Trigger,
        target: BumpTarget,
        multiplier: f32,
    ) -> (String, Tree) {
        (
            name.to_string(),
            Tree::When(
                trigger,
                Box::new(Tree::On(
                    ParticipantTarget::Bump(target),
                    Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(multiplier),
                    })),
                )),
            ),
        )
    }

    // ====================================================================
    // Behavior 14: on_bumped walks breaker entity on any grade
    // ====================================================================

    #[test]
    fn on_bumped_walks_breaker_on_perfect_grade() {
        let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::Bumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("breaker should have EffectStack");
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn on_bumped_walks_breaker_on_early_grade() {
        let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::Bumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Early,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("breaker should have EffectStack for Early grade");
        assert_eq!(stack.len(), 1);
    }

    // ====================================================================
    // Behavior 15: on_bumped walks bolt entity when bolt is Some
    // ====================================================================

    #[test]
    fn on_bumped_walks_bolt_entity_when_bolt_is_some() {
        let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

        let bolt_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::Bumped,
                2.0,
            )]))
            .id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Late,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
            .expect("bolt should have EffectStack");
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn on_bumped_skips_bolt_without_bound_effects() {
        let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::Bumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        // Bolt should be silently skipped (no BoundEffects), no panic
        let bolt_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity);
        assert!(
            bolt_stack.is_none(),
            "bolt without BoundEffects should not have EffectStack"
        );

        // Breaker still gets the effect
        let breaker_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("breaker should still get EffectStack");
        assert_eq!(breaker_stack.len(), 1);
    }

    // ====================================================================
    // Behavior 16: on_bumped skips bolt when bolt is None
    // ====================================================================

    #[test]
    fn on_bumped_skips_bolt_when_none() {
        let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::Bumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    None,
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("breaker should have EffectStack despite None bolt");
        assert_eq!(stack.len(), 1);
    }

    // ====================================================================
    // Behavior 17: on_bumped provides TriggerContext::Bump with participants
    // ====================================================================

    #[test]
    fn on_bumped_provides_bump_context_on_resolves_to_bolt() {
        let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![on_target_tree(
                "chip_a",
                Trigger::Bumped,
                BumpTarget::Bolt,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let bolt_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
            .expect("bolt_entity should gain EffectStack via On(Bump(Bolt))");
        assert_eq!(bolt_stack.len(), 1);

        let breaker_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
        assert!(
            breaker_stack.is_none(),
            "breaker should NOT have EffectStack — effect redirected to bolt"
        );
    }

    #[test]
    fn on_bumped_on_bump_breaker_resolves_to_breaker() {
        let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

        let bolt_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![on_target_tree(
                "chip_a",
                Trigger::Bumped,
                BumpTarget::Breaker,
                1.5,
            )]))
            .id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let breaker_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("breaker_entity should gain EffectStack via On(Bump(Breaker))");
        assert_eq!(breaker_stack.len(), 1);
    }

    // ====================================================================
    // Behavior 18: on_bumped is no-op without messages
    // ====================================================================

    #[test]
    fn on_bumped_is_noop_without_messages() {
        let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::Bumped,
                1.5,
            )]))
            .id();

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(stack.is_none(), "no EffectStack without messages");
    }

    // ====================================================================
    // Behavior 19: on_bumped does not walk third-party entities (local scope)
    // ====================================================================

    #[test]
    fn on_bumped_does_not_walk_bystander_entities() {
        let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::Bumped,
                1.5,
            )]))
            .id();

        // Bystander Entity C — has BoundEffects but is not bolt or breaker
        let bystander = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_c",
                Trigger::Bumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        // Breaker gets the effect (is a participant)
        let breaker_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("breaker should have EffectStack");
        assert_eq!(breaker_stack.len(), 1);

        // Bolt has no BoundEffects, so no effect
        let bolt_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity);
        assert!(bolt_stack.is_none(), "bolt without BoundEffects skipped");

        // Bystander should NOT be walked — local dispatch only
        let bystander_stack = app.world().get::<EffectStack<SpeedBoostConfig>>(bystander);
        assert!(
            bystander_stack.is_none(),
            "bystander entity should NOT be walked by local dispatch"
        );
    }

    // ====================================================================
    // Behavior 20: on_bumped handles multiple messages in one frame
    // ====================================================================

    #[test]
    fn on_bumped_handles_multiple_messages_per_frame() {
        let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

        let bolt_a = app.world_mut().spawn_empty().id();
        let bolt_b = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::Bumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![
            BumpPerformed {
                grade:   BumpGrade::Perfect,
                bolt:    Some(bolt_a),
                breaker: breaker_entity,
            },
            BumpPerformed {
                grade:   BumpGrade::Late,
                bolt:    Some(bolt_b),
                breaker: breaker_entity,
            },
        ]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("breaker should have EffectStack from two messages");
        assert_eq!(
            stack.len(),
            2,
            "each BumpPerformed message should trigger a separate walk"
        );
    }

    // ====================================================================
    // Behavior 21: on_perfect_bumped fires only on Perfect
    // ====================================================================

    #[test]
    fn on_perfect_bumped_fires_on_perfect_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_perfect_bumped),
            on_perfect_bumped,
        ));

        let bolt_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_b",
                Trigger::PerfectBumped,
                1.5,
            )]))
            .id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::PerfectBumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let breaker_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("breaker should have EffectStack");
        assert_eq!(breaker_stack.len(), 1);

        // Bolt also gets walked (local dispatch)
        let bolt_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
            .expect("bolt should also gain EffectStack from local dispatch");
        assert_eq!(bolt_stack.len(), 1);
    }

    // ====================================================================
    // Behavior 22: on_perfect_bumped filters out Early
    // ====================================================================

    #[test]
    fn on_perfect_bumped_filters_out_early_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_perfect_bumped),
            on_perfect_bumped,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::PerfectBumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Early,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
        assert!(
            stack.is_none(),
            "PerfectBumped should not fire on Early grade"
        );
    }

    // ====================================================================
    // Behavior 23: on_perfect_bumped filters out Late
    // ====================================================================

    #[test]
    fn on_perfect_bumped_filters_out_late_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_perfect_bumped),
            on_perfect_bumped,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::PerfectBumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Late,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
        assert!(
            stack.is_none(),
            "PerfectBumped should not fire on Late grade"
        );
    }

    // ====================================================================
    // Behavior 24: on_perfect_bumped provides TriggerContext::Bump
    // ====================================================================

    #[test]
    fn on_perfect_bumped_provides_bump_context_on_resolves_to_breaker() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_perfect_bumped),
            on_perfect_bumped,
        ));

        let bolt_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![on_target_tree(
                "chip_a",
                Trigger::PerfectBumped,
                BumpTarget::Breaker,
                1.5,
            )]))
            .id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let breaker_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("breaker_entity should gain EffectStack via On(Bump(Breaker))");
        assert_eq!(breaker_stack.len(), 1);

        let bolt_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity);
        assert!(
            bolt_stack.is_none(),
            "bolt should NOT have EffectStack — effect redirected to breaker"
        );
    }

    // ====================================================================
    // Behavior 25: on_early_bumped fires only on Early
    // ====================================================================

    #[test]
    fn on_early_bumped_fires_on_early_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_early_bumped),
            on_early_bumped,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::EarlyBumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Early,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("breaker should have EffectStack for Early");
        assert_eq!(stack.len(), 1);
    }

    // ====================================================================
    // Behavior 26: on_early_bumped filters out Perfect
    // ====================================================================

    #[test]
    fn on_early_bumped_filters_out_perfect_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_early_bumped),
            on_early_bumped,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::EarlyBumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
        assert!(
            stack.is_none(),
            "EarlyBumped should not fire on Perfect grade"
        );
    }

    // ====================================================================
    // Behavior 27: on_early_bumped filters out Late
    // ====================================================================

    #[test]
    fn on_early_bumped_filters_out_late_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_early_bumped),
            on_early_bumped,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::EarlyBumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Late,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
        assert!(stack.is_none(), "EarlyBumped should not fire on Late grade");
    }

    // ====================================================================
    // Behavior 28: on_late_bumped fires only on Late
    // ====================================================================

    #[test]
    fn on_late_bumped_fires_on_late_grade() {
        let mut app =
            bump_performed_test_app((inject_bump_performed.before(on_late_bumped), on_late_bumped));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::LateBumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Late,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("breaker should have EffectStack for Late");
        assert_eq!(stack.len(), 1);
    }

    // ====================================================================
    // Behavior 29: on_late_bumped filters out Perfect
    // ====================================================================

    #[test]
    fn on_late_bumped_filters_out_perfect_grade() {
        let mut app =
            bump_performed_test_app((inject_bump_performed.before(on_late_bumped), on_late_bumped));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::LateBumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
        assert!(
            stack.is_none(),
            "LateBumped should not fire on Perfect grade"
        );
    }

    // ====================================================================
    // Behavior 30: on_late_bumped filters out Early
    // ====================================================================

    #[test]
    fn on_late_bumped_filters_out_early_grade() {
        let mut app =
            bump_performed_test_app((inject_bump_performed.before(on_late_bumped), on_late_bumped));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::LateBumped,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Early,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
        assert!(stack.is_none(), "LateBumped should not fire on Early grade");
    }

    // ====================================================================
    // Behavior 31: on_bump_occurred walks ALL entities on any grade
    // ====================================================================

    #[test]
    fn on_bump_occurred_walks_all_entities_with_bound_effects_any_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_bump_occurred),
            on_bump_occurred,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity_a = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::BumpOccurred,
                1.5,
            )]))
            .id();

        let entity_b = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_b",
                Trigger::BumpOccurred,
                2.0,
            )]))
            .id();

        // entity_c has no BoundEffects
        let entity_c = app.world_mut().spawn_empty().id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Early,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack_a = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .expect("entity_a should have EffectStack");
        assert_eq!(stack_a.len(), 1);

        let stack_b = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_b)
            .expect("entity_b should have EffectStack");
        assert_eq!(stack_b.len(), 1);

        let stack_c = app.world().get::<EffectStack<SpeedBoostConfig>>(entity_c);
        assert!(stack_c.is_none(), "entity_c without BoundEffects: no stack");
    }

    // ====================================================================
    // Behavior 32: on_bump_occurred provides TriggerContext::Bump
    // ====================================================================

    #[test]
    fn on_bump_occurred_provides_bump_context_on_resolves_to_bolt() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_bump_occurred),
            on_bump_occurred,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let owner = app
            .world_mut()
            .spawn(BoundEffects(vec![on_target_tree(
                "chip_a",
                Trigger::BumpOccurred,
                BumpTarget::Bolt,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let bolt_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
            .expect("bolt_entity should gain EffectStack via On(Bump(Bolt))");
        assert_eq!(bolt_stack.len(), 1);

        let owner_stack = app.world().get::<EffectStack<SpeedBoostConfig>>(owner);
        assert!(
            owner_stack.is_none(),
            "owner should NOT have EffectStack — redirected to bolt"
        );
    }

    #[test]
    fn on_bump_occurred_on_bump_breaker_resolves_to_breaker() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_bump_occurred),
            on_bump_occurred,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let _owner = app
            .world_mut()
            .spawn(BoundEffects(vec![on_target_tree(
                "chip_a",
                Trigger::BumpOccurred,
                BumpTarget::Breaker,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let breaker_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("breaker_entity should gain EffectStack via On(Bump(Breaker))");
        assert_eq!(breaker_stack.len(), 1);
    }

    // ====================================================================
    // Behavior 33: on_bump_occurred is no-op without messages
    // ====================================================================

    #[test]
    fn on_bump_occurred_is_noop_without_messages() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_bump_occurred),
            on_bump_occurred,
        ));

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::BumpOccurred,
                1.5,
            )]))
            .id();

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(stack.is_none(), "no EffectStack without messages");
    }

    // ====================================================================
    // Behavior 34: on_bump_occurred handles multiple messages in one frame
    // ====================================================================

    #[test]
    fn on_bump_occurred_handles_multiple_messages_per_frame() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_bump_occurred),
            on_bump_occurred,
        ));

        let bolt_a = app.world_mut().spawn_empty().id();
        let breaker_a = app.world_mut().spawn_empty().id();
        let bolt_b = app.world_mut().spawn_empty().id();
        let breaker_b = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::BumpOccurred,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![
            BumpPerformed {
                grade:   BumpGrade::Perfect,
                bolt:    Some(bolt_a),
                breaker: breaker_a,
            },
            BumpPerformed {
                grade:   BumpGrade::Late,
                bolt:    Some(bolt_b),
                breaker: breaker_b,
            },
        ]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist after two messages");
        assert_eq!(stack.len(), 2);
    }

    // ====================================================================
    // Behavior 35: on_perfect_bump_occurred fires on all entities for Perfect
    // ====================================================================

    #[test]
    fn on_perfect_bump_occurred_fires_on_all_entities_for_perfect_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_perfect_bump_occurred),
            on_perfect_bump_occurred,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity_a = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::PerfectBumpOccurred,
                1.5,
            )]))
            .id();

        let entity_b = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_b",
                Trigger::PerfectBumpOccurred,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack_a = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .expect("entity_a should have EffectStack");
        assert_eq!(stack_a.len(), 1);

        let stack_b = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_b)
            .expect("entity_b should have EffectStack");
        assert_eq!(stack_b.len(), 1);
    }

    // ====================================================================
    // Behavior 36: on_perfect_bump_occurred filters out Early and Late
    // ====================================================================

    #[test]
    fn on_perfect_bump_occurred_filters_out_early_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_perfect_bump_occurred),
            on_perfect_bump_occurred,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::PerfectBumpOccurred,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Early,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "PerfectBumpOccurred should not fire on Early grade"
        );
    }

    #[test]
    fn on_perfect_bump_occurred_filters_out_late_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_perfect_bump_occurred),
            on_perfect_bump_occurred,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::PerfectBumpOccurred,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Late,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "PerfectBumpOccurred should not fire on Late grade"
        );
    }

    // ====================================================================
    // Behavior 37: on_perfect_bump_occurred provides TriggerContext::Bump
    // ====================================================================

    #[test]
    fn on_perfect_bump_occurred_provides_bump_context_resolves_to_breaker() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_perfect_bump_occurred),
            on_perfect_bump_occurred,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let _entity = app
            .world_mut()
            .spawn(BoundEffects(vec![on_target_tree(
                "chip_a",
                Trigger::PerfectBumpOccurred,
                BumpTarget::Breaker,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let breaker_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("breaker_entity should gain EffectStack via On(Bump(Breaker))");
        assert_eq!(breaker_stack.len(), 1);
    }

    // ====================================================================
    // Behavior 38: on_early_bump_occurred fires only for Early
    // ====================================================================

    #[test]
    fn on_early_bump_occurred_fires_on_early_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_early_bump_occurred),
            on_early_bump_occurred,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::EarlyBumpOccurred,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Early,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack for Early");
        assert_eq!(stack.len(), 1);
    }

    // ====================================================================
    // Behavior 39: on_early_bump_occurred filters out Perfect and Late
    // ====================================================================

    #[test]
    fn on_early_bump_occurred_filters_out_perfect_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_early_bump_occurred),
            on_early_bump_occurred,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::EarlyBumpOccurred,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "EarlyBumpOccurred should not fire on Perfect grade"
        );
    }

    #[test]
    fn on_early_bump_occurred_filters_out_late_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_early_bump_occurred),
            on_early_bump_occurred,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::EarlyBumpOccurred,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Late,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "EarlyBumpOccurred should not fire on Late grade"
        );
    }

    // ====================================================================
    // Behavior 40: on_late_bump_occurred fires only for Late
    // ====================================================================

    #[test]
    fn on_late_bump_occurred_fires_on_late_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_late_bump_occurred),
            on_late_bump_occurred,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::LateBumpOccurred,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Late,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack for Late");
        assert_eq!(stack.len(), 1);
    }

    // ====================================================================
    // Behavior 41: on_late_bump_occurred filters out Perfect and Early
    // ====================================================================

    #[test]
    fn on_late_bump_occurred_filters_out_perfect_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_late_bump_occurred),
            on_late_bump_occurred,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::LateBumpOccurred,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "LateBumpOccurred should not fire on Perfect grade"
        );
    }

    #[test]
    fn on_late_bump_occurred_filters_out_early_grade() {
        let mut app = bump_performed_test_app((
            inject_bump_performed.before(on_late_bump_occurred),
            on_late_bump_occurred,
        ));

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::LateBumpOccurred,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade:   BumpGrade::Early,
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        }]));

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "LateBumpOccurred should not fire on Early grade"
        );
    }

    // ====================================================================
    // Behavior 42: on_bump_whiff_occurred walks all entities
    // ====================================================================

    #[test]
    fn on_bump_whiff_occurred_walks_all_entities_with_bound_effects() {
        let mut app = bump_whiff_occurred_test_app();

        let entity_a = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::BumpWhiffOccurred,
                1.5,
            )]))
            .id();

        let entity_b = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_b",
                Trigger::BumpWhiffOccurred,
                1.5,
            )]))
            .id();

        // Entity without BoundEffects
        let entity_c = app.world_mut().spawn_empty().id();

        app.insert_resource(TestBumpWhiffedMessages(vec![BumpWhiffed]));

        tick(&mut app);

        let stack_a = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .expect("entity_a should have EffectStack");
        assert_eq!(stack_a.len(), 1);

        let stack_b = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_b)
            .expect("entity_b should have EffectStack");
        assert_eq!(stack_b.len(), 1);

        let stack_c = app.world().get::<EffectStack<SpeedBoostConfig>>(entity_c);
        assert!(stack_c.is_none(), "entity_c without BoundEffects: no stack");
    }

    // ====================================================================
    // Behavior 43: on_bump_whiff_occurred uses TriggerContext::None
    // ====================================================================

    #[test]
    fn on_bump_whiff_occurred_uses_trigger_context_none_on_bump_cannot_resolve() {
        let mut app = bump_whiff_occurred_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![on_target_tree(
                "chip_a",
                Trigger::BumpWhiffOccurred,
                BumpTarget::Bolt,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpWhiffedMessages(vec![BumpWhiffed]));

        tick(&mut app);

        let bolt_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity);
        assert!(
            bolt_stack.is_none(),
            "On(Bump(Bolt)) should not resolve with TriggerContext::None"
        );

        let owner_stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            owner_stack.is_none(),
            "effect should not fire on owner when On cannot resolve"
        );
    }

    // ====================================================================
    // Behavior 44: on_bump_whiff_occurred is no-op without messages
    // ====================================================================

    #[test]
    fn on_bump_whiff_occurred_is_noop_without_messages() {
        let mut app = bump_whiff_occurred_test_app();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::BumpWhiffOccurred,
                1.5,
            )]))
            .id();

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(stack.is_none(), "no EffectStack without messages");
    }

    // ====================================================================
    // Behavior 45: on_bump_whiff_occurred handles multiple whiff messages
    // ====================================================================

    #[test]
    fn on_bump_whiff_occurred_handles_multiple_messages_per_frame() {
        let mut app = bump_whiff_occurred_test_app();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![speed_tree(
                "chip_a",
                Trigger::BumpWhiffOccurred,
                1.5,
            )]))
            .id();

        app.insert_resource(TestBumpWhiffedMessages(vec![BumpWhiffed, BumpWhiffed]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist after two whiff messages");
        assert_eq!(stack.len(), 2);
    }
}
