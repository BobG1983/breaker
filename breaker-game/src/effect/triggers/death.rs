//! Bridge systems for death triggers — evaluates `Trigger::Death` on the dying
//! entity's OWN `EffectChains`. Also writes `CellDestroyedAt` / `BoltDestroyedAt`
//! messages.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    bolt::messages::{BoltDestroyedAt, RequestBoltDestroyed},
    cells::{
        components::RequiredToClear,
        messages::{CellDestroyedAt, RequestCellDestroyed},
    },
    effect::{
        definition::{EffectChains, EffectNode, EffectTarget, Trigger},
        evaluate::evaluate_node,
        typed_events::fire_typed_event,
    },
};

/// Evaluates `Trigger::Death` on an entity's `EffectChains` and fires
/// any matching leaf effects.
fn evaluate_ondeath_chains(entity: Entity, chains: Option<&EffectChains>, commands: &mut Commands) {
    if let Some(chains) = chains {
        let targets = vec![EffectTarget::Entity(entity)];
        for (chip_name, node) in &chains.0 {
            if let Some(children) = evaluate_node(Trigger::Death, node) {
                for child in children {
                    if let EffectNode::Do(effect) = child {
                        fire_typed_event(
                            effect.clone(),
                            targets.clone(),
                            chip_name.clone(),
                            commands,
                        );
                    }
                }
            }
        }
    }
}

/// Bridge for `RequestCellDestroyed` — evaluates the dying cell's OWN
/// `EffectChains` with `Trigger::Death`, then writes `CellDestroyedAt`.
pub(crate) fn bridge_cell_death(
    mut reader: MessageReader<RequestCellDestroyed>,
    cell_query: Query<(Option<&EffectChains>, &Position2D, Has<RequiredToClear>)>,
    mut destroyed_writer: MessageWriter<CellDestroyedAt>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let Ok((chains, position, is_required)) = cell_query.get(msg.cell) else {
            continue;
        };

        evaluate_ondeath_chains(msg.cell, chains, &mut commands);

        destroyed_writer.write(CellDestroyedAt {
            position: position.0,
            was_required_to_clear: is_required,
        });
    }
}

/// Bridge for `RequestBoltDestroyed` — evaluates the dying bolt's OWN
/// `EffectChains` with `Trigger::Death`, then writes `BoltDestroyedAt`.
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

/// Registers bridge systems for death triggers.
pub(crate) fn register(app: &mut App) {
    use crate::{effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        (bridge_cell_death, bridge_bolt_death)
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::{super::test_helpers::*, *};
    use crate::{
        cells::components::Cell,
        effect::{
            definition::{Effect, EffectNode, Trigger},
            typed_events::*,
        },
    };

    // --- Cell death bridge tests ---

    /// Cell with `When(Death)` fires on `RequestCellDestroyed`.
    #[test]
    fn bridge_cell_death_evaluates_dying_cell_chains() {
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
            .add_message::<CellDestroyedAt>()
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_req, bridge_cell_death).chain());

        let cell = app
            .world_mut()
            .spawn((
                Cell,
                RequiredToClear,
                Position2D(Vec2::new(100.0, 200.0)),
                EffectChains(vec![(
                    None,
                    EffectNode::trigger_leaf(Trigger::Death, Effect::test_shockwave(48.0)),
                )]),
            ))
            .id();

        app.insert_resource(SendReq(Some(RequestCellDestroyed { cell })));
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "cell's Death EffectChains should fire — got {}",
            captured.0.len()
        );
        assert!((captured.0[0].base_range - 48.0).abs() < f32::EPSILON);
    }

    /// `CellDestroyedAt` is written with position and `required_to_clear`.
    #[test]
    fn bridge_cell_death_writes_cell_destroyed_at() {
        #[derive(Resource)]
        struct SendReq(Option<RequestCellDestroyed>);

        fn send_req(msg: Res<SendReq>, mut writer: MessageWriter<RequestCellDestroyed>) {
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

        let cell = app
            .world_mut()
            .spawn((Cell, RequiredToClear, Position2D(Vec2::new(100.0, 200.0))))
            .id();

        app.insert_resource(SendReq(Some(RequestCellDestroyed { cell })));
        tick(&mut app);

        let captured = app.world().resource::<CapturedCDA>();
        assert_eq!(
            captured.0.len(),
            1,
            "CellDestroyedAt should be written — got {}",
            captured.0.len()
        );
        assert_eq!(captured.0[0].position, Vec2::new(100.0, 200.0));
        assert!(captured.0[0].was_required_to_clear);
    }

    // --- M7: Death bridge is entity-local, not global sweep ---

    /// M7: Breaker entity with When(Death) chains does NOT fire when a cell is
    /// destroyed — Death evaluates ONLY the dying entity's chains, not globally.
    /// CellDestroyedAt is still written.
    #[test]
    fn bridge_cell_death_does_not_evaluate_other_entity_chains() {
        #[derive(Resource)]
        struct SendReqM7(Option<RequestCellDestroyed>);

        fn send_req_m7(msg: Res<SendReqM7>, mut writer: MessageWriter<RequestCellDestroyed>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        #[derive(Resource, Default)]
        struct CapturedCDAM7(Vec<CellDestroyedAt>);

        fn capture_cda_m7(
            mut reader: MessageReader<CellDestroyedAt>,
            mut captured: ResMut<CapturedCDAM7>,
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
            .init_resource::<CapturedCDAM7>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_req_m7, bridge_cell_death, capture_cda_m7).chain(),
            );

        // Breaker entity with When(Death) chain — should NOT be evaluated
        use crate::breaker::components::Breaker;
        app.world_mut().spawn((
            Breaker,
            EffectChains(vec![(
                None,
                EffectNode::trigger_leaf(Trigger::Death, Effect::test_shockwave(64.0)),
            )]),
        ));

        // Cell entity (no EffectChains) with Position2D and RequiredToClear
        let cell = app
            .world_mut()
            .spawn((Cell, RequiredToClear, Position2D(Vec2::new(50.0, 100.0))))
            .id();

        app.insert_resource(SendReqM7(Some(RequestCellDestroyed { cell })));
        tick(&mut app);

        // Breaker's Death chains should NOT fire
        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "Death bridge should only evaluate dying entity's chains, not globally. Got {}",
            captured.0.len()
        );

        // CellDestroyedAt should still be written
        let cda = app.world().resource::<CapturedCDAM7>();
        assert_eq!(
            cda.0.len(),
            1,
            "CellDestroyedAt should still be written even when dying entity has no chains"
        );
        assert_eq!(cda.0[0].position, Vec2::new(50.0, 100.0));
    }

    // --- Bolt death bridge tests ---

    /// Bolt with `When(Death)` fires on `RequestBoltDestroyed`.
    #[test]
    fn bridge_bolt_death_evaluates_dying_bolt_chains() {
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
            .add_message::<BoltDestroyedAt>()
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_req, bridge_bolt_death).chain());

        let bolt = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(50.0, -100.0)),
                EffectChains(vec![(
                    None,
                    EffectNode::trigger_leaf(Trigger::Death, Effect::test_shockwave(32.0)),
                )]),
            ))
            .id();

        app.insert_resource(SendReq(Some(RequestBoltDestroyed { bolt })));
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bolt's Death EffectChains should fire — got {}",
            captured.0.len()
        );
        assert!((captured.0[0].base_range - 32.0).abs() < f32::EPSILON);
    }

    /// `BoltDestroyedAt` is written with position.
    #[test]
    fn bridge_bolt_death_writes_bolt_destroyed_at() {
        #[derive(Resource)]
        struct SendReq(Option<RequestBoltDestroyed>);

        fn send_req(msg: Res<SendReq>, mut writer: MessageWriter<RequestBoltDestroyed>) {
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
            .init_resource::<CapturedBDA>()
            .add_systems(
                FixedUpdate,
                (send_req, bridge_bolt_death, capture_bda).chain(),
            );

        let bolt = app
            .world_mut()
            .spawn(Position2D(Vec2::new(50.0, -100.0)))
            .id();

        app.insert_resource(SendReq(Some(RequestBoltDestroyed { bolt })));
        tick(&mut app);

        let captured = app.world().resource::<CapturedBDA>();
        assert_eq!(
            captured.0.len(),
            1,
            "BoltDestroyedAt should be written — got {}",
            captured.0.len()
        );
        assert_eq!(captured.0[0].position, Vec2::new(50.0, -100.0));
    }
}
