//! Bridge for `Trigger::DestroyedCell` — evaluates the dying cell's OWN
//! `EffectChains` for the `DestroyedCell` trigger (no bolt source for now).

use bevy::prelude::*;

use crate::{
    cells::messages::RequestCellDestroyed,
    effect::{
        definition::{EffectChains, EffectTarget, Trigger},
        helpers::evaluate_entity_chains,
    },
};

/// Bridge for `RequestCellDestroyed` — evaluates `Trigger::DestroyedCell` on
/// the dying cell's OWN `EffectChains` only.
pub(crate) fn bridge_destroyed_cell(
    mut reader: MessageReader<RequestCellDestroyed>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    let trigger_kind = Trigger::DestroyedCell;
    for msg in reader.read() {
        let targets = vec![EffectTarget::Entity(msg.cell)];
        if let Ok(mut chains) = chains_query.get_mut(msg.cell) {
            evaluate_entity_chains(&mut chains, trigger_kind, targets, &mut commands);
        }
    }
}

/// Registers bridge systems for destroyed cell trigger.
pub(crate) fn register(app: &mut App) {
    use crate::{effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        bridge_destroyed_cell
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::{super::test_helpers::*, *};
    use crate::effect::definition::{Effect, EffectNode, Trigger};

    // --- DestroyedCell bridge tests ---

    /// Cell with `When(DestroyedCell)` fires when that cell is destroyed.
    #[test]
    fn bridge_destroyed_cell_evaluates_dying_cell() {
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
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_req, bridge_destroyed_cell).chain());

        // Cell with DestroyedCell chain
        let cell = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::trigger_leaf(Trigger::DestroyedCell, Effect::test_shockwave(64.0)),
            )]))
            .id();

        app.insert_resource(SendReq(Some(RequestCellDestroyed { cell })));
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "dying cell's DestroyedCell chain should fire — got {}",
            captured.0.len()
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    /// Other entities with `When(DestroyedCell)` should NOT fire when a
    /// different cell is destroyed.
    #[test]
    fn bridge_destroyed_cell_does_not_evaluate_unrelated_entities() {
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
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_req, bridge_destroyed_cell).chain());

        // Dying cell — no EffectChains
        let cell = app.world_mut().spawn_empty().id();

        // Unrelated entity with DestroyedCell chain — should NOT fire
        app.world_mut().spawn(EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(Trigger::DestroyedCell, Effect::test_shockwave(32.0)),
        )]));

        app.insert_resource(SendReq(Some(RequestCellDestroyed { cell })));
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "unrelated entity's DestroyedCell chain should NOT fire — got {}",
            captured.0.len()
        );
    }
}
