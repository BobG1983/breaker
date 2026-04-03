//! Bridge system for the `late_bump` trigger.
use bevy::prelude::*;

use crate::{
    breaker::{
        messages::{BumpGrade, BumpPerformed},
        sets::BreakerSystems,
    },
    effect::{
        core::*,
        sets::EffectSystems,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    shared::PlayingState,
};

fn bridge_late_bump(
    mut reader: MessageReader<BumpPerformed>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Late {
            continue;
        }
        let context = TriggerContext {
            bolt: msg.bolt,
            breaker: Some(msg.breaker),
            ..default()
        };
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::LateBump,
                entity,
                bound,
                &mut staged,
                &mut commands,
                context,
            );
            evaluate_staged_effects(
                &Trigger::LateBump,
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
        bridge_late_bump
            .in_set(EffectSystems::Bridge)
            .after(BreakerSystems::GradeBump)
            .run_if(in_state(PlayingState::Active)),
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
                (enqueue_bump.before(bridge_late_bump), bridge_late_bump),
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

    #[test]
    fn bridge_late_bump_fires_on_late_grade() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn_empty().id();
        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Late,
            bolt: None,
            breaker,
        })));
        app.world_mut().spawn((
            BoundEffects(vec![(
                "test".into(),
                EffectNode::When {
                    trigger: Trigger::LateBump,
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
            "bridge_late_bump should fire on Late grade"
        );
        assert!(
            (active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );
    }

    #[test]
    fn late_bump_context_resolves_to_specific_bolt() {
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
                    trigger: Trigger::LateBump,
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
            grade: BumpGrade::Late,
            bolt: Some(bolt_b),
            breaker,
        })));
        tick(&mut app);

        let staged_b = app.world().get::<StagedEffects>(bolt_b).unwrap();
        assert!(!staged_b.0.is_empty(), "bolt_b SHOULD have staged effects");
        assert!(
            app.world()
                .get::<StagedEffects>(bolt_a)
                .unwrap()
                .0
                .is_empty(),
            "bolt_a should be empty"
        );
        assert!(
            app.world()
                .get::<StagedEffects>(bolt_c)
                .unwrap()
                .0
                .is_empty(),
            "bolt_c should be empty"
        );
    }
}
