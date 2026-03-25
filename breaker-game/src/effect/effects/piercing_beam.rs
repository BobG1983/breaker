//! Piercing beam effect handler — fires a beam through cells in a line.
//!
//! Observes [`EffectFired`], pattern-matches on
//! [`TriggerChain::PiercingBeam`], and damages cells along the beam path.

use bevy::prelude::*;

use crate::{effect::events::EffectFired, chips::definition::TriggerChain};

/// Observer: handles piercing beam — fires a beam through cells.
///
/// Self-selects via pattern matching on [`TriggerChain::PiercingBeam`].
pub(crate) fn handle_piercing_beam(trigger: On<EffectFired>) {
    let TriggerChain::PiercingBeam { .. } = &trigger.event().effect else {
        return;
    };
    // Stub: no implementation yet
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{effect::events::EffectFired, chips::definition::TriggerChain};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_piercing_beam);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn handle_piercing_beam_ignores_non_piercing_beam_effects() {
        let mut app = test_app();

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::LoseLife,
            bolt: None,
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        // If the handler incorrectly panics or processes non-matching effects,
        // this test catches it. A no-op return for non-matching variants is correct.
    }
}
