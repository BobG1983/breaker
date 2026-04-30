use super::helpers::*;

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
