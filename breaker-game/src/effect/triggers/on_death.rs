//! Bridge systems for entity death — cell and bolt destruction, cleanup, and Once evaluation.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    bolt::messages::{BoltDestroyedAt, RequestBoltDestroyed},
    cells::{
        components::RequiredToClear,
        messages::{CellDestroyedAt, RequestCellDestroyed},
    },
    effect::{
        armed::ArmedEffects,
        definition::{EffectChains, EffectNode, Trigger},
        evaluate::{NodeEvalResult, evaluate_node},
        helpers::evaluate_armed_all,
        typed_events::fire_typed_event,
    },
};

/// Evaluates `Trigger::Death` on an entity's `EffectChains` and fires
/// any matching leaf effects. Shared by `bridge_cell_death` and `bridge_bolt_death`.
fn evaluate_ondeath_chains(entity: Entity, chains: Option<&EffectChains>, commands: &mut Commands) {
    if let Some(chains) = chains {
        let targets = vec![crate::effect::definition::EffectTarget::Entity(entity)];
        for (chip_name, node) in &chains.0 {
            for result in evaluate_node(Trigger::Death, node) {
                if let NodeEvalResult::Fire(effect) = result {
                    fire_typed_event(effect, targets.clone(), chip_name.clone(), commands);
                }
            }
        }
    }
}

/// Bridge for `RequestCellDestroyed` — evaluates cell's `EffectChains` with
/// `Trigger::Death` while the entity is still alive, then writes
/// `CellDestroyedAt` with position and required-to-clear data.
///
/// Also evaluates `Trigger::CellDestroyed` active chains and armed triggers.
pub(crate) fn bridge_cell_death(
    mut reader: MessageReader<RequestCellDestroyed>,
    cell_query: Query<(Option<&EffectChains>, &Position2D, Has<RequiredToClear>)>,
    armed_query: Query<(Entity, &mut ArmedEffects)>,
    mut destroyed_writer: MessageWriter<CellDestroyedAt>,
    mut commands: Commands,
) {
    let mut any_destroyed = false;
    for msg in reader.read() {
        let Ok((chains, position, is_required)) = cell_query.get(msg.cell) else {
            continue;
        };

        any_destroyed = true;
        evaluate_ondeath_chains(msg.cell, chains, &mut commands);

        destroyed_writer.write(CellDestroyedAt {
            position: position.0,
            was_required_to_clear: is_required,
        });
    }

    if any_destroyed {
        evaluate_armed_all(armed_query, Trigger::CellDestroyed, &mut commands);
    }
}

/// Bridge for `RequestBoltDestroyed` — evaluates bolt's `EffectChains` with
/// `Trigger::Death` while the entity is still alive, then writes
/// `BoltDestroyedAt` with position data.
pub(crate) fn bridge_bolt_death(
    mut reader: MessageReader<RequestBoltDestroyed>,
    bolt_query: Query<(Option<&EffectChains>, &Position2D)>,
    mut destroyed_writer: MessageWriter<BoltDestroyedAt>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let Ok((chains, position)) = bolt_query.get(msg.bolt) else {
            continue;
        };

        evaluate_ondeath_chains(msg.bolt, chains, &mut commands);

        destroyed_writer.write(BoltDestroyedAt {
            position: position.0,
        });
    }
}

/// Cleanup system for cells — despawns cell entities from
/// `RequestCellDestroyed` messages. Runs after all bridges.
pub(crate) fn cleanup_destroyed_cells(
    mut reader: MessageReader<RequestCellDestroyed>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        commands.entity(msg.cell).despawn();
    }
}

/// Cleanup system for bolts — despawns bolt entities from
/// `RequestBoltDestroyed` messages. Runs after all bridges.
pub(crate) fn cleanup_destroyed_bolts(
    mut reader: MessageReader<RequestBoltDestroyed>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        commands.entity(msg.bolt).despawn();
    }
}

/// Processes `Once` nodes wrapping bare `Do` children at chip selection time.
/// Fires the effect and removes the `Once` wrapper from `EffectChains`.
/// Once nodes wrapping `When` nodes are left for bridge evaluation.
pub(crate) fn apply_once_nodes(mut query: Query<&mut EffectChains>, mut commands: Commands) {
    for mut chains in &mut query {
        chains.0.retain(|(chip_name, node)| {
            if let EffectNode::Once(children) = node {
                // Check if all children are bare Do nodes
                let all_bare_do = children.iter().all(|c| matches!(c, EffectNode::Do(_)));
                if all_bare_do && !children.is_empty() {
                    // Fire all bare Do children
                    for child in children {
                        if let EffectNode::Do(effect) = child {
                            fire_typed_event(
                                effect.clone(),
                                vec![],
                                chip_name.clone(),
                                &mut commands,
                            );
                        }
                    }
                    return false; // Remove the Once node
                }
            }
            true // Keep non-Once nodes and Once nodes wrapping When
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cells::messages::RequestCellDestroyed,
        effect::{
            definition::{Effect, EffectNode, Trigger},
            typed_events::*,
        },
    };

    // --- Test infrastructure ---

    #[derive(Resource, Default)]
    struct CapturedShockwaveFired(Vec<ShockwaveFired>);

    fn capture_shockwave_fired(
        trigger: On<ShockwaveFired>,
        mut captured: ResMut<CapturedShockwaveFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource)]
    struct SendCellDestroyed(Option<RequestCellDestroyed>);

    fn send_cell_destroyed(
        msg: Res<SendCellDestroyed>,
        mut writer: MessageWriter<RequestCellDestroyed>,
    ) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn cell_destroyed_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestCellDestroyed>()
            .add_message::<CellDestroyedAt>()
            .insert_resource(SendCellDestroyed(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_cell_destroyed, bridge_cell_death).chain(),
            );
        app
    }

    // --- Cell destroyed bridge tests ---

    #[test]
    fn cell_destroyed_fires_armed_chain() {
        let chain = EffectNode::trigger_leaf(Trigger::CellDestroyed, Effect::test_shockwave(32.0));
        let mut app = cell_destroyed_test_app();
        // Spawn a bolt with armed CellDestroyed chain
        app.world_mut()
            .spawn(crate::effect::armed::ArmedEffects(vec![(None, chain)]));
        // Spawn a cell entity with Position2D for bridge_cell_death to query
        let cell = app
            .world_mut()
            .spawn((Position2D(Vec2::new(10.0, 20.0)), RequiredToClear))
            .id();
        app.world_mut().resource_mut::<SendCellDestroyed>().0 = Some(RequestCellDestroyed { cell });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_range - 32.0).abs() < f32::EPSILON);
    }

    #[test]
    fn cell_destroyed_no_message_no_fire() {
        let chain = EffectNode::trigger_leaf(Trigger::CellDestroyed, Effect::test_shockwave(32.0));
        let mut app = cell_destroyed_test_app();
        app.world_mut()
            .spawn(crate::effect::armed::ArmedEffects(vec![(None, chain)]));
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(captured.0.is_empty());
    }

    // --- Two-Phase Destruction — bridge_cell_death ---

    #[test]
    fn bridge_cell_death_evaluates_ondeath_effect_chains_and_writes_cell_destroyed_at() {
        use rantzsoft_spatial2d::components::Position2D;

        use crate::{
            cells::{
                components::{Cell, RequiredToClear},
                messages::{CellDestroyedAt, RequestCellDestroyed},
            },
            effect::definition::EffectChains,
        };

        #[derive(Resource)]
        struct SendRequestCellDestroyed(Option<RequestCellDestroyed>);

        fn send_request(
            msg: Res<SendRequestCellDestroyed>,
            mut writer: MessageWriter<RequestCellDestroyed>,
        ) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        #[derive(Resource, Default)]
        struct CapturedCellDestroyedAt(Vec<CellDestroyedAt>);

        fn capture_cell_destroyed_at(
            mut reader: MessageReader<CellDestroyedAt>,
            mut captured: ResMut<CapturedCellDestroyedAt>,
        ) {
            for msg in reader.read() {
                captured.0.push(msg.clone());
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestCellDestroyed>()
            .add_message::<CellDestroyedAt>()
            .init_resource::<CapturedShockwaveFired>()
            .init_resource::<CapturedCellDestroyedAt>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_request, bridge_cell_death, capture_cell_destroyed_at).chain(),
            );

        let cell = app
            .world_mut()
            .spawn((
                Cell,
                RequiredToClear,
                Position2D(Vec2::new(100.0, 200.0)),
                EffectChains(vec![(
                    None,
                    EffectNode::When {
                        trigger: Trigger::Death,
                        then: vec![EffectNode::Do(Effect::Shockwave {
                            base_range: 48.0,
                            range_per_level: 0.0,
                            stacks: 1,
                            speed: 400.0,
                        })],
                    },
                )]),
            ))
            .id();

        app.insert_resource(SendRequestCellDestroyed(Some(RequestCellDestroyed {
            cell,
        })));

        tick(&mut app);

        // Cell entity should still be alive (bridge doesn't despawn)
        assert!(
            app.world().get_entity(cell).is_ok(),
            "cell entity should still be alive after bridge_cell_death"
        );

        // Death EffectChains should have fired
        let shockwaves = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            shockwaves.0.len(),
            1,
            "bridge_cell_death should evaluate cell's Death EffectChains"
        );
        assert!((shockwaves.0[0].base_range - 48.0).abs() < f32::EPSILON);

        // CellDestroyedAt message should be written
        let destroyed = app.world().resource::<CapturedCellDestroyedAt>();
        assert_eq!(
            destroyed.0.len(),
            1,
            "bridge_cell_death should write CellDestroyedAt"
        );
        assert_eq!(destroyed.0[0].position, Vec2::new(100.0, 200.0));
        assert!(destroyed.0[0].was_required_to_clear);
    }

    #[test]
    fn bridge_cell_death_writes_cell_destroyed_at_even_without_effect_chains() {
        use rantzsoft_spatial2d::components::Position2D;

        use crate::cells::{
            components::{Cell, RequiredToClear},
            messages::{CellDestroyedAt, RequestCellDestroyed},
        };

        #[derive(Resource)]
        struct SendReqCellDest(Option<RequestCellDestroyed>);

        fn send_req(msg: Res<SendReqCellDest>, mut writer: MessageWriter<RequestCellDestroyed>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        #[derive(Resource, Default)]
        struct CapturedCDA(Vec<CellDestroyedAt>);

        fn capture_cda(
            mut reader: MessageReader<CellDestroyedAt>,
            mut captured: ResMut<CapturedCDA>,
        ) {
            for msg in reader.read() {
                captured.0.push(msg.clone());
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestCellDestroyed>()
            .add_message::<CellDestroyedAt>()
            .init_resource::<CapturedCDA>()
            .add_systems(
                FixedUpdate,
                (send_req, bridge_cell_death, capture_cda).chain(),
            );

        // Cell without EffectChains
        let cell = app
            .world_mut()
            .spawn((Cell, RequiredToClear, Position2D(Vec2::new(50.0, 75.0))))
            .id();

        app.insert_resource(SendReqCellDest(Some(RequestCellDestroyed { cell })));

        tick(&mut app);

        let captured = app.world().resource::<CapturedCDA>();
        assert_eq!(
            captured.0.len(),
            1,
            "CellDestroyedAt should be written even without EffectChains"
        );
        assert_eq!(captured.0[0].position, Vec2::new(50.0, 75.0));
        assert!(captured.0[0].was_required_to_clear);
    }

    // --- Two-Phase — bridge_bolt_death ---

    #[test]
    fn bridge_bolt_death_evaluates_ondeath_and_writes_bolt_destroyed_at() {
        use rantzsoft_spatial2d::components::Position2D;

        use crate::{
            bolt::messages::{BoltDestroyedAt, RequestBoltDestroyed},
            effect::definition::EffectChains,
        };

        #[derive(Resource)]
        struct SendReqBoltDest(Option<RequestBoltDestroyed>);

        fn send_req(msg: Res<SendReqBoltDest>, mut writer: MessageWriter<RequestBoltDestroyed>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        #[derive(Resource, Default)]
        struct CapturedBDA(Vec<BoltDestroyedAt>);

        fn capture_bda(
            mut reader: MessageReader<BoltDestroyedAt>,
            mut captured: ResMut<CapturedBDA>,
        ) {
            for msg in reader.read() {
                captured.0.push(msg.clone());
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestBoltDestroyed>()
            .add_message::<BoltDestroyedAt>()
            .init_resource::<CapturedShockwaveFired>()
            .init_resource::<CapturedBDA>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_req, bridge_bolt_death, capture_bda).chain(),
            );

        let bolt = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(50.0, -100.0)),
                EffectChains(vec![(
                    None,
                    EffectNode::When {
                        trigger: Trigger::Death,
                        then: vec![EffectNode::Do(Effect::Shockwave {
                            base_range: 32.0,
                            range_per_level: 0.0,
                            stacks: 1,
                            speed: 400.0,
                        })],
                    },
                )]),
            ))
            .id();

        app.insert_resource(SendReqBoltDest(Some(RequestBoltDestroyed { bolt })));

        tick(&mut app);

        // Bolt entity should still be alive
        assert!(
            app.world().get_entity(bolt).is_ok(),
            "bolt entity should still be alive after bridge_bolt_death"
        );

        let shockwaves = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            shockwaves.0.len(),
            1,
            "bridge_bolt_death should evaluate bolt's Death EffectChains"
        );

        let captured = app.world().resource::<CapturedBDA>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge_bolt_death should write BoltDestroyedAt"
        );
        assert_eq!(captured.0[0].position, Vec2::new(50.0, -100.0));
    }

    // --- Breaker stub types ---

    #[test]
    fn request_breaker_destroyed_and_breaker_destroyed_at_types_exist() {
        use crate::breaker::messages::{BreakerDestroyedAt, RequestBreakerDestroyed};

        let req = RequestBreakerDestroyed {
            breaker: Entity::PLACEHOLDER,
        };
        let dest = BreakerDestroyedAt {
            position: Vec2::new(0.0, -300.0),
        };
        // Types construct without error — Message derive is valid
        assert!(format!("{req:?}").contains("RequestBreakerDestroyed"));
        assert!(format!("{dest:?}").contains("BreakerDestroyedAt"));
    }

    // --- cleanup_destroyed_cells ---

    #[test]
    fn cleanup_destroyed_cells_despawns_cell_entity() {
        use crate::cells::messages::RequestCellDestroyed;

        #[derive(Resource)]
        struct SendReq(Option<RequestCellDestroyed>);

        fn send_req(msg: Res<SendReq>, mut writer: MessageWriter<RequestCellDestroyed>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestCellDestroyed>()
            .add_systems(FixedUpdate, (send_req, cleanup_destroyed_cells).chain());

        let cell = app.world_mut().spawn_empty().id();
        app.insert_resource(SendReq(Some(RequestCellDestroyed { cell })));

        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_err(),
            "cell entity should be despawned after cleanup_destroyed_cells"
        );
    }

    #[test]
    fn cleanup_destroyed_cells_no_panic_if_entity_already_despawned() {
        use crate::cells::messages::RequestCellDestroyed;

        #[derive(Resource)]
        struct SendReqTwice(Vec<RequestCellDestroyed>);

        fn send_req_twice(
            mut msgs: ResMut<SendReqTwice>,
            mut writer: MessageWriter<RequestCellDestroyed>,
        ) {
            for m in msgs.0.drain(..) {
                writer.write(m);
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestCellDestroyed>()
            .add_systems(
                FixedUpdate,
                (send_req_twice, cleanup_destroyed_cells).chain(),
            );

        let cell = app.world_mut().spawn_empty().id();
        // Send the same cell entity in two separate messages — second should not panic
        app.insert_resource(SendReqTwice(vec![
            RequestCellDestroyed { cell },
            RequestCellDestroyed { cell },
        ]));

        // Should not panic even if the entity is despawned by the first message
        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_err(),
            "cell entity should be despawned"
        );
    }

    // --- cleanup_destroyed_bolts ---

    #[test]
    fn cleanup_destroyed_bolts_despawns_bolt_entity() {
        use crate::bolt::messages::RequestBoltDestroyed;

        #[derive(Resource)]
        struct SendReq(Option<RequestBoltDestroyed>);

        fn send_req(msg: Res<SendReq>, mut writer: MessageWriter<RequestBoltDestroyed>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestBoltDestroyed>()
            .add_systems(FixedUpdate, (send_req, cleanup_destroyed_bolts).chain());

        let bolt = app.world_mut().spawn_empty().id();
        app.insert_resource(SendReq(Some(RequestBoltDestroyed { bolt })));

        tick(&mut app);

        assert!(
            app.world().get_entity(bolt).is_err(),
            "bolt entity should be despawned after cleanup_destroyed_bolts"
        );
    }

    // --- Once node evaluation ---

    #[test]
    fn apply_once_nodes_fires_bare_do_and_removes_once_wrapper() {
        use crate::effect::definition::EffectChains;

        #[derive(Resource, Default)]
        struct CapturedSpawnBoltsFired(Vec<SpawnBoltsFired>);

        fn capture_spawn_bolts(
            trigger: On<SpawnBoltsFired>,
            mut captured: ResMut<CapturedSpawnBoltsFired>,
        ) {
            captured.0.push(trigger.event().clone());
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CapturedSpawnBoltsFired>()
            .add_observer(capture_spawn_bolts)
            .add_systems(FixedUpdate, apply_once_nodes);

        let entity = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::Once(vec![EffectNode::Do(Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                })]),
            )]))
            .id();

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnBoltsFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bare Do inside Once should fire at chip selection time"
        );

        let chains = app.world().get::<EffectChains>(entity).unwrap();
        assert!(
            chains.0.is_empty(),
            "Once node should be removed from EffectChains after firing"
        );
    }

    #[test]
    fn once_already_consumed_does_not_fire_again() {
        use crate::effect::definition::EffectChains;

        #[derive(Resource, Default)]
        struct CapturedSpawnBoltsFired2(Vec<SpawnBoltsFired>);

        fn capture_spawn_bolts2(
            trigger: On<SpawnBoltsFired>,
            mut captured: ResMut<CapturedSpawnBoltsFired2>,
        ) {
            captured.0.push(trigger.event().clone());
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CapturedSpawnBoltsFired2>()
            .add_observer(capture_spawn_bolts2)
            .add_systems(FixedUpdate, apply_once_nodes);

        // Empty EffectChains — Once was already consumed
        app.world_mut().spawn(EffectChains::default());

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnBoltsFired2>();
        assert!(
            captured.0.is_empty(),
            "empty EffectChains should not fire anything"
        );
    }
}
