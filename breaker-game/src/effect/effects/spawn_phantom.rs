//! Phantom breaker effect handler — spawns a temporary phantom breaker entity.
//!
//! Observes [`SpawnPhantomFired`] and spawns a phantom breaker.

use bevy::prelude::*;

use crate::effect::typed_events::SpawnPhantomFired;

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
            bolt: None,
            source_chip: None,
        });
        app.world_mut().flush();

        // Stub handler should not panic when receiving its typed event.
    }
}
