//! Chain lightning effect handler — arcs lightning between nearby cells.
//!
//! Observes [`ChainLightningFired`] and damages nearby cells in an arc.

use bevy::prelude::*;

use crate::effect::typed_events::ChainLightningFired;

/// Observer: handles chain lightning — arcs damage between cells.
pub(crate) fn handle_chain_lightning(_trigger: On<ChainLightningFired>) {
    // Stub: no implementation yet
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_chain_lightning);
        app
    }

    #[test]
    fn handle_chain_lightning_does_not_panic() {
        use crate::effect::typed_events::ChainLightningFired;

        let mut app = test_app();

        app.world_mut().commands().trigger(ChainLightningFired {
            arcs: 3,
            range: 100.0,
            damage_mult: 1.0,
            bolt: None,
            source_chip: None,
        });
        app.world_mut().flush();

        // Stub handler should not panic when receiving its typed event.
    }
}
