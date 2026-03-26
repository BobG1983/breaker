//! Shared test infrastructure for trigger bridge tests.

use bevy::prelude::*;

use crate::effect::{definition::EffectNode, typed_events::ShockwaveFired};

/// Tick one fixed update frame.
pub(crate) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

/// Wrap chains in the `(Option<String>, EffectNode)` format for `EffectChains`.
pub(crate) fn wrap_chains(chains: Vec<EffectNode>) -> Vec<(Option<String>, EffectNode)> {
    chains.into_iter().map(|c| (None, c)).collect()
}

/// Captured `ShockwaveFired` events for test assertions.
#[derive(Resource, Default)]
pub(crate) struct CapturedShockwaveFired(pub Vec<ShockwaveFired>);

/// Observer system that captures `ShockwaveFired` events into the resource.
pub(crate) fn capture_shockwave_fired(
    trigger: On<ShockwaveFired>,
    mut captured: ResMut<CapturedShockwaveFired>,
) {
    captured.0.push(trigger.event().clone());
}
