//! Bridge system for the `death` trigger.
//!
//! Reads `RequestCellDestroyed` and `RequestBoltDestroyed` messages and fires
//! `Trigger::Death` globally on all entities with `BoundEffects`.
use bevy::prelude::*;

use crate::{
    bolt::messages::RequestBoltDestroyed,
    cells::messages::RequestCellDestroyed,
    effect::{
        core::*,
        sets::EffectSystems,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    shared::PlayingState,
};

fn bridge_death(
    mut cell_reader: MessageReader<RequestCellDestroyed>,
    mut bolt_reader: MessageReader<RequestBoltDestroyed>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in cell_reader.read() {
        let context = TriggerContext {
            cell: Some(msg.cell),
            ..default()
        };
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Death,
                entity,
                bound,
                &mut staged,
                &mut commands,
                context,
            );
            evaluate_staged_effects(&Trigger::Death, entity, &mut staged, &mut commands, context);
        }
    }
    for msg in bolt_reader.read() {
        let context = TriggerContext {
            bolt: Some(msg.bolt),
            ..default()
        };
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Death,
                entity,
                bound,
                &mut staged,
                &mut commands,
                context,
            );
            evaluate_staged_effects(&Trigger::Death, entity, &mut staged, &mut commands, context);
        }
    }
}

/// Register trigger bridge systems.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        bridge_death
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(PlayingState::Active)),
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
                    enqueue_cell_destroyed.before(bridge_death),
                    enqueue_bolt_destroyed.before(bridge_death),
                    bridge_death,
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

    fn death_bound_effects() -> BoundEffects {
        BoundEffects(vec![(
            "test".into(),
            EffectNode::When {
                trigger: Trigger::Death,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
        )])
    }

    #[test]
    fn bridge_death_fires_globally_on_request_cell_destroyed() {
        let mut app = test_app();

        let cell = app.world_mut().spawn_empty().id();

        app.insert_resource(TestCellDestroyedMsg(Some(RequestCellDestroyed {
            cell,
            was_required_to_clear: true,
        })));
        app.insert_resource(TestBoltDestroyedMsg(None));

        app.world_mut().spawn((
            death_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ));

        tick(&mut app);

        let active = app
            .world_mut()
            .query::<&ActiveSpeedBoosts>()
            .iter(app.world())
            .find(|a| !a.0.is_empty());
        assert!(
            active.is_some(),
            "bridge_death should fire Death globally when RequestCellDestroyed is sent"
        );
        let active = active.unwrap();
        assert_eq!(active.0.len(), 1, "Exactly one SpeedBoost should be fired");
        assert!(
            (active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );
    }

    // ── Context entity: cell death passes dying cell as context ──

    #[test]
    fn bridge_death_cell_context_resolves_to_specific_cell() {
        use crate::cells::components::Cell;

        let mut app = test_app();

        let cell_a = app.world_mut().spawn((Cell, StagedEffects::default())).id();
        let cell_b = app.world_mut().spawn((Cell, StagedEffects::default())).id();
        let cell_c = app.world_mut().spawn((Cell, StagedEffects::default())).id();

        // Observer: When(Death, [On(Cell, [When(Died, [Do(SpeedBoost)])])])
        // On(Cell) should resolve to the specific dying cell via context
        app.world_mut().spawn((
            BoundEffects(vec![(
                "death_test".into(),
                EffectNode::When {
                    trigger: Trigger::Death,
                    then: vec![EffectNode::On {
                        target: Target::Cell,
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

        app.insert_resource(TestCellDestroyedMsg(Some(RequestCellDestroyed {
            cell: cell_b,
            was_required_to_clear: true,
        })));
        app.insert_resource(TestBoltDestroyedMsg(None));

        tick(&mut app);

        let staged_b = app.world().get::<StagedEffects>(cell_b).unwrap();
        assert!(
            !staged_b.0.is_empty(),
            "cell_b SHOULD have staged effects — it was the dying cell"
        );
        let staged_a = app.world().get::<StagedEffects>(cell_a).unwrap();
        let staged_c = app.world().get::<StagedEffects>(cell_c).unwrap();
        assert!(
            staged_a.0.is_empty(),
            "cell_a should have no staged effects — not the dying cell"
        );
        assert!(
            staged_c.0.is_empty(),
            "cell_c should have no staged effects — not the dying cell"
        );
    }

    // ── Context entity: bolt death passes dying bolt as context ──

    #[test]
    fn bridge_death_bolt_context_resolves_to_specific_bolt() {
        use crate::bolt::components::Bolt;

        let mut app = test_app();

        let bolt_a = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
        let bolt_b = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
        let bolt_c = app.world_mut().spawn((Bolt, StagedEffects::default())).id();

        app.world_mut().spawn((
            BoundEffects(vec![(
                "death_test".into(),
                EffectNode::When {
                    trigger: Trigger::Death,
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

        app.insert_resource(TestCellDestroyedMsg(None));
        app.insert_resource(TestBoltDestroyedMsg(Some(RequestBoltDestroyed {
            bolt: bolt_b,
        })));

        tick(&mut app);

        let staged_b = app.world().get::<StagedEffects>(bolt_b).unwrap();
        assert!(
            !staged_b.0.is_empty(),
            "bolt_b SHOULD have staged effects — it was the dying bolt"
        );
        let staged_a = app.world().get::<StagedEffects>(bolt_a).unwrap();
        let staged_c = app.world().get::<StagedEffects>(bolt_c).unwrap();
        assert!(
            staged_a.0.is_empty(),
            "bolt_a should have no staged effects — not the dying bolt"
        );
        assert!(
            staged_c.0.is_empty(),
            "bolt_c should have no staged effects — not the dying bolt"
        );
    }

    #[test]
    fn bridge_death_fires_globally_on_request_bolt_destroyed() {
        let mut app = test_app();

        let bolt = app.world_mut().spawn_empty().id();

        app.insert_resource(TestCellDestroyedMsg(None));
        app.insert_resource(TestBoltDestroyedMsg(Some(RequestBoltDestroyed { bolt })));

        app.world_mut().spawn((
            death_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ));

        tick(&mut app);

        let active = app
            .world_mut()
            .query::<&ActiveSpeedBoosts>()
            .iter(app.world())
            .find(|a| !a.0.is_empty());
        assert!(
            active.is_some(),
            "bridge_death should fire Death globally when RequestBoltDestroyed is sent"
        );
        let active = active.unwrap();
        assert_eq!(active.0.len(), 1, "Exactly one SpeedBoost should be fired");
        assert!(
            (active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );
    }
}
