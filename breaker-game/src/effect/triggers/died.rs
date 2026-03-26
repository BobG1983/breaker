//! Bridge systems for `Trigger::Died` — evaluates the SPECIFIC dying entity only.

use bevy::prelude::*;

use crate::{
    bolt::messages::RequestBoltDestroyed,
    cells::messages::RequestCellDestroyed,
    effect::{
        definition::{EffectChains, EffectTarget, Trigger},
        helpers::evaluate_entity_chains,
    },
};

/// Bridge for `RequestCellDestroyed` — evaluates `Trigger::Died` on the
/// SPECIFIC dying cell entity's `EffectChains` only.
pub(crate) fn bridge_cell_died(
    mut reader: MessageReader<RequestCellDestroyed>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    let trigger_kind = Trigger::Died;
    for msg in reader.read() {
        let targets = vec![EffectTarget::Entity(msg.cell)];
        if let Ok(mut chains) = chains_query.get_mut(msg.cell) {
            evaluate_entity_chains(&mut chains, trigger_kind, targets, &mut commands);
        }
    }
}

/// Bridge for `RequestBoltDestroyed` — evaluates `Trigger::Died` on the
/// SPECIFIC dying bolt entity's `EffectChains` only.
pub(crate) fn bridge_bolt_died(
    mut reader: MessageReader<RequestBoltDestroyed>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    let trigger_kind = Trigger::Died;
    for msg in reader.read() {
        let targets = vec![EffectTarget::Entity(msg.bolt)];
        if let Ok(mut chains) = chains_query.get_mut(msg.bolt) {
            evaluate_entity_chains(&mut chains, trigger_kind, targets, &mut commands);
        }
    }
}

/// Registers bridge systems for died triggers.
pub(crate) fn register(app: &mut App) {
    use crate::{effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        (bridge_cell_died, bridge_bolt_died)
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::{super::test_helpers::*, *};
    use crate::effect::definition::{Effect, EffectNode, Trigger};

    // --- Cell died bridge tests ---

    /// Cell A has `When(Died)`, Cell B has `When(Died)`.
    /// Only A fires when A dies — B should NOT fire.
    #[test]
    fn bridge_cell_died_evaluates_specific_dying_cell() {
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
            .add_systems(FixedUpdate, (send_req, bridge_cell_died).chain());

        // Cell A with Died chain (shockwave 64.0)
        let cell_a = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::trigger_leaf(Trigger::Died, Effect::test_shockwave(64.0)),
            )]))
            .id();

        // Cell B with Died chain (shockwave 32.0) — should NOT fire
        let _cell_b = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::trigger_leaf(Trigger::Died, Effect::test_shockwave(32.0)),
            )]))
            .id();

        app.insert_resource(SendReq(Some(RequestCellDestroyed { cell: cell_a })));
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "only Cell A's Died chain should fire — got {}",
            captured.0.len()
        );
        assert!(
            (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
            "fired shockwave should be Cell A's (64.0), got {}",
            captured.0[0].base_range
        );
    }

    // --- Bolt died bridge tests ---

    /// Bolt A has `When(Died)`, Bolt B has `When(Died)`.
    /// Only A fires when A dies — B should NOT fire.
    #[test]
    fn bridge_bolt_died_evaluates_specific_dying_bolt() {
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
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_req, bridge_bolt_died).chain());

        // Bolt A with Died chain (shockwave 64.0)
        let bolt_a = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::trigger_leaf(Trigger::Died, Effect::test_shockwave(64.0)),
            )]))
            .id();

        // Bolt B with Died chain (shockwave 32.0) — should NOT fire
        let _bolt_b = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::trigger_leaf(Trigger::Died, Effect::test_shockwave(32.0)),
            )]))
            .id();

        app.insert_resource(SendReq(Some(RequestBoltDestroyed { bolt: bolt_a })));
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "only Bolt A's Died chain should fire — got {}",
            captured.0.len()
        );
        assert!(
            (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
            "fired shockwave should be Bolt A's (64.0), got {}",
            captured.0[0].base_range
        );
    }
}
