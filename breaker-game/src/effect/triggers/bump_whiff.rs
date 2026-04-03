//! Bridge system for the `bump_whiff` trigger.
use bevy::prelude::*;

use crate::{
    breaker::{messages::BumpWhiffed, sets::BreakerSystems},
    effect::{
        core::*,
        sets::EffectSystems,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    shared::PlayingState,
};

fn bridge_bump_whiff(
    mut reader: MessageReader<BumpWhiffed>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for _msg in reader.read() {
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::BumpWhiff,
                entity,
                bound,
                &mut staged,
                &mut commands,
                TriggerContext::default(),
            );
            evaluate_staged_effects(
                &Trigger::BumpWhiff,
                entity,
                &mut staged,
                &mut commands,
                TriggerContext::default(),
            );
        }
    }
}

/// Register trigger bridge systems.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        bridge_bump_whiff
            .in_set(EffectSystems::Bridge)
            .after(BreakerSystems::GradeBump)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::effects::speed_boost::ActiveSpeedBoosts;

    #[derive(Resource)]
    struct TestWhiffMsg(bool);

    fn enqueue_whiff(msg_res: Res<TestWhiffMsg>, mut writer: MessageWriter<BumpWhiffed>) {
        if msg_res.0 {
            writer.write(BumpWhiffed);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpWhiffed>()
            .add_systems(
                FixedUpdate,
                (enqueue_whiff.before(bridge_bump_whiff), bridge_bump_whiff),
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
    fn bridge_bump_whiff_fires_on_whiff() {
        let mut app = test_app();
        app.insert_resource(TestWhiffMsg(true));
        app.world_mut().spawn((
            BoundEffects(vec![(
                "test".into(),
                EffectNode::When {
                    trigger: Trigger::BumpWhiff,
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
            "bridge_bump_whiff should fire on BumpWhiffed"
        );
        assert!(
            (active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );
    }
}
