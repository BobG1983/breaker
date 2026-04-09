//! Fixed-timestep tick helper for integration tests.

use bevy::prelude::*;

/// Advances exactly one `FixedUpdate` timestep by accumulating overstep then updating.
pub(crate) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}
