//! Bridge system for the `node_end` trigger.
//!
//! Reads `NodeCleared` messages and fires `Trigger::NodeEnd` globally
//! on all entities with `BoundEffects`.
use bevy::prelude::*;

use crate::{
    effect::{
        core::*,
        sets::EffectSystems,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    shared::PlayingState,
    state::run::node::messages::NodeCleared,
};

fn bridge_node_end(
    mut reader: MessageReader<NodeCleared>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for _msg in reader.read() {
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::NodeEnd,
                entity,
                bound,
                &mut staged,
                &mut commands,
                TriggerContext::default(),
            );
            evaluate_staged_effects(
                &Trigger::NodeEnd,
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
        bridge_node_end
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::effects::speed_boost::ActiveSpeedBoosts;

    #[derive(Resource)]
    struct TestNodeClearedMsg(bool);

    fn enqueue_node_cleared(
        msg_res: Res<TestNodeClearedMsg>,
        mut writer: MessageWriter<NodeCleared>,
    ) {
        if msg_res.0 {
            writer.write(NodeCleared);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<NodeCleared>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_node_cleared.before(bridge_node_end),
                    bridge_node_end,
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

    fn node_end_bound_effects() -> BoundEffects {
        BoundEffects(vec![(
            "test".into(),
            EffectNode::When {
                trigger: Trigger::NodeEnd,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
        )])
    }

    #[test]
    fn bridge_node_end_fires_globally_on_node_cleared() {
        let mut app = test_app();

        app.insert_resource(TestNodeClearedMsg(true));

        app.world_mut().spawn((
            node_end_bound_effects(),
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
            "bridge_node_end should fire NodeEnd globally when NodeCleared is sent"
        );
        assert!(
            (active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );
    }
}
