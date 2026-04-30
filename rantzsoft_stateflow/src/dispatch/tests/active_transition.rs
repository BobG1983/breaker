use super::helpers::*;

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
        .add_plugins(crate::RantzStateflowPlugin::new().register_state::<TestState>());

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
        .add_plugins(crate::RantzStateflowPlugin::new().register_state::<TestState>());

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
