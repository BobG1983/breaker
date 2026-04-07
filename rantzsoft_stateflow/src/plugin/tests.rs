use bevy::{prelude::*, state::app::StatesPlugin};

use super::definition::*;
use crate::{
    Route, RoutingTable,
    messages::{ChangeState, StateChanged, TransitionEnd, TransitionStart},
    routing_table::RoutingTableAppExt,
    transition::{
        messages::{TransitionOver, TransitionReady, TransitionRunComplete},
        registry::TransitionRegistry,
        traits::{InTransition, OutTransition, Transition},
    },
};

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum AppState {
    #[default]
    Loading,
    Game,
}

#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(AppState = AppState::Game)]
enum GameState {
    #[default]
    Menu,
}

// Test effect types
struct TestEffectOut;
impl Transition for TestEffectOut {}
impl OutTransition for TestEffectOut {}

struct TestEffectIn;
impl Transition for TestEffectIn {}
impl InTransition for TestEffectIn {}

// Test effect systems (instant completion)
fn test_start_sys(mut writer: MessageWriter<TransitionReady>) {
    writer.write(TransitionReady);
}
fn test_run_sys(mut writer: MessageWriter<TransitionRunComplete>) {
    writer.write(TransitionRunComplete);
}
fn test_end_sys(mut writer: MessageWriter<TransitionOver>) {
    writer.write(TransitionOver);
}
fn test_start_sys_in(mut writer: MessageWriter<TransitionReady>) {
    writer.write(TransitionReady);
}
fn test_run_sys_in(mut writer: MessageWriter<TransitionRunComplete>) {
    writer.write(TransitionRunComplete);
}
fn test_end_sys_in(mut writer: MessageWriter<TransitionOver>) {
    writer.write(TransitionOver);
}

// --- Existing tests (preserved) ---

#[test]
fn plugin_builds_and_registers_state_types() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_sub_state::<GameState>()
        .add_plugins(
            RantzStateflowPlugin::new()
                .register_state::<AppState>()
                .register_state::<GameState>(),
        );
    app.update();

    // Routing tables should exist
    assert!(app.world().contains_resource::<RoutingTable<AppState>>());
    assert!(app.world().contains_resource::<RoutingTable<GameState>>());
}

#[test]
fn plugin_dispatch_works_end_to_end() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_sub_state::<GameState>()
        .add_plugins(
            RantzStateflowPlugin::new()
                .register_state::<AppState>()
                .register_state::<GameState>(),
        );

    // Add route: Loading → Game
    app.add_route(Route::from(AppState::Loading).to(AppState::Game));

    app.update();
    assert_eq!(
        **app.world().resource::<State<AppState>>(),
        AppState::Loading
    );

    // Send ChangeState — plugin's dispatch should route Loading → Game
    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<ChangeState<AppState>>>()
        .write(ChangeState::new());
    app.update();
    app.update();

    assert_eq!(**app.world().resource::<State<AppState>>(), AppState::Game,);
}

#[test]
fn plugin_condition_dispatch_works() {
    #[derive(Resource)]
    struct Ready(bool);

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .insert_resource(Ready(false))
        .add_plugins(RantzStateflowPlugin::new().register_state::<AppState>());

    app.add_route(
        Route::from(AppState::Loading)
            .to(AppState::Game)
            .when(|world| world.resource::<Ready>().0),
    );

    app.update();
    app.update();
    // Not ready yet
    assert_eq!(
        **app.world().resource::<State<AppState>>(),
        AppState::Loading
    );

    app.world_mut().resource_mut::<Ready>().0 = true;
    app.update();
    app.update();

    assert_eq!(**app.world().resource::<State<AppState>>(), AppState::Game,);
}

#[test]
fn plugin_sends_state_changed_message() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_plugins(RantzStateflowPlugin::new().register_state::<AppState>());

    app.add_route(Route::from(AppState::Loading).to(AppState::Game));
    app.update();

    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<ChangeState<AppState>>>()
        .write(ChangeState::new());
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<StateChanged<AppState>>>();
    let changed: Vec<_> = msgs.iter_current_update_messages().collect();
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].from, AppState::Loading);
    assert_eq!(changed[0].to, AppState::Game);
}

// --- Section L: register_custom_transition ---

// Behavior 33: register_custom_transition registers effect systems gated on marker resources
#[test]
fn register_custom_transition_registers_effect_in_registry() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_plugins(
            RantzStateflowPlugin::new()
                .register_state::<AppState>()
                .register_custom_transition::<TestEffectOut, _, _, _>(
                    test_start_sys,
                    test_run_sys,
                    test_end_sys,
                ),
        );
    app.update();

    let registry = app.world().resource::<TransitionRegistry>();
    assert!(
        registry.contains::<TestEffectOut>(),
        "TransitionRegistry should contain TestEffectOut after register_custom_transition"
    );
}

#[test]
fn effect_systems_do_not_run_when_marker_absent() {
    // This test verifies that the gated systems do NOT run when
    // their marker resources are absent. Since the test effect systems
    // send TransitionReady/RunComplete/Over, we check that no such
    // messages exist after an update with no markers present.
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_plugins(
            RantzStateflowPlugin::new()
                .register_state::<AppState>()
                .register_custom_transition::<TestEffectOut, _, _, _>(
                    test_start_sys,
                    test_run_sys,
                    test_end_sys,
                ),
        );
    app.update();

    // No marker resources inserted — systems should not fire
    let ready_msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(
        ready_msgs.iter_current_update_messages().count(),
        0,
        "TransitionReady should not be sent when StartingTransition is absent"
    );
}

// Behavior 34: register_custom_transition for multiple effects
#[test]
fn register_multiple_custom_transitions() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_plugins(
            RantzStateflowPlugin::new()
                .register_state::<AppState>()
                .register_custom_transition::<TestEffectOut, _, _, _>(
                    test_start_sys,
                    test_run_sys,
                    test_end_sys,
                )
                .register_custom_transition::<TestEffectIn, _, _, _>(
                    test_start_sys_in,
                    test_run_sys_in,
                    test_end_sys_in,
                ),
        );
    app.update();

    let registry = app.world().resource::<TransitionRegistry>();
    assert!(registry.contains::<TestEffectOut>());
    assert!(registry.contains::<TestEffectIn>());
}

// Behavior 35: Plugin registers internal messages
#[test]
fn plugin_registers_internal_messages() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_plugins(RantzStateflowPlugin::new().register_state::<AppState>());
    app.update();

    // The internal messages should be registered (pub(crate) but resources exist)
    assert!(
        app.world()
            .contains_resource::<bevy::ecs::message::Messages<TransitionReady>>(),
        "TransitionReady message should be registered"
    );
    assert!(
        app.world()
            .contains_resource::<bevy::ecs::message::Messages<TransitionRunComplete>>(),
        "TransitionRunComplete message should be registered"
    );
    assert!(
        app.world()
            .contains_resource::<bevy::ecs::message::Messages<TransitionOver>>(),
        "TransitionOver message should be registered"
    );
}

// Behavior 36: Plugin registers TransitionStart<S> and TransitionEnd<S> per state
#[test]
fn plugin_registers_transition_start_and_end_per_state() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_plugins(RantzStateflowPlugin::new().register_state::<AppState>());
    app.update();

    assert!(
        app.world()
            .contains_resource::<bevy::ecs::message::Messages<TransitionStart<AppState>>>(),
        "TransitionStart<AppState> should be registered"
    );
    assert!(
        app.world()
            .contains_resource::<bevy::ecs::message::Messages<TransitionEnd<AppState>>>(),
        "TransitionEnd<AppState> should be registered"
    );
}

// Behavior 37: Plugin registers TransitionRegistry resource
#[test]
fn plugin_registers_transition_registry() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_plugins(RantzStateflowPlugin::new());
    app.update();

    assert!(
        app.world().contains_resource::<TransitionRegistry>(),
        "TransitionRegistry resource should exist after plugin builds"
    );
}

// Behavior 38: Plugin registers orchestration system
// This is verified implicitly by the end-to-end orchestration tests in Section G.
// Here we just verify that the plugin builds without panicking when the
// orchestration system is registered.
#[test]
fn plugin_builds_with_orchestration_system() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_plugins(
            RantzStateflowPlugin::new()
                .register_state::<AppState>()
                .register_custom_transition::<TestEffectOut, _, _, _>(
                    test_start_sys,
                    test_run_sys,
                    test_end_sys,
                ),
        );
    app.update(); // Should not panic
}
