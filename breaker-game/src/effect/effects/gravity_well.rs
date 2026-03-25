//! Gravity well effect handler — creates a gravity well that attracts bolts.
//!
//! Observes [`EffectFired`], pattern-matches on
//! [`TriggerChain::GravityWell`], and spawns a gravity well entity.

use bevy::prelude::*;

use crate::{chips::definition::TriggerChain, effect::events::EffectFired};

/// Observer: handles gravity well creation.
///
/// Self-selects via pattern matching on [`TriggerChain::GravityWell`].
pub(crate) fn handle_gravity_well(trigger: On<EffectFired>) {
    let TriggerChain::GravityWell { .. } = &trigger.event().effect else {
        return;
    };
    // Stub: no implementation yet
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chips::definition::TriggerChain, effect::events::EffectFired};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_gravity_well);
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
    fn handle_gravity_well_ignores_non_gravity_well_effects() {
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
