//! Bridge system for the `bumped` trigger.
use bevy::prelude::*;

use crate::{
    breaker::{messages::BumpPerformed, sets::BreakerSystems},
    effect::{
        core::*,
        sets::EffectSystems,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    state::types::NodeState,
};

fn bridge_bumped(
    mut reader: MessageReader<BumpPerformed>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let Some(bolt) = msg.bolt else { continue };
        if let Ok((entity, bound, mut staged)) = query.get_mut(bolt) {
            let context = TriggerContext {
                breaker: Some(msg.breaker),
                ..default()
            };
            evaluate_bound_effects(
                &Trigger::Bumped,
                entity,
                bound,
                &mut staged,
                &mut commands,
                context,
            );
            evaluate_staged_effects(
                &Trigger::Bumped,
                entity,
                &mut staged,
                &mut commands,
                context,
            );
        }
    }
}

/// Register trigger bridge systems.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        bridge_bumped
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
            .add_systems(
                FixedUpdate,
                (enqueue_bump.before(bridge_bumped), bridge_bumped),
            );
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn bumped_bound_effects() -> BoundEffects {
        BoundEffects(vec![(
            "test".into(),
            EffectNode::When {
                trigger: Trigger::Bumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
        )])
    }

    #[test]
    fn bridge_bumped_fires_on_bolt_entity_only() {
        let mut app = test_app();

        // Bolt entity — targeted by msg.bolt
        let bolt_entity = app
            .world_mut()
            .spawn((
                bumped_bound_effects(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        // Second entity — should NOT be evaluated (targeted, not global)
        app.world_mut().spawn((
            bumped_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ));

        let breaker = app.world_mut().spawn_empty().id();
        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt_entity),
            breaker,
        })));

        tick(&mut app);

        let bolt_active = app.world().get::<ActiveSpeedBoosts>(bolt_entity).unwrap();
        assert_eq!(
            bolt_active.0.len(),
            1,
            "bridge_bumped should fire on the bolt entity from the message"
        );
        assert!(
            (bolt_active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );

        // Verify second entity was NOT affected
        let mut other_query = app.world_mut().query::<&ActiveSpeedBoosts>();
        let mut other_count = 0;
        for active in other_query.iter(app.world()) {
            if active.0.is_empty() {
                other_count += 1;
            }
        }
        assert_eq!(
            other_count, 1,
            "Second entity should not have any speed boosts (targeted bridge)"
        );
    }

    #[test]
    fn bridge_bumped_skips_when_bolt_is_none() {
        let mut app = test_app();

        app.world_mut().spawn((
            bumped_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ));

        let breaker = app.world_mut().spawn_empty().id();
        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: None,
            breaker,
        })));

        tick(&mut app);

        let active = app
            .world_mut()
            .query::<&ActiveSpeedBoosts>()
            .single(app.world())
            .unwrap();
        assert_eq!(
            active.0.len(),
            0,
            "bridge_bumped must skip when msg.bolt is None"
        );
    }

    #[test]
    fn bumped_context_resolves_to_specific_breaker() {
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

        let bolt = app
            .world_mut()
            .spawn((
                BoundEffects(vec![(
                    "ctx_test".into(),
                    EffectNode::When {
                        trigger: Trigger::Bumped,
                        then: vec![EffectNode::On {
                            target: Target::Breaker,
                            permanent: false,
                            then: vec![EffectNode::When {
                                trigger: Trigger::Died,
                                then: vec![EffectNode::Do(EffectKind::SpeedBoost {
                                    multiplier: 1.5,
                                })],
                            }],
                        }],
                    },
                )]),
                StagedEffects::default(),
            ))
            .id();

        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
            breaker: breaker_b,
        })));
        tick(&mut app);

        let staged_b = app.world().get::<StagedEffects>(breaker_b).unwrap();
        assert!(
            !staged_b.0.is_empty(),
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
}
