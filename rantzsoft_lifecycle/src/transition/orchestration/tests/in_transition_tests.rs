use bevy::prelude::*;

use super::helpers::*;
use crate::transition::resources::{
    ActiveTransition, EndingTransition, RunningTransition, StartingTransition,
};

// =======================================================================
// Section H: In Transition Lifecycle
// =======================================================================

// --- Behavior 13: In transition pauses Time<Virtual> (idempotent) ---

#[test]
fn in_transition_pauses_time_virtual_at_start() {
    let mut app = transition_test_app();
    add_in_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    app.update();

    assert!(
        app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be paused after In transition starts"
    );
}

#[test]
fn in_transition_pause_is_idempotent_when_already_paused() {
    let mut app = transition_test_app();
    add_in_route(&mut app, TestState::A, TestState::B);
    app.update();

    // Pre-pause Time<Virtual> (simulating it was already paused by a preceding Out)
    app.world_mut().resource_mut::<Time<Virtual>>().pause();

    send_change_state(&mut app);
    app.update(); // should not error, stays paused

    assert!(
        app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should remain paused (idempotent)"
    );
}

// --- Behavior 14a: In transition sets state BEFORE inserting StartingTransition ---

#[test]
fn in_transition_sets_state_before_starting_transition() {
    let mut app = transition_test_app();
    add_in_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    app.update(); // dispatch + begin_transition: sets NextState, sends StateChanged
    app.update(); // Bevy applies state change

    // State should already be B before the effect lifecycle begins
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::B,
        "In transition should change state BEFORE the effect starts"
    );
}

// --- Behavior 14b: In runs full lifecycle after state change ---

#[test]
fn in_transition_runs_full_lifecycle_after_state_change() {
    let mut app = transition_test_app();
    add_in_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    for _ in 0..10 {
        app.update();
    }

    // All marker resources should have been inserted and removed
    assert!(
        !app.world()
            .contains_resource::<StartingTransition<TestEffectIn>>(),
        "StartingTransition<TestEffectIn> should be cleaned up"
    );
    assert!(
        !app.world()
            .contains_resource::<RunningTransition<TestEffectIn>>(),
        "RunningTransition<TestEffectIn> should be cleaned up"
    );
    assert!(
        !app.world()
            .contains_resource::<EndingTransition<TestEffectIn>>(),
        "EndingTransition<TestEffectIn> should be cleaned up"
    );
    assert!(
        !app.world().contains_resource::<ActiveTransition>(),
        "ActiveTransition should be removed after In lifecycle completes"
    );
}

// --- Behavior 15: In unpauses Time<Virtual> and sends TransitionEnd ---

#[test]
fn in_transition_unpauses_time_virtual_after_effect_ends() {
    let mut app = transition_test_app();
    add_in_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    for _ in 0..10 {
        app.update();
    }

    assert!(
        !app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be unpaused after In transition completes"
    );
}

#[test]
fn in_transition_sends_transition_end_after_completion() {
    let mut app = transition_test_app();
    add_in_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    for _ in 0..10 {
        app.update();
    }

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<crate::messages::TransitionEnd<TestState>>>();
    assert!(
        msgs.iter_current_update_messages().next().is_some(),
        "TransitionEnd<TestState> should be sent after In transition completes"
    );
}

#[test]
fn in_transition_removes_active_transition_after_completion() {
    let mut app = transition_test_app();
    add_in_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    for _ in 0..10 {
        app.update();
    }

    assert!(
        !app.world().contains_resource::<ActiveTransition>(),
        "ActiveTransition should be removed after In lifecycle completes"
    );
}
