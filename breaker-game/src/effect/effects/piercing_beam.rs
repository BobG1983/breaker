//! Piercing beam effect handler — fires a beam through cells in a line.
//!
//! Observes [`PiercingBeamFired`] and damages cells along the beam path.

use bevy::prelude::*;

use crate::effect::typed_events::PiercingBeamFired;

/// Observer: handles piercing beam — fires a beam through cells.
pub(crate) fn handle_piercing_beam(_trigger: On<PiercingBeamFired>) {
    // Stub: no implementation yet
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_piercing_beam);
        app
    }

    #[test]
    fn handle_piercing_beam_does_not_panic() {
        use crate::effect::typed_events::PiercingBeamFired;

        let mut app = test_app();

        app.world_mut().commands().trigger(PiercingBeamFired {
            damage_mult: 1.5,
            width: 10.0,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        // Stub handler should not panic when receiving its typed event.
    }
}
