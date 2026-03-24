//! Chain bolt effect handler — observer that translates `ChainBolt` effect into a message.

use bevy::prelude::*;

use crate::{behaviors::events::EffectFired, bolt::messages::SpawnChainBolt};

/// Observer that handles chain bolt — writes [`SpawnChainBolt`] message.
///
/// Self-selects on `TriggerChain::ChainBolt`. If the bolt entity is `None`,
/// no spawn occurs.
pub(crate) fn handle_chain_bolt(
    trigger: On<EffectFired>,
    mut writer: MessageWriter<SpawnChainBolt>,
) {
    let event = trigger.event();
    let crate::chips::definition::TriggerChain::ChainBolt { tether_distance } = &event.effect
    else {
        return;
    };
    let Some(bolt_entity) = event.bolt else {
        return;
    };
    writer.write(SpawnChainBolt {
        anchor: bolt_entity,
        tether_distance: *tether_distance,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        behaviors::events::EffectFired,
        bolt::messages::SpawnChainBolt,
        chips::definition::TriggerChain,
    };

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
        let mut app = test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::ChainBolt {
                tether_distance: 200.0,
            },
            bolt: Some(bolt_entity),
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
        // Given: EffectFired with ChainBolt effect but bolt=None
        // When: handle_chain_bolt runs
        // Then: no SpawnChainBolt message is written
        let mut app = test_app();

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::ChainBolt {
                tether_distance: 200.0,
            },
            bolt: None,
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

    // ── Behavior 14: Non-ChainBolt effects ignored ──

    #[test]
    fn non_chain_bolt_effect_does_not_send_message() {
        let mut app = test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();

        // Fire a SpawnBolt effect (not ChainBolt)
        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::SpawnBolt,
            bolt: Some(bolt_entity),
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnChainBolt>();
        assert_eq!(
            captured.0.len(),
            0,
            "SpawnBolt effect should not produce SpawnChainBolt message (self-selection)"
        );
    }

    #[test]
    fn lose_life_effect_does_not_send_message() {
        let mut app = test_app();

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::LoseLife,
            bolt: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnChainBolt>();
        assert_eq!(
            captured.0.len(),
            0,
            "LoseLife effect should not produce SpawnChainBolt message"
        );
    }
}
