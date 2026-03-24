//! Second wind effect handler — grants temporary invulnerability after bolt loss.
//!
//! Observes [`EffectFired`], pattern-matches on
//! [`TriggerChain::SecondWind`], and applies invulnerability to the breaker.

use bevy::prelude::*;

use crate::{behaviors::events::EffectFired, chips::definition::TriggerChain};

/// Observer: handles second wind — temporary invulnerability.
///
/// Self-selects via pattern matching on [`TriggerChain::SecondWind`].
pub(crate) fn handle_second_wind(trigger: On<EffectFired>) {
    let TriggerChain::SecondWind { .. } = &trigger.event().effect else {
        return;
    };
    // Stub: no implementation yet
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{behaviors::events::EffectFired, chips::definition::TriggerChain};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_second_wind);
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
    fn handle_second_wind_ignores_non_second_wind_effects() {
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
