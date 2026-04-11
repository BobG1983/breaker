//! Bridge system for the `died` trigger.
//!
//! Reads `RequestCellDestroyed` and `RequestBoltDestroyed` messages and fires
//! `Trigger::Died` only on the dying entity (targeted, not global).
use bevy::prelude::*;

use crate::{
    effect::{
        core::*,
        sets::EffectSystems,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    prelude::*,
};

fn bridge_died(
    mut cell_reader: MessageReader<RequestCellDestroyed>,
    mut bolt_reader: MessageReader<RequestBoltDestroyed>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in cell_reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.cell) {
            evaluate_bound_effects(
                &Trigger::Died,
                entity,
                bound,
                &mut staged,
                &mut commands,
                TriggerContext::default(),
            );
            evaluate_staged_effects(
                &Trigger::Died,
                entity,
                &mut staged,
                &mut commands,
                TriggerContext::default(),
            );
        }
    }
    for msg in bolt_reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.bolt) {
            evaluate_bound_effects(
                &Trigger::Died,
                entity,
                bound,
                &mut staged,
                &mut commands,
                TriggerContext::default(),
            );
            evaluate_staged_effects(
                &Trigger::Died,
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
        bridge_died
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(NodeState::Playing)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::effects::speed_boost::ActiveSpeedBoosts;

    // -- RequestCellDestroyed helper --

    #[derive(Resource)]
    struct TestCellDestroyedMsg(Option<RequestCellDestroyed>);

    fn enqueue_cell_destroyed(
        msg_res: Res<TestCellDestroyedMsg>,
        mut writer: MessageWriter<RequestCellDestroyed>,
    ) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    // -- RequestBoltDestroyed helper --

    #[derive(Resource)]
    struct TestBoltDestroyedMsg(Option<RequestBoltDestroyed>);

    fn enqueue_bolt_destroyed(
        msg_res: Res<TestBoltDestroyedMsg>,
        mut writer: MessageWriter<RequestBoltDestroyed>,
    ) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestCellDestroyed>()
            .add_message::<RequestBoltDestroyed>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_cell_destroyed.before(bridge_died),
                    enqueue_bolt_destroyed.before(bridge_died),
                    bridge_died,
                ),
            );
        app
    }

    use crate::shared::test_utils::tick;

    fn died_bound_effects() -> BoundEffects {
        BoundEffects(vec![(
            "test".into(),
            EffectNode::When {
                trigger: Trigger::Died,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
        )])
    }

    #[test]
    fn bridge_died_fires_on_dying_cell_entity_only() {
        let mut app = test_app();

        // The cell entity that will die — has Died chain, should fire
        let dying_cell = app
            .world_mut()
            .spawn((
                died_bound_effects(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        // Second entity — also has Died chain, but is NOT the dying entity
        app.world_mut().spawn((
            died_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ));

        app.insert_resource(TestCellDestroyedMsg(Some(RequestCellDestroyed {
            cell: dying_cell,
            was_required_to_clear: true,
        })));
        app.insert_resource(TestBoltDestroyedMsg(None));

        tick(&mut app);

        // The dying cell's effects should fire
        let dying_active = app.world().get::<ActiveSpeedBoosts>(dying_cell).unwrap();
        assert_eq!(
            dying_active.0.len(),
            1,
            "bridge_died should fire Died on the dying cell entity from RequestCellDestroyed"
        );
        assert!(
            (dying_active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );

        // Count entities with non-empty ActiveSpeedBoosts — only the dying one
        let mut affected_count = 0;
        for active in app
            .world_mut()
            .query::<&ActiveSpeedBoosts>()
            .iter(app.world())
        {
            if !active.0.is_empty() {
                affected_count += 1;
            }
        }
        assert_eq!(
            affected_count, 1,
            "Only the dying entity should be affected (targeted, not global)"
        );
    }

    #[test]
    fn bridge_died_fires_on_dying_bolt_entity() {
        let mut app = test_app();

        // The bolt entity that will die
        let dying_bolt = app
            .world_mut()
            .spawn((
                died_bound_effects(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        // Second entity — should NOT fire
        app.world_mut().spawn((
            died_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ));

        app.insert_resource(TestCellDestroyedMsg(None));
        app.insert_resource(TestBoltDestroyedMsg(Some(RequestBoltDestroyed {
            bolt: dying_bolt,
        })));

        tick(&mut app);

        let dying_active = app.world().get::<ActiveSpeedBoosts>(dying_bolt).unwrap();
        assert_eq!(
            dying_active.0.len(),
            1,
            "bridge_died should fire Died on the dying bolt entity from RequestBoltDestroyed"
        );
        assert!(
            (dying_active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );

        // Only the dying entity should be affected
        let mut affected_count = 0;
        for active in app
            .world_mut()
            .query::<&ActiveSpeedBoosts>()
            .iter(app.world())
        {
            if !active.0.is_empty() {
                affected_count += 1;
            }
        }
        assert_eq!(
            affected_count, 1,
            "Only the dying bolt entity should be affected (targeted, not global)"
        );
    }
}
