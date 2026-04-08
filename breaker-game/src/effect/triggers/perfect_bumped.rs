//! Bridge system for the `perfect_bumped` trigger.
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
    state::types::NodeState,
};

fn bridge_perfect_bumped(
    mut reader: MessageReader<BumpPerformed>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Perfect {
            continue;
        }
        let Some(bolt) = msg.bolt else { continue };
        if let Ok((entity, bound, mut staged)) = query.get_mut(bolt) {
            let context = TriggerContext {
                breaker: Some(msg.breaker),
                ..default()
            };
            evaluate_bound_effects(
                &Trigger::PerfectBumped,
                entity,
                bound,
                &mut staged,
                &mut commands,
                context,
            );
            evaluate_staged_effects(
                &Trigger::PerfectBumped,
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
        bridge_perfect_bumped
            .in_set(EffectSystems::Bridge)
            .after(BreakerSystems::GradeBump)
            .run_if(in_state(NodeState::Playing)),
    );
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::CommandQueue;

    use super::*;
    use crate::{breaker::messages::BumpGrade, effect::effects::speed_boost::ActiveSpeedBoosts};

    fn spawn_in_world(world: &mut World, f: impl FnOnce(&mut Commands) -> Entity) -> Entity {
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            f(&mut commands)
        };
        queue.apply(world);
        entity
    }

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
                (
                    enqueue_bump.before(bridge_perfect_bumped),
                    bridge_perfect_bumped,
                ),
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
    fn bridge_perfect_bumped_fires_on_bolt_with_perfect_grade() {
        let mut app = test_app();

        let bolt_entity = app
            .world_mut()
            .spawn((
                BoundEffects(vec![(
                    "test".into(),
                    EffectNode::When {
                        trigger: Trigger::PerfectBumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                    },
                )]),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        let breaker = app.world_mut().spawn_empty().id();
        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt_entity),
            breaker,
        })));

        tick(&mut app);

        let active = app.world().get::<ActiveSpeedBoosts>(bolt_entity).unwrap();
        assert_eq!(
            active.0.len(),
            1,
            "bridge_perfect_bumped should fire on bolt with Perfect grade"
        );
        assert!(
            (active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );
    }

    #[test]
    fn perfect_bumped_context_resolves_to_specific_breaker() {
        use crate::breaker::components::Breaker;

        let mut app = test_app();

        let def = crate::breaker::definition::BreakerDefinition::default();
        let breaker_a = spawn_in_world(app.world_mut(), |commands| {
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .spawn(commands)
        });
        app.world_mut()
            .entity_mut(breaker_a)
            .insert(StagedEffects::default());
        let breaker_b = spawn_in_world(app.world_mut(), |commands| {
            Breaker::builder()
                .definition(&def)
                .headless()
                .extra()
                .spawn(commands)
        });
        app.world_mut()
            .entity_mut(breaker_b)
            .insert(StagedEffects::default());

        let bolt = app
            .world_mut()
            .spawn((
                BoundEffects(vec![(
                    "ctx_test".into(),
                    EffectNode::When {
                        trigger: Trigger::PerfectBumped,
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
