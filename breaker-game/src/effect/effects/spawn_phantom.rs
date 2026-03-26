//! Phantom breaker effect handler — spawns a temporary phantom breaker entity.
//!
//! Observes [`SpawnPhantomFired`] and spawns a phantom breaker.

use bevy::prelude::*;

use crate::effect::definition::EffectTarget;

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a spawn phantom effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct SpawnPhantomFired {
    /// How long the phantom persists in seconds.
    pub duration: f32,
    /// Maximum active phantoms at once.
    pub max_active: u32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Observer: handles phantom breaker spawning.
pub(crate) fn handle_spawn_phantom(_trigger: On<SpawnPhantomFired>) {
    // Stub: no implementation yet
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_spawn_phantom);
        app
    }

    #[test]
    fn handle_spawn_phantom_does_not_panic() {
        use crate::effect::typed_events::SpawnPhantomFired;

        let mut app = test_app();

        app.world_mut().commands().trigger(SpawnPhantomFired {
            duration: 5.0,
            max_active: 2,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        // Stub handler should not panic when receiving its typed event.
    }
}
