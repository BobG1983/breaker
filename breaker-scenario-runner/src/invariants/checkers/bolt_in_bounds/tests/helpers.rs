//! Shared test helpers for `bolt_in_bounds` checker tests.

use bevy::prelude::*;
use breaker::shared::PlayfieldConfig;

use crate::invariants::{checkers::bolt_in_bounds::*, *};

pub(super) fn test_app_bolt_in_bounds() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(PlayfieldConfig::default())
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame::default())
        .add_systems(FixedUpdate, check_bolt_in_bounds);
    app
}

pub(super) fn test_app_bolt_in_bounds_with_radius() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
            zone_fraction: 0.667,
        })
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame::default())
        .add_systems(FixedUpdate, check_bolt_in_bounds);
    app
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}
