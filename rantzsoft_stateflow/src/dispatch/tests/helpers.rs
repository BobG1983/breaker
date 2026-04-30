pub(super) use bevy::{prelude::*, state::app::StatesPlugin};

pub(super) use super::super::system::*;
pub(super) use crate::{
    Route,
    messages::{ChangeState, StateChanged},
    routing_table::RoutingTable,
};

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum TestState {
    #[default]
    Loading,
    AnimateIn,
    Playing,
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<TestState>()
        .add_message::<ChangeState<TestState>>()
        .add_message::<StateChanged<TestState>>()
        .init_resource::<RoutingTable<TestState>>()
        .add_systems(
            Update,
            dispatch_message_routes::<TestState>.run_if(on_message::<ChangeState<TestState>>),
        );
    app
}

pub(super) fn send_change_state(app: &mut App) {
    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<ChangeState<TestState>>>()
        .write(ChangeState::new());
}

/// Helper: builds an app with a message writer in `writer_schedule` and
/// dispatch in `dispatch_schedule`. Uses `ManualDuration` to ensure fixed
/// ticks fire deterministically. Returns whether the route fires within
/// 10 `update()` calls.
pub(super) fn probe_schedule_bridging(
    writer_schedule: impl bevy::ecs::schedule::ScheduleLabel + Clone,
    dispatch_schedule: impl bevy::ecs::schedule::ScheduleLabel + Clone,
) -> bool {
    fn write_once(mut writer: MessageWriter<ChangeState<TestState>>, mut fired: Local<bool>) {
        if !*fired {
            writer.write(ChangeState::new());
            *fired = true;
        }
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<TestState>()
        .add_message::<ChangeState<TestState>>()
        .add_message::<StateChanged<TestState>>()
        .init_resource::<RoutingTable<TestState>>()
        // Ensure each update() advances enough time for one fixed tick
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(20),
        ))
        .add_systems(writer_schedule, write_once)
        .add_systems(
            dispatch_schedule,
            dispatch_message_routes::<TestState>.run_if(on_message::<ChangeState<TestState>>),
        );

    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
        .ok();

    // Give 10 frames for the message to propagate + state transition to apply
    for _ in 0..10 {
        app.update();
    }

    **app.world().resource::<State<TestState>>() == TestState::AnimateIn
}
