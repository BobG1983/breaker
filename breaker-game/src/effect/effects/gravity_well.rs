//! Gravity well effect handler — creates a gravity well that attracts bolts.
//!
//! Observes [`GravityWellFired`] and spawns a gravity well entity.

use bevy::prelude::*;

use crate::effect::typed_events::GravityWellFired;

/// Observer: handles gravity well creation.
pub(crate) fn handle_gravity_well(_trigger: On<GravityWellFired>) {
    // Stub: no implementation yet
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
            bolt: None,
            source_chip: None,
        });
        app.world_mut().flush();

        // Stub handler should not panic when receiving its typed event.
    }
}
