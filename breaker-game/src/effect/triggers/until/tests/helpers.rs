use bevy::prelude::*;

use super::super::system::*;
pub(super) use crate::effect::core::*;

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    register(&mut app);
    app
}

/// Accumulates one fixed timestep then runs one update.
pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}
