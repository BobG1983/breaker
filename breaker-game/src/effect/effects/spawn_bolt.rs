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

/// Observer that handles spawn-bolt — writes [`SpawnAdditionalBolt`] messages.
///
/// Writes one [`SpawnAdditionalBolt`] message per requested bolt (`count` times).
pub(crate) fn handle_spawn_bolt(
    trigger: On<SpawnBoltFired>,
    mut writer: MessageWriter<SpawnAdditionalBolt>,
) {
    let event = trigger.event();
    for _ in 0..event.count {
        writer.write(SpawnAdditionalBolt {
            source_chip: event.source_chip.clone(),
            lifespan: event.lifespan,
            inherit: event.inherit,
        });
    }
}

/// Registers all observers and systems for the spawn bolt effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_spawn_bolt);
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Capture infrastructure (captures full messages for field assertions) ---

    #[derive(Resource, Default)]
    struct CapturedSpawnBolt(Vec<SpawnAdditionalBolt>);

    fn capture_spawn(
        mut reader: MessageReader<SpawnAdditionalBolt>,
        mut captured: ResMut<CapturedSpawnBolt>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
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

    fn trigger_spawn_bolts(
        app: &mut App,
        count: u32,
        lifespan: Option<f32>,
        inherit: bool,
        source_chip: Option<String>,
    ) {
        use crate::effect::typed_events::SpawnBoltsFired;

        app.world_mut().commands().trigger(SpawnBoltsFired {
            count,
            lifespan,
            inherit,
            targets: vec![],
            source_chip,
        });
        app.world_mut().flush();
        tick(app);
    }

    // --- Preserved existing tests (updated for new capture infrastructure) ---

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
            captured.0.len(),
            1,
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
            captured.0.len(),
            1,
            "SpawnBoltFired typed event should write one SpawnAdditionalBolt message"
        );
    }

    // =========================================================================
    // SpawnBolts handler fix — count, lifespan, inherit passthrough
    // =========================================================================

    /// `SpawnBoltsFired` with count:3 should write 3 `SpawnAdditionalBolt` messages.
    /// Current handler writes only 1 regardless of count — this test MUST fail.
    #[test]
    fn spawn_bolts_writes_count_messages() {
        let mut app = test_app();

        trigger_spawn_bolts(&mut app, 3, None, false, Some("test".to_owned()));

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0.len(),
            3,
            "SpawnBolts count:3 should write 3 SpawnAdditionalBolt messages, got {}",
            captured.0.len()
        );
    }

    /// `SpawnBoltsFired` with lifespan:Some(5.0) should pass lifespan through
    /// to each `SpawnAdditionalBolt` message.
    #[test]
    fn spawn_bolts_passes_lifespan() {
        let mut app = test_app();

        trigger_spawn_bolts(&mut app, 1, Some(5.0), false, None);

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(captured.0.len(), 1, "should produce 1 message");
        assert_eq!(
            captured.0[0].lifespan,
            Some(5.0),
            "SpawnAdditionalBolt should carry lifespan:Some(5.0), got {:?}",
            captured.0[0].lifespan
        );
    }

    /// `SpawnBoltsFired` with inherit:true should pass inherit through
    /// to each `SpawnAdditionalBolt` message.
    #[test]
    fn spawn_bolts_passes_inherit() {
        let mut app = test_app();

        trigger_spawn_bolts(&mut app, 1, None, true, None);

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(captured.0.len(), 1, "should produce 1 message");
        assert!(
            captured.0[0].inherit,
            "SpawnAdditionalBolt should have inherit:true, got false"
        );
    }

    /// `SpawnBoltsFired` with count:0 should write zero `SpawnAdditionalBolt` messages.
    /// Current handler writes 1 regardless — this test MUST fail.
    #[test]
    fn spawn_bolts_count_zero_writes_nothing() {
        let mut app = test_app();

        trigger_spawn_bolts(&mut app, 0, None, false, None);

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0.len(),
            0,
            "SpawnBolts count:0 should write 0 messages, got {}",
            captured.0.len()
        );
    }
}
