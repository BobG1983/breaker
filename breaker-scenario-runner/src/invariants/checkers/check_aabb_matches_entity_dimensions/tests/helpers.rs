//! Shared test helpers for `check_aabb_matches_entity_dimensions` checker tests.

use bevy::prelude::*;

use super::super::checker::*;
use crate::invariants::*;

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame::default())
        .add_systems(FixedUpdate, check_aabb_matches_entity_dimensions);
    app
}
