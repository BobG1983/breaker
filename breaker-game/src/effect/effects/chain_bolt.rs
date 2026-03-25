//! Chain bolt effect handler — observer that translates `ChainBoltFired` into a message.

use bevy::prelude::*;

use crate::{bolt::messages::SpawnChainBolt, effect::typed_events::ChainBoltFired};

/// Observer that handles chain bolt — writes [`SpawnChainBolt`] message.
///
/// If the bolt entity is `None`, no spawn occurs.
pub(crate) fn handle_chain_bolt(
    trigger: On<ChainBoltFired>,
    mut writer: MessageWriter<SpawnChainBolt>,
) {
    let event = trigger.event();
    let Some(bolt_entity) = event.targets.iter().find_map(|t| match t {
        crate::effect::definition::EffectTarget::Entity(e) => Some(*e),
        _ => None,
    }) else {
        return;
    };
    writer.write(SpawnChainBolt {
        anchor: bolt_entity,
        tether_distance: event.tether_distance,
        source_chip: event.source_chip.clone(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct CapturedSpawnChainBolt(Vec<SpawnChainBolt>);

    fn capture_spawn_chain(
        mut reader: MessageReader<SpawnChainBolt>,
        mut captured: ResMut<CapturedSpawnChainBolt>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<SpawnChainBolt>()
            .init_resource::<CapturedSpawnChainBolt>()
            .add_observer(handle_chain_bolt)
            .add_systems(FixedUpdate, capture_spawn_chain);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 13: Writes SpawnChainBolt message on ChainBolt effect ──

    #[test]
    fn handle_chain_bolt_sends_spawn_chain_bolt_message() {
        use crate::effect::typed_events::ChainBoltFired;

        let mut app = test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();

        app.world_mut().commands().trigger(ChainBoltFired {
            tether_distance: 200.0,
            targets: vec![crate::effect::definition::EffectTarget::Entity(bolt_entity)],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnChainBolt>();
        assert_eq!(
            captured.0.len(),
            1,
            "ChainBolt effect should write one SpawnChainBolt message"
        );
        assert_eq!(
            captured.0[0].anchor, bolt_entity,
            "SpawnChainBolt.anchor should be the bolt entity from EffectFired"
        );
        assert!(
            (captured.0[0].tether_distance - 200.0).abs() < f32::EPSILON,
            "SpawnChainBolt.tether_distance should be 200.0, got {}",
            captured.0[0].tether_distance,
        );
    }

    #[test]
    fn handle_chain_bolt_with_none_bolt_does_not_spawn() {
        use crate::effect::typed_events::ChainBoltFired;

        // Given: ChainBoltFired with bolt=None
        // When: handle_chain_bolt runs
        // Then: no SpawnChainBolt message is written
        let mut app = test_app();

        app.world_mut().commands().trigger(ChainBoltFired {
            tether_distance: 200.0,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnChainBolt>();
        assert_eq!(
            captured.0.len(),
            0,
            "ChainBolt with bolt=None should not write any message"
        );
    }
}
