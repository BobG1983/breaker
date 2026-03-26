//! Chain lightning effect handler — arcs lightning between nearby cells.
//!
//! Observes [`ChainLightningFired`] and damages nearby cells in an arc.

use bevy::prelude::*;

use crate::effect::definition::EffectTarget;

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a chain lightning effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct ChainLightningFired {
    /// Number of arcs from the origin cell.
    pub arcs: u32,
    /// Maximum arc range in world units.
    pub range: f32,
    /// Damage multiplier per arc.
    pub damage_mult: f32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

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
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        // Stub handler should not panic when receiving its typed event.
    }
}
