use bevy::prelude::*;

/// Tracks whether a test system's query loop body actually executed.
#[derive(Resource, Default)]
pub(super) struct QueryMatched(pub bool);

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<QueryMatched>();
    app
}

/// Asserts the system's query matched at least one entity.
pub(super) fn assert_query_matched(app: &App) {
    assert!(
        app.world().resource::<QueryMatched>().0,
        "QueryData system should have matched at least one entity"
    );
}

/// Accumulates one fixed timestep then runs one update.
pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}
