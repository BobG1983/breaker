//! Spawn-bolt consequence — observer that translates event into a message.

use bevy::prelude::*;

use crate::{
    behaviors::definition::{Consequence, ConsequenceFired},
    bolt::messages::SpawnAdditionalBolt,
};

/// Observer that handles spawn-bolt — writes [`SpawnAdditionalBolt`] message.
pub(crate) fn handle_spawn_bolt(
    trigger: On<ConsequenceFired>,
    mut writer: MessageWriter<SpawnAdditionalBolt>,
) {
    let Consequence::SpawnBolt = trigger.event().0 else {
        return;
    };
    writer.write(SpawnAdditionalBolt);
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
        app.add_plugins(MinimalPlugins);
        app.add_message::<SpawnAdditionalBolt>();
        app.init_resource::<CapturedSpawnBolt>();
        app.add_observer(handle_spawn_bolt);
        app.add_systems(FixedUpdate, capture_spawn);
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

        app.world_mut()
            .commands()
            .trigger(ConsequenceFired(Consequence::SpawnBolt));
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(captured.0, 1);
    }
}
