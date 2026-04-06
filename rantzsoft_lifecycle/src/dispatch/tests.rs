use bevy::{prelude::*, state::app::StatesPlugin};

use super::system::*;
use crate::{
    Route,
    messages::{ChangeState, StateChanged},
    routing_table::RoutingTable,
};

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum TestState {
    #[default]
    Loading,
    AnimateIn,
    Playing,
}

fn test_app() -> App {
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

fn send_change_state(app: &mut App) {
    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<ChangeState<TestState>>>()
        .write(ChangeState::new());
}

// --- Message-triggered dispatch ---

#[test]
fn dispatch_transitions_via_static_route() {
    let mut app = test_app();
    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
        .ok();

    app.update();

    send_change_state(&mut app);
    app.update(); // dispatch fires, sets NextState
    app.update(); // state transition applies

    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::AnimateIn,
    );
}

#[test]
fn dispatch_transitions_via_dynamic_route() {
    let mut app = test_app();
    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(Route::from(TestState::Loading).to_dynamic(|_world| TestState::Playing))
        .ok();

    app.update();

    send_change_state(&mut app);
    app.update();
    app.update();

    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::Playing,
    );
}

#[test]
fn dispatch_sends_state_changed_message() {
    let mut app = test_app();
    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
        .ok();

    app.update();

    send_change_state(&mut app);
    app.update(); // dispatch fires

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<StateChanged<TestState>>>();
    let changed: Vec<_> = msgs.iter_current_update_messages().collect();
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].from, TestState::Loading);
    assert_eq!(changed[0].to, TestState::AnimateIn);
}

#[test]
fn dispatch_does_nothing_without_route() {
    let mut app = test_app();
    app.update();

    send_change_state(&mut app);
    app.update();
    app.update();

    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::Loading,
    );
}

#[test]
fn dispatch_does_nothing_without_message() {
    let mut app = test_app();
    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
        .ok();

    app.update();
    app.update();

    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::Loading,
    );
}

#[test]
fn dispatch_skips_condition_triggered_routes() {
    let mut app = test_app();
    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(
            Route::from(TestState::Loading)
                .to(TestState::AnimateIn)
                .when(|_| true),
        )
        .ok();

    app.update();

    send_change_state(&mut app);
    app.update();
    app.update();

    // Condition route should be skipped by message dispatch
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::Loading,
    );
}

#[test]
fn dispatch_chains_through_multiple_routes() {
    let mut app = test_app();
    {
        let mut table = app.world_mut().resource_mut::<RoutingTable<TestState>>();
        table
            .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
            .ok();
        table
            .add(Route::from(TestState::AnimateIn).to(TestState::Playing))
            .ok();
    }

    app.update();

    // First: Loading → AnimateIn
    send_change_state(&mut app);
    app.update();
    app.update();
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::AnimateIn,
    );

    // Second: AnimateIn → Playing
    send_change_state(&mut app);
    app.update();
    app.update();
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::Playing,
    );
}

// --- Condition-triggered dispatch ---

#[test]
fn condition_dispatch_fires_when_condition_true() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<TestState>()
        .add_message::<StateChanged<TestState>>()
        .init_resource::<RoutingTable<TestState>>()
        .add_systems(Update, dispatch_condition_routes::<TestState>);

    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(
            Route::from(TestState::Loading)
                .to(TestState::AnimateIn)
                .when(|_world| true),
        )
        .ok();

    app.update(); // initial
    app.update(); // condition evaluates to true, sets NextState
    app.update(); // state applies

    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::AnimateIn,
    );
}

#[test]
fn condition_dispatch_does_not_fire_when_condition_false() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<TestState>()
        .add_message::<StateChanged<TestState>>()
        .init_resource::<RoutingTable<TestState>>()
        .add_systems(Update, dispatch_condition_routes::<TestState>);

    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(
            Route::from(TestState::Loading)
                .to(TestState::AnimateIn)
                .when(|_world| false),
        )
        .ok();

    app.update();
    app.update();
    app.update();

    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::Loading,
    );
}

#[test]
fn condition_dispatch_reads_world_in_condition() {
    /// Resource used by the condition closure.
    #[derive(Resource)]
    struct ReadyFlag(bool);

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<TestState>()
        .add_message::<StateChanged<TestState>>()
        .init_resource::<RoutingTable<TestState>>()
        .insert_resource(ReadyFlag(false))
        .add_systems(Update, dispatch_condition_routes::<TestState>);

    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(
            Route::from(TestState::Loading)
                .to(TestState::AnimateIn)
                .when(|world| world.resource::<ReadyFlag>().0),
        )
        .ok();

    // Flag is false — should not transition
    app.update();
    app.update();
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::Loading,
    );

    // Set flag to true — should transition
    app.world_mut().resource_mut::<ReadyFlag>().0 = true;
    app.update();
    app.update();

    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::AnimateIn,
    );
}

#[test]
fn condition_dispatch_skips_message_triggered_routes() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<TestState>()
        .add_message::<StateChanged<TestState>>()
        .init_resource::<RoutingTable<TestState>>()
        .add_systems(Update, dispatch_condition_routes::<TestState>);

    // Message-triggered route (no .when())
    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
        .ok();

    app.update();
    app.update();
    app.update();

    // Condition dispatch should skip message-triggered routes
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::Loading,
    );
}

// --- SubState test ---

// --- Section K: Deferred ChangeState during Active Transition ---

// Behavior 29: Message dispatch skips route execution when ActiveTransition exists
#[test]
fn dispatch_skips_route_when_active_transition_exists() {
    let mut app = test_app();
    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
        .ok();

    app.update();

    // Manually insert ActiveTransition (simulating an active transition)
    app.world_mut()
        .insert_resource(crate::transition::resources::ActiveTransition);

    send_change_state(&mut app);
    app.update();
    app.update();

    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::Loading,
        "State should remain Loading when ActiveTransition is present"
    );
}

// Behavior 30: Condition dispatch skips route execution when ActiveTransition exists
#[test]
fn condition_dispatch_skips_when_active_transition_exists() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<TestState>()
        .add_message::<StateChanged<TestState>>()
        .init_resource::<RoutingTable<TestState>>()
        .add_systems(Update, dispatch_condition_routes::<TestState>);

    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(
            Route::from(TestState::Loading)
                .to(TestState::AnimateIn)
                .when(|_world| true),
        )
        .ok();

    // Insert ActiveTransition BEFORE any update — the condition when(|_| true)
    // fires on the first frame, so ActiveTransition must be present from the start.
    app.world_mut()
        .insert_resource(crate::transition::resources::ActiveTransition);

    app.update(); // condition evaluates to true but dispatch should skip
    app.update();
    app.update();

    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::Loading,
        "State should remain Loading when ActiveTransition is present despite condition being true"
    );
}

// --- Section M: Route Without Transition (regression) ---

// Behavior 39: Route without transition still works after transition infrastructure is added
#[test]
fn plain_route_still_works_after_transition_infrastructure_added() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<TestState>()
        .add_plugins(crate::RantzLifecyclePlugin::new().register_state::<TestState>());

    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
        .ok();

    app.update();

    send_change_state(&mut app);
    app.update();
    app.update();

    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::AnimateIn,
        "Plain route (no transition) should still transition state"
    );

    // No ActiveTransition should have been inserted
    assert!(
        !app.world()
            .contains_resource::<crate::transition::resources::ActiveTransition>(),
        "No ActiveTransition should be inserted for plain routes"
    );

    // No TransitionStart should have been sent
    let start_msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<crate::messages::TransitionStart<TestState>>>();
    assert_eq!(
        start_msgs.iter_current_update_messages().count(),
        0,
        "No TransitionStart should be sent for plain routes"
    );
}

// Behavior 40: Condition-triggered route without transition still works
#[test]
fn condition_route_without_transition_still_works() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<TestState>()
        .add_plugins(crate::RantzLifecyclePlugin::new().register_state::<TestState>());

    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(
            Route::from(TestState::Loading)
                .to(TestState::AnimateIn)
                .when(|_world| true),
        )
        .ok();

    app.update(); // initial
    app.update(); // condition fires
    app.update(); // state applies

    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::AnimateIn,
        "Condition route (no transition) should still transition state"
    );

    // No transition infrastructure touched
    assert!(
        !app.world()
            .contains_resource::<crate::transition::resources::ActiveTransition>(),
        "No ActiveTransition for condition routes without transition"
    );
}

// --- SubState test ---

#[test]
fn dispatch_works_with_substates() {
    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum Parent {
        #[default]
        Off,
        On,
    }

    #[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    #[source(Parent = Parent::On)]
    enum Child {
        #[default]
        Loading,
        Ready,
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<Parent>()
        .add_sub_state::<Child>()
        .add_message::<ChangeState<Child>>()
        .add_message::<StateChanged<Child>>()
        .init_resource::<RoutingTable<Child>>()
        .add_systems(
            Update,
            dispatch_message_routes::<Child>.run_if(on_message::<ChangeState<Child>>),
        );

    app.world_mut()
        .resource_mut::<RoutingTable<Child>>()
        .add(Route::from(Child::Loading).to(Child::Ready))
        .ok();

    // Activate parent
    app.world_mut()
        .resource_mut::<NextState<Parent>>()
        .set(Parent::On);
    app.update();

    assert_eq!(**app.world().resource::<State<Child>>(), Child::Loading);

    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<ChangeState<Child>>>()
        .write(ChangeState::new());
    app.update();
    app.update();

    assert_eq!(**app.world().resource::<State<Child>>(), Child::Ready);
}

// --- Schedule placement: which dispatch schedule sees which writer schedule? ---
//
// Bevy 0.18 frame order:
//   First → PreUpdate → StateTransition → RunFixedMainLoop(
//       FixedFirst → FixedPreUpdate → FixedUpdate → FixedPostUpdate → FixedLast
//   ) → Update → PostUpdate → Last
//
// These tests write ChangeState from one schedule and place dispatch in another,
// proving which combinations work. A fire-once Local<bool> guard ensures the
// message is written exactly once.

/// Helper: builds an app with a message writer in `writer_schedule` and
/// dispatch in `dispatch_schedule`. Uses `ManualDuration` to ensure fixed
/// ticks fire deterministically. Returns whether the route fires within
/// 10 `update()` calls.
fn probe_schedule_bridging(
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

// --- Writer in Update ---

#[test]
fn update_writer_update_dispatch() {
    assert!(
        probe_schedule_bridging(Update, Update),
        "Update → Update: message and dispatch in same schedule must work"
    );
}

#[test]
fn update_writer_post_update_dispatch() {
    assert!(
        probe_schedule_bridging(Update, PostUpdate),
        "Update → PostUpdate: dispatch after writer in same frame must work"
    );
}

#[test]
fn update_writer_fixed_post_update_dispatch() {
    // FixedPostUpdate runs BEFORE Update in the frame, so messages
    // written in Update won't be visible until the next frame's fixed loop.
    let result = probe_schedule_bridging(Update, FixedPostUpdate);
    // Record actual behavior — don't assert direction, just observe
    eprintln!("Update → FixedPostUpdate: {result}");
}

// --- Writer in FixedUpdate ---

#[test]
fn fixed_update_writer_update_dispatch() {
    // This is the broken case: FixedUpdate runs before Update, but
    // message double-buffering may hide the message.
    let result = probe_schedule_bridging(FixedUpdate, Update);
    eprintln!("FixedUpdate → Update: {result}");
}

#[test]
fn fixed_update_writer_post_update_dispatch() {
    let result = probe_schedule_bridging(FixedUpdate, PostUpdate);
    eprintln!("FixedUpdate → PostUpdate: {result}");
}

#[test]
fn fixed_update_writer_fixed_post_update_dispatch() {
    let result = probe_schedule_bridging(FixedUpdate, FixedPostUpdate);
    eprintln!("FixedUpdate → FixedPostUpdate: {result}");
}

/// Same as the FixedUpdate→FixedPostUpdate test but WITHOUT the `on_message`
/// run condition — dispatch runs unconditionally. Also verifies the fixed
/// tick actually runs by tracking a counter.
#[test]
fn fixed_update_writer_fixed_post_update_dispatch_no_run_condition() {
    #[derive(Resource, Default)]
    struct WriteCount(u32);

    fn write_once(
        mut writer: MessageWriter<ChangeState<TestState>>,
        mut count: ResMut<WriteCount>,
    ) {
        count.0 += 1;
        if count.0 == 1 {
            writer.write(ChangeState::new());
        }
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<TestState>()
        .init_resource::<WriteCount>()
        .add_message::<ChangeState<TestState>>()
        .add_message::<StateChanged<TestState>>()
        .init_resource::<RoutingTable<TestState>>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(20),
        ))
        .add_systems(FixedUpdate, write_once)
        .add_systems(FixedPostUpdate, dispatch_message_routes::<TestState>);

    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
        .ok();

    for _ in 0..10 {
        app.update();
    }

    let writes = app.world().resource::<WriteCount>().0;
    let state = **app.world().resource::<State<TestState>>();
    eprintln!("FixedUpdate → FixedPostUpdate (no run_if): writes={writes} state={state:?}");
    assert!(
        writes > 0,
        "write_once must have fired at least once (verifies FixedUpdate ran)"
    );
    assert_eq!(
        state,
        TestState::AnimateIn,
        "dispatch in FixedPostUpdate should see FixedUpdate messages"
    );
}

// --- Writer in FixedPostUpdate ---

#[test]
fn fixed_post_update_writer_update_dispatch() {
    let result = probe_schedule_bridging(FixedPostUpdate, Update);
    eprintln!("FixedPostUpdate → Update: {result}");
}

#[test]
fn fixed_post_update_writer_post_update_dispatch() {
    let result = probe_schedule_bridging(FixedPostUpdate, PostUpdate);
    eprintln!("FixedPostUpdate → PostUpdate: {result}");
}
