//! Gravity well effect handler — creates a gravity well that attracts bolts.
//!
//! Observes [`GravityWellFired`] and spawns a gravity well entity.

use bevy::prelude::*;

use crate::effect::definition::EffectTarget;

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a gravity well effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct GravityWellFired {
    /// Attraction strength.
    pub strength: f32,
    /// Duration in seconds.
    pub duration: f32,
    /// Effect radius in world units.
    pub radius: f32,
    /// Maximum active wells at once.
    pub max: u32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Observer: handles gravity well creation.
pub(crate) fn handle_gravity_well(_trigger: On<GravityWellFired>) {
    // Stub: no implementation yet
}

/// Registers all observers and systems for the gravity well effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_gravity_well);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_gravity_well);
        app
    }

    #[test]
    fn handle_gravity_well_does_not_panic() {
        use crate::effect::typed_events::GravityWellFired;

        let mut app = test_app();

        app.world_mut().commands().trigger(GravityWellFired {
            strength: 50.0,
            duration: 5.0,
            radius: 100.0,
            max: 2,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        // Stub handler should not panic when receiving its typed event.
    }
}
