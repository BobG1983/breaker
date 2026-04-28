use bevy::prelude::*;
use breaker::state::run::node::messages::IncreaseNodeTimer;

use crate::invariants::{checkers::timer_monotonically_decreasing::*, *};

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

pub(super) fn test_app_timer_monotonic() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<IncreaseNodeTimer>()
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame::default())
        .add_systems(FixedUpdate, check_timer_monotonically_decreasing);
    app
}
