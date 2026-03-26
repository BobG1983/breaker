//! Spawn-bolt effect handler — observer that translates event into a message.

use bevy::prelude::*;

use crate::{bolt::messages::SpawnAdditionalBolt, effect::definition::EffectTarget};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a spawn-bolts effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct SpawnBoltsFired {
    /// Number of bolts to spawn.
    pub count: u32,
    /// Optional lifespan in seconds (temporary bolts).
    pub lifespan: Option<f32>,
    /// Whether spawned bolts inherit the parent bolt's velocity.
    pub inherit: bool,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Backward-compatible alias — production code still references this name.
///
/// Will be removed when all handler files are updated.
pub(crate) type SpawnBoltFired = SpawnBoltsFired;

/// Observer that handles spawn-bolt — writes [`SpawnAdditionalBolt`] message.
pub(crate) fn handle_spawn_bolt(
    trigger: On<SpawnBoltFired>,
    mut writer: MessageWriter<SpawnAdditionalBolt>,
) {
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
        use crate::effect::typed_events::SpawnBoltFired;

        let mut app = test_app();

        app.world_mut().commands().trigger(SpawnBoltFired {
            count: 1,
            lifespan: None,
            inherit: false,
            targets: vec![],
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

    // =========================================================================
    // B12c: handle_spawn_bolt observes SpawnBoltFired (not EffectFired)
    // =========================================================================

    #[test]
    fn spawn_bolt_fired_sends_spawn_message() {
        use crate::effect::typed_events::SpawnBoltFired;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<SpawnAdditionalBolt>()
            .init_resource::<CapturedSpawnBolt>()
            .add_observer(handle_spawn_bolt)
            .add_systems(FixedUpdate, capture_spawn);

        app.world_mut().commands().trigger(SpawnBoltFired {
            count: 1,
            lifespan: None,
            inherit: false,
            targets: vec![],
            source_chip: Some("Reflex".to_owned()),
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0, 1,
            "SpawnBoltFired typed event should write one SpawnAdditionalBolt message"
        );
    }
}
