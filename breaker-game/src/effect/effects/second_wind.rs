//! Second wind effect handler — grants temporary invulnerability after bolt loss.
//!
//! Observes [`SecondWindFired`] and applies invulnerability to the breaker.

use bevy::prelude::*;

use crate::effect::typed_events::SecondWindFired;

/// Observer: handles second wind — temporary invulnerability.
pub(crate) fn handle_second_wind(_trigger: On<SecondWindFired>) {
    // Stub: no implementation yet
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
            bolt: None,
            source_chip: None,
        });
        app.world_mut().flush();

        // Stub handler should not panic when receiving its typed event.
    }
}
