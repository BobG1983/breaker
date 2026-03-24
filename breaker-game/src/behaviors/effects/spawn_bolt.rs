//! Spawn-bolt effect handler — observer that translates event into a message.

use bevy::prelude::*;

use crate::{
    behaviors::events::EffectFired, bolt::messages::SpawnAdditionalBolt,
    chips::definition::TriggerChain,
};

/// Observer that handles spawn-bolt — writes [`SpawnAdditionalBolt`] message.
pub(crate) fn handle_spawn_bolt(
    trigger: On<EffectFired>,
    mut writer: MessageWriter<SpawnAdditionalBolt>,
) {
    let TriggerChain::SpawnBolt = &trigger.event().effect else {
        return;
    };
    writer.write(SpawnAdditionalBolt {
        source_chip: trigger.event().source_chip.clone(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct CapturedSpawnBolt(u32);

    fn capture_spawn(
        mut reader: MessageReader<SpawnAdditionalBolt>,
        mut captured: ResMut<CapturedSpawnBolt>,
    ) {
        for _msg in reader.read() {
            captured.0 += 1;
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<SpawnAdditionalBolt>()
            .init_resource::<CapturedSpawnBolt>()
            .add_observer(handle_spawn_bolt)
            .add_systems(FixedUpdate, capture_spawn);
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
    fn handle_spawn_bolt_sends_message() {
        let mut app = test_app();

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::SpawnBolt,
            bolt: None,
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0, 1,
            "SpawnBolt effect should write one SpawnAdditionalBolt message"
        );
    }

    #[test]
    fn non_spawn_bolt_effect_does_not_send_message() {
        let mut app = test_app();

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::LoseLife,
            bolt: None,
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0, 0,
            "LoseLife effect should not produce SpawnAdditionalBolt (self-selection)"
        );
    }
}
