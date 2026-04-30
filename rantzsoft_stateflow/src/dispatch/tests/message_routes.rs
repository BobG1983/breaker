use super::helpers::*;

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
