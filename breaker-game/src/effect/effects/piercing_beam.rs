//! Piercing beam effect handler — fires a beam through cells in a line.
//!
//! Observes [`PiercingBeamFired`] and damages cells along the beam path.

use bevy::prelude::*;

use crate::effect::definition::EffectTarget;

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a piercing beam effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct PiercingBeamFired {
    /// Damage multiplier for the beam.
    pub damage_mult: f32,
    /// Width of the beam in world units.
    pub width: f32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Observer: handles piercing beam — fires a beam through cells.
pub(crate) fn handle_piercing_beam(_trigger: On<PiercingBeamFired>) {
    // Stub: no implementation yet
}

/// Registers all observers and systems for the piercing beam effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_piercing_beam);
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
