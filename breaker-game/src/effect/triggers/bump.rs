//! Bridge system for the `bump` trigger.
use bevy::prelude::*;

use crate::{
    breaker::sets::BreakerSystems,
    effect::{
        core::*,
        sets::EffectSystems,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    prelude::*,
};

fn bridge_bump(
    mut reader: MessageReader<BumpPerformed>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext {
            bolt: msg.bolt,
            breaker: Some(msg.breaker),
            ..default()
        };
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Bump,
                entity,
                bound,
                &mut staged,
                &mut commands,
                context,
            );
            evaluate_staged_effects(&Trigger::Bump, entity, &mut staged, &mut commands, context);
        }
    }
}

/// Register trigger bridge systems.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        bridge_bump
            .in_set(EffectSystems::Bridge)
            .after(BreakerSystems::GradeBump)
            .run_if(in_state(NodeState::Playing)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{breaker::messages::BumpGrade, effect::effects::speed_boost::ActiveSpeedBoosts};

    #[derive(Resource)]
    struct TestBumpMsg(Option<BumpPerformed>);

    fn enqueue_bump(msg_res: Res<TestBumpMsg>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .add_systems(FixedUpdate, (enqueue_bump.before(bridge_bump), bridge_bump));
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    /// Spawns a breaker entity via the builder with `StagedEffects`.
    fn spawn_test_breaker(app: &mut App, primary: bool) -> Entity {
        use crate::breaker::{components::Breaker, definition::BreakerDefinition};
        let def = BreakerDefinition::default();
        let entity = if primary {
            app.world_mut()
                .spawn(
                    Breaker::builder()
                        .definition(&def)
                        .headless()
                        .primary()
                        .build(),
                )
                .id()
        } else {
            app.world_mut()
                .spawn(
                    Breaker::builder()
                        .definition(&def)
                        .headless()
                        .extra()
                        .build(),
                )
                .id()
        };
        app.world_mut()
            .entity_mut(entity)
            .insert(StagedEffects::default());
        entity
    }

    #[test]
    fn bridge_bump_fires_on_any_grade() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn_empty().id();
        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: None,
            breaker,
        })));
        app.world_mut().spawn((
            BoundEffects(vec![(
                "test".into(),
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
            )]),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ));

        tick(&mut app);

        let active = app
            .world_mut()
            .query::<&ActiveSpeedBoosts>()
            .single(app.world())
            .unwrap();
        assert_eq!(
            active.0.len(),
            1,
            "bridge_bump should fire SpeedBoost on BumpPerformed with any grade"
        );
        assert!(
            (active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );
    }

    // ── Context: bolt present → retargets to specific bolt ──

    #[test]
    fn bump_context_resolves_to_specific_bolt() {
        use crate::bolt::components::Bolt;

        let mut app = test_app();
        let breaker = app.world_mut().spawn_empty().id();

        let bolt_a = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
        let bolt_b = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
        let bolt_c = app.world_mut().spawn((Bolt, StagedEffects::default())).id();

        app.world_mut().spawn((
            BoundEffects(vec![(
                "ctx_test".into(),
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::On {
                        target: Target::Bolt,
                        permanent: false,
                        then: vec![EffectNode::When {
                            trigger: Trigger::Died,
                            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                        }],
                    }],
                },
            )]),
            StagedEffects::default(),
        ));

        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt_b),
            breaker,
        })));

        tick(&mut app);

        let staged_b = app.world().get::<StagedEffects>(bolt_b).unwrap();
        assert!(
            !staged_b.0.is_empty(),
            "bolt_b SHOULD have staged effects — it was the bumped bolt"
        );
        let staged_a = app.world().get::<StagedEffects>(bolt_a).unwrap();
        let staged_c = app.world().get::<StagedEffects>(bolt_c).unwrap();
        assert!(
            staged_a.0.is_empty(),
            "bolt_a should have no staged effects"
        );
        assert!(
            staged_c.0.is_empty(),
            "bolt_c should have no staged effects"
        );
    }

    // ── Context: bolt absent → fallback to PrimaryBolt ──

    #[test]
    fn bump_no_bolt_context_falls_back_to_primary_bolt() {
        use crate::bolt::components::{Bolt, PrimaryBolt};

        let mut app = test_app();
        let breaker = app.world_mut().spawn_empty().id();

        let primary = app
            .world_mut()
            .spawn((Bolt, PrimaryBolt, StagedEffects::default()))
            .id();
        let secondary = app.world_mut().spawn((Bolt, StagedEffects::default())).id();

        app.world_mut().spawn((
            BoundEffects(vec![(
                "ctx_test".into(),
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::On {
                        target: Target::Bolt,
                        permanent: false,
                        then: vec![EffectNode::When {
                            trigger: Trigger::Died,
                            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                        }],
                    }],
                },
            )]),
            StagedEffects::default(),
        ));

        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: None,
            breaker,
        })));

        tick(&mut app);

        let primary_staged = app.world().get::<StagedEffects>(primary).unwrap();
        assert!(
            !primary_staged.0.is_empty(),
            "PrimaryBolt should get the effect — no context, falls back to default"
        );
        let secondary_staged = app.world().get::<StagedEffects>(secondary).unwrap();
        assert!(
            secondary_staged.0.is_empty(),
            "Non-primary bolt should NOT get the effect"
        );
    }

    // ── Corner case 1: dual retarget — both bolt and breaker from one Bump ──

    #[test]
    fn bump_dual_retarget_resolves_both_bolt_and_breaker() {
        use crate::bolt::components::Bolt;

        let mut app = test_app();

        let bolt_a = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
        let bolt_b = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
        let breaker_a = spawn_test_breaker(&mut app, true);
        let breaker_b = spawn_test_breaker(&mut app, false);

        // Observer: When(Bump, [On(Bolt, ...), On(Breaker, ...)])
        app.world_mut().spawn((
            BoundEffects(vec![
                (
                    "dual_bolt".into(),
                    EffectNode::When {
                        trigger: Trigger::Bump,
                        then: vec![EffectNode::On {
                            target: Target::Bolt,
                            permanent: false,
                            then: vec![EffectNode::When {
                                trigger: Trigger::Died,
                                then: vec![EffectNode::Do(EffectKind::SpeedBoost {
                                    multiplier: 1.5,
                                })],
                            }],
                        }],
                    },
                ),
                (
                    "dual_breaker".into(),
                    EffectNode::When {
                        trigger: Trigger::Bump,
                        then: vec![EffectNode::On {
                            target: Target::Breaker,
                            permanent: false,
                            then: vec![EffectNode::When {
                                trigger: Trigger::Died,
                                then: vec![EffectNode::Do(EffectKind::SpeedBoost {
                                    multiplier: 2.0,
                                })],
                            }],
                        }],
                    },
                ),
            ]),
            StagedEffects::default(),
        ));

        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt_b),
            breaker: breaker_b,
        })));

        tick(&mut app);

        // bolt_b gets the bolt retarget
        assert!(
            !app.world()
                .get::<StagedEffects>(bolt_b)
                .unwrap()
                .0
                .is_empty(),
            "bolt_b SHOULD have staged effects"
        );
        assert!(
            app.world()
                .get::<StagedEffects>(bolt_a)
                .unwrap()
                .0
                .is_empty(),
            "bolt_a should be empty"
        );

        // breaker_b gets the breaker retarget
        assert!(
            !app.world()
                .get::<StagedEffects>(breaker_b)
                .unwrap()
                .0
                .is_empty(),
            "breaker_b SHOULD have staged effects"
        );
        assert!(
            app.world()
                .get::<StagedEffects>(breaker_a)
                .unwrap()
                .0
                .is_empty(),
            "breaker_a should be empty"
        );
    }

    // ── Corner case 2: bolt None + On(Breaker) still resolves ──

    #[test]
    fn bump_bolt_none_breaker_still_resolves() {
        use crate::breaker::components::Breaker;

        let mut app = test_app();

        let def = crate::breaker::definition::BreakerDefinition::default();
        let breaker_a = app
            .world_mut()
            .spawn(
                Breaker::builder()
                    .definition(&def)
                    .headless()
                    .primary()
                    .build(),
            )
            .id();
        app.world_mut()
            .entity_mut(breaker_a)
            .insert(StagedEffects::default());
        let breaker_b = app
            .world_mut()
            .spawn(
                Breaker::builder()
                    .definition(&def)
                    .headless()
                    .extra()
                    .build(),
            )
            .id();
        app.world_mut()
            .entity_mut(breaker_b)
            .insert(StagedEffects::default());

        app.world_mut().spawn((
            BoundEffects(vec![(
                "ctx_test".into(),
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::On {
                        target: Target::Breaker,
                        permanent: false,
                        then: vec![EffectNode::When {
                            trigger: Trigger::Died,
                            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                        }],
                    }],
                },
            )]),
            StagedEffects::default(),
        ));

        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: None,
            breaker: breaker_b,
        })));

        tick(&mut app);

        assert!(
            !app.world()
                .get::<StagedEffects>(breaker_b)
                .unwrap()
                .0
                .is_empty(),
            "breaker_b SHOULD have staged effects — bolt being None doesn't affect breaker context"
        );
        assert!(
            app.world()
                .get::<StagedEffects>(breaker_a)
                .unwrap()
                .0
                .is_empty(),
            "breaker_a should be empty"
        );
    }
}
