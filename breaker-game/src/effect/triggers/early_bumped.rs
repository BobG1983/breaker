//! Bridge system for the `early_bumped` trigger.
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
    shared::playing_state::PlayingState,
};

fn bridge_early_bumped(
    mut reader: MessageReader<BumpPerformed>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if msg.grade != BumpGrade::Early {
            continue;
        }
        let Some(bolt) = msg.bolt else { continue };
        if let Ok((entity, bound, mut staged)) = query.get_mut(bolt) {
            evaluate_bound_effects(
                &Trigger::EarlyBumped,
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(&Trigger::EarlyBumped, entity, &mut staged, &mut commands);
        }
    }
}

/// Register trigger bridge systems.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        bridge_early_bumped
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
                (
                    enqueue_bump.before(bridge_early_bumped),
                    bridge_early_bumped,
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
    fn bridge_early_bumped_fires_on_bolt_with_early_grade() {
        let mut app = test_app();

        let bolt_entity = app
            .world_mut()
            .spawn((
                BoundEffects(vec![(
                    "test".into(),
                    EffectNode::When {
                        trigger: Trigger::EarlyBumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                    },
                )]),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Early,
            bolt: Some(bolt_entity),
        })));

        tick(&mut app);

        let active = app.world().get::<ActiveSpeedBoosts>(bolt_entity).unwrap();
        assert_eq!(
            active.0.len(),
            1,
            "bridge_early_bumped should fire on bolt with Early grade"
        );
        assert!(
            (active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );
    }
}
