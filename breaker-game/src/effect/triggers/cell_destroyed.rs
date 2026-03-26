//! Bridge for `CellDestroyed` — sweeps ALL entities with `EffectChains`
//! for `Trigger::CellDestroyed`, evaluates `ArmedEffects` on all bolt entities.

use bevy::prelude::*;

use crate::{
    cells::messages::RequestCellDestroyed,
    effect::{
        armed::ArmedEffects,
        definition::{EffectChains, Trigger},
        helpers::{evaluate_armed_all, evaluate_entity_chains},
    },
};

/// Bridge for `RequestCellDestroyed` — sweeps ALL entities with `EffectChains`
/// for `Trigger::CellDestroyed`. Also evaluates `ArmedEffects`.
///
/// FIX from old bridge: was `ArmedEffects` only; now also sweeps entity chains.
pub(crate) fn bridge_cell_destroyed(
    mut reader: MessageReader<RequestCellDestroyed>,
    armed_query: Query<(Entity, &mut ArmedEffects)>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }

    let trigger_kind = Trigger::CellDestroyed;

    for mut chains in &mut chains_query {
        evaluate_entity_chains(&mut chains, trigger_kind, vec![], &mut commands);
    }

    evaluate_armed_all(armed_query, trigger_kind, &mut commands);
}

/// Registers bridge systems for cell destroyed trigger.
pub(crate) fn register(app: &mut App) {
    use crate::{effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        bridge_cell_destroyed
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::{super::test_helpers::*, *};
    use crate::{
        breaker::components::Breaker,
        effect::{
            armed::ArmedEffects,
            definition::{Effect, EffectNode, Trigger},
        },
    };

    // --- Test infrastructure ---

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

    fn cell_destroyed_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestCellDestroyed>()
            .insert_resource(SendCellDestroyed(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_cell_destroyed, bridge_cell_destroyed).chain(),
            );
        app
    }

    // --- CellDestroyed bridge tests ---

    /// Breaker + bolt both have `When(CellDestroyed)` — both should fire.
    #[test]
    fn bridge_cell_destroyed_sweeps_all_entity_chains() {
        let mut app = cell_destroyed_test_app();

        // Breaker entity with CellDestroyed chain
        app.world_mut().spawn((
            Breaker,
            EffectChains(vec![(
                None,
                EffectNode::trigger_leaf(Trigger::CellDestroyed, Effect::test_shockwave(64.0)),
            )]),
        ));

        // Bolt entity with CellDestroyed chain (no Breaker marker)
        app.world_mut().spawn(EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(Trigger::CellDestroyed, Effect::test_shockwave(32.0)),
        )]));

        let cell = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendCellDestroyed>().0 = Some(RequestCellDestroyed { cell });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            2,
            "both breaker and bolt entity chains should fire on CellDestroyed — got {}",
            captured.0.len()
        );
    }

    /// `ArmedEffects` should fire on `CellDestroyed`.
    #[test]
    fn bridge_cell_destroyed_evaluates_armed() {
        let mut app = cell_destroyed_test_app();

        app.world_mut().spawn(ArmedEffects(vec![(
            None,
            EffectNode::trigger_leaf(Trigger::CellDestroyed, Effect::test_shockwave(48.0)),
        )]));

        let cell = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendCellDestroyed>().0 = Some(RequestCellDestroyed { cell });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "armed CellDestroyed chain should fire — got {}",
            captured.0.len()
        );
        assert!((captured.0[0].base_range - 48.0).abs() < f32::EPSILON);
    }

    /// No `RequestCellDestroyed` message means no firing.
    #[test]
    fn bridge_cell_destroyed_noop_without_message() {
        let mut app = cell_destroyed_test_app();

        app.world_mut().spawn(EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(Trigger::CellDestroyed, Effect::test_shockwave(64.0)),
        )]));

        // Do NOT send any message
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "no RequestCellDestroyed message — nothing should fire"
        );
    }
}
