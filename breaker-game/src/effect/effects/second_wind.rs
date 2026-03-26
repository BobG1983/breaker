//! Second wind effect handler — grants temporary invulnerability after bolt loss.
//!
//! Observes [`SecondWindFired`] and applies invulnerability to the breaker.

use bevy::prelude::*;

use crate::effect::definition::EffectTarget;

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a second wind effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct SecondWindFired {
    /// Duration of invulnerability in seconds.
    pub invuln_secs: f32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Observer: handles second wind — temporary invulnerability.
pub(crate) fn handle_second_wind(_trigger: On<SecondWindFired>) {
    // Stub: no implementation yet
}

/// Registers all observers and systems for the second wind effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_second_wind);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_second_wind);
        app
    }

    #[test]
    fn handle_second_wind_does_not_panic() {
        use crate::effect::typed_events::SecondWindFired;

        let mut app = test_app();

        app.world_mut().commands().trigger(SecondWindFired {
            invuln_secs: 3.0,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        // Stub handler should not panic when receiving its typed event.
    }
}
