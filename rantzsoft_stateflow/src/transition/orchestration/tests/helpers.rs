use std::sync::Arc;

use bevy::{prelude::*, state::app::StatesPlugin};

use crate::{
    RantzStateflowPlugin, Route,
    messages::ChangeState,
    transition::{
        messages::{TransitionOver, TransitionReady, TransitionRunComplete},
        traits::{InTransition, OneShotTransition, OutTransition, Transition},
        types::TransitionType,
    },
};

// ---- Test effect types ----

pub(super) struct TestEffectOut;
impl Transition for TestEffectOut {}
impl OutTransition for TestEffectOut {}

pub(super) struct TestEffectIn;
impl Transition for TestEffectIn {}
impl InTransition for TestEffectIn {}

pub(super) struct TestEffectOneShot;
impl Transition for TestEffectOneShot {}
impl OneShotTransition for TestEffectOneShot {}

// ---- Test state enum ----

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum TestState {
    #[default]
    A,
    B,
    C,
}

// ---- Test effect systems (instant completion) ----
// These are test infrastructure, NOT stubs. They immediately send the
// phase-completion message so orchestration tests can validate the full
// lifecycle in a small number of update calls.

fn test_effect_out_start(mut writer: MessageWriter<TransitionReady>) {
    writer.write(TransitionReady);
}

fn test_effect_out_run(mut writer: MessageWriter<TransitionRunComplete>) {
    writer.write(TransitionRunComplete);
}

fn test_effect_out_end(mut writer: MessageWriter<TransitionOver>) {
    writer.write(TransitionOver);
}

fn test_effect_in_start(mut writer: MessageWriter<TransitionReady>) {
    writer.write(TransitionReady);
}

fn test_effect_in_run(mut writer: MessageWriter<TransitionRunComplete>) {
    writer.write(TransitionRunComplete);
}

fn test_effect_in_end(mut writer: MessageWriter<TransitionOver>) {
    writer.write(TransitionOver);
}

fn test_effect_oneshot_start(mut writer: MessageWriter<TransitionReady>) {
    writer.write(TransitionReady);
}

fn test_effect_oneshot_run(mut writer: MessageWriter<TransitionRunComplete>) {
    writer.write(TransitionRunComplete);
}

fn test_effect_oneshot_end(mut writer: MessageWriter<TransitionOver>) {
    writer.write(TransitionOver);
}

// ---- Test app helper ----

pub(super) fn transition_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<TestState>()
        .add_plugins(
            RantzStateflowPlugin::new()
                .register_state::<TestState>()
                .register_custom_transition::<TestEffectOut, _, _, _>(
                    test_effect_out_start,
                    test_effect_out_run,
                    test_effect_out_end,
                )
                .register_custom_transition::<TestEffectIn, _, _, _>(
                    test_effect_in_start,
                    test_effect_in_run,
                    test_effect_in_end,
                )
                .register_custom_transition::<TestEffectOneShot, _, _, _>(
                    test_effect_oneshot_start,
                    test_effect_oneshot_run,
                    test_effect_oneshot_end,
                ),
        );
    app
}

pub(super) fn send_change_state(app: &mut App) {
    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<ChangeState<TestState>>>()
        .write(ChangeState::new());
}

pub(super) fn add_out_route(app: &mut App, from: TestState, to: TestState) {
    app.world_mut()
        .resource_mut::<crate::routing_table::RoutingTable<TestState>>()
        .add(
            Route::from(from)
                .to(to)
                .with_transition(TransitionType::Out(Arc::new(TestEffectOut))),
        )
        .ok();
}

pub(super) fn add_in_route(app: &mut App, from: TestState, to: TestState) {
    app.world_mut()
        .resource_mut::<crate::routing_table::RoutingTable<TestState>>()
        .add(
            Route::from(from)
                .to(to)
                .with_transition(TransitionType::In(Arc::new(TestEffectIn))),
        )
        .ok();
}

pub(super) fn add_outin_route(app: &mut App, from: TestState, to: TestState) {
    app.world_mut()
        .resource_mut::<crate::routing_table::RoutingTable<TestState>>()
        .add(
            Route::from(from)
                .to(to)
                .with_transition(TransitionType::OutIn {
                    out_e: Arc::new(TestEffectOut),
                    in_e: Arc::new(TestEffectIn),
                }),
        )
        .ok();
}

pub(super) fn add_oneshot_route(app: &mut App, from: TestState, to: TestState) {
    app.world_mut()
        .resource_mut::<crate::routing_table::RoutingTable<TestState>>()
        .add(
            Route::from(from)
                .to(to)
                .with_transition(TransitionType::OneShot(Arc::new(TestEffectOneShot))),
        )
        .ok();
}

pub(super) fn add_plain_route(app: &mut App, from: TestState, to: TestState) {
    app.world_mut()
        .resource_mut::<crate::routing_table::RoutingTable<TestState>>()
        .add(Route::from(from).to(to))
        .ok();
}
