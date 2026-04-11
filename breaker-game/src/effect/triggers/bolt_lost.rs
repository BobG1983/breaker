//! Bridge system for the `bolt_lost` trigger.
//!
//! Reads `BoltLost` messages and fires `Trigger::BoltLost` globally
//! on all entities with `BoundEffects`.
use bevy::prelude::*;

use crate::{
    bolt::{messages::BoltLost, sets::BoltSystems},
    effect::{
        core::*,
        sets::EffectSystems,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    state::types::NodeState,
};

fn bridge_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for _msg in reader.read() {
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::BoltLost,
                entity,
                bound,
                &mut staged,
                &mut commands,
                TriggerContext::default(),
            );
            evaluate_staged_effects(
                &Trigger::BoltLost,
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
        bridge_bolt_lost
            .in_set(EffectSystems::Bridge)
            .after(BoltSystems::BoltLost)
            .run_if(in_state(NodeState::Playing)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::effects::speed_boost::ActiveSpeedBoosts;

    #[derive(Resource)]
    struct TestBoltLostMsg(bool);

    fn enqueue_bolt_lost(msg_res: Res<TestBoltLostMsg>, mut writer: MessageWriter<BoltLost>) {
        if msg_res.0 {
            writer.write(BoltLost);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .add_systems(
                FixedUpdate,
                (enqueue_bolt_lost.before(bridge_bolt_lost), bridge_bolt_lost),
            );
        app
    }

    use crate::shared::test_utils::tick;

    #[test]
    fn bridge_bolt_lost_fires_globally_on_bolt_lost() {
        let mut app = test_app();
        app.insert_resource(TestBoltLostMsg(true));

        app.world_mut().spawn((
            BoundEffects(vec![(
                "test".into(),
                EffectNode::When {
                    trigger: Trigger::BoltLost,
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
            "bridge_bolt_lost should fire BoltLost globally on BoltLost message"
        );
        assert!(
            (active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );
    }

    #[test]
    fn bridge_bolt_lost_no_message_no_fire() {
        let mut app = test_app();
        app.insert_resource(TestBoltLostMsg(false));

        app.world_mut().spawn((
            BoundEffects(vec![(
                "test".into(),
                EffectNode::When {
                    trigger: Trigger::BoltLost,
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
            0,
            "No BoltLost message means no BoltLost trigger should fire"
        );
    }

    #[test]
    fn bridge_bolt_lost_fires_on_all_entities_with_bound_effects() {
        let mut app = test_app();
        app.insert_resource(TestBoltLostMsg(true));

        // Two entities both listening for BoltLost — both should fire
        app.world_mut().spawn((
            BoundEffects(vec![(
                "test_a".into(),
                EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
            )]),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ));

        app.world_mut().spawn((
            BoundEffects(vec![(
                "test_b".into(),
                EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 2.0 })],
                },
            )]),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ));

        tick(&mut app);

        let mut total_boosts = 0;
        for active in app
            .world_mut()
            .query::<&ActiveSpeedBoosts>()
            .iter(app.world())
        {
            total_boosts += active.0.len();
        }
        assert_eq!(
            total_boosts, 2,
            "Global trigger should fire on ALL entities with matching BoundEffects"
        );
    }
}
