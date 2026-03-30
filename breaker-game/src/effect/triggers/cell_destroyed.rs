//! Bridge system for the `cell_destroyed` trigger.
//!
//! Reads `CellDestroyedAt` messages and fires `Trigger::CellDestroyed` globally
//! on all entities with `BoundEffects`.
use bevy::prelude::*;

use crate::{
    cells::messages::CellDestroyedAt,
    effect::{
        core::*,
        sets::EffectSystems,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    shared::playing_state::PlayingState,
};

fn bridge_cell_destroyed(
    mut reader: MessageReader<CellDestroyedAt>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for _msg in reader.read() {
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::CellDestroyed,
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(&Trigger::CellDestroyed, entity, &mut staged, &mut commands);
        }
    }
}

/// Register trigger bridge systems.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        bridge_cell_destroyed
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::effects::speed_boost::ActiveSpeedBoosts;

    #[derive(Resource)]
    struct TestCellDestroyedMsg(Option<CellDestroyedAt>);

    fn enqueue_cell_destroyed(
        msg_res: Res<TestCellDestroyedMsg>,
        mut writer: MessageWriter<CellDestroyedAt>,
    ) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellDestroyedAt>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_cell_destroyed.before(bridge_cell_destroyed),
                    bridge_cell_destroyed,
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
    fn bridge_cell_destroyed_fires_globally_on_cell_destroyed_at() {
        let mut app = test_app();

        app.insert_resource(TestCellDestroyedMsg(Some(CellDestroyedAt {
            was_required_to_clear: true,
        })));

        app.world_mut().spawn((
            BoundEffects(vec![(
                "test".into(),
                EffectNode::When {
                    trigger: Trigger::CellDestroyed,
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
            "bridge_cell_destroyed should fire CellDestroyed globally on CellDestroyedAt"
        );
        assert!(
            (active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );
    }

    #[test]
    fn bridge_cell_destroyed_no_message_no_fire() {
        let mut app = test_app();

        app.insert_resource(TestCellDestroyedMsg(None));

        app.world_mut().spawn((
            BoundEffects(vec![(
                "test".into(),
                EffectNode::When {
                    trigger: Trigger::CellDestroyed,
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
            "No CellDestroyedAt message means no CellDestroyed trigger should fire"
        );
    }

    #[test]
    fn bridge_cell_destroyed_fires_on_all_entities_with_bound_effects() {
        let mut app = test_app();

        app.insert_resource(TestCellDestroyedMsg(Some(CellDestroyedAt {
            was_required_to_clear: false,
        })));

        // Two entities both listening for CellDestroyed — both should fire
        app.world_mut().spawn((
            BoundEffects(vec![(
                "test_a".into(),
                EffectNode::When {
                    trigger: Trigger::CellDestroyed,
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
                    trigger: Trigger::CellDestroyed,
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
