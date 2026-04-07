use bevy::prelude::*;

use super::helpers::*;
use crate::transition::resources::{
    ActiveTransition, EndingTransition, RunningTransition, StartingTransition,
};

// =======================================================================
// Section J: OneShot Transition Lifecycle
// =======================================================================

// --- Behavior 24: OneShot pauses Time<Virtual> at start ---

#[test]
fn oneshot_transition_pauses_time_virtual_at_start() {
    let mut app = transition_test_app();
    add_oneshot_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    app.update();

    assert!(
        app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be paused after OneShot transition starts"
    );
}

// Time<Real> has no is_paused() in Bevy — Real time is never paused.
// The guarantee that Time<Real> is unaffected is by Bevy design.

// --- Behavior 25: OneShot applies state change before effect starts ---

#[test]
fn oneshot_transition_applies_state_change_before_effect() {
    let mut app = transition_test_app();
    add_oneshot_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    app.update(); // begin_transition sets NextState + sends StateChanged
    app.update(); // Bevy applies state change

    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::B,
        "OneShot should change state BEFORE the effect lifecycle"
    );
}

// --- Behavior 26: OneShot runs full Starting -> Running -> Ending lifecycle ---

#[test]
fn oneshot_transition_runs_full_lifecycle() {
    let mut app = transition_test_app();
    add_oneshot_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    for _ in 0..10 {
        app.update();
    }

    // All marker resources should be cleaned up
    assert!(
        !app.world()
            .contains_resource::<StartingTransition<TestEffectOneShot>>(),
        "StartingTransition<TestEffectOneShot> should be cleaned up"
    );
    assert!(
        !app.world()
            .contains_resource::<RunningTransition<TestEffectOneShot>>(),
        "RunningTransition<TestEffectOneShot> should be cleaned up"
    );
    assert!(
        !app.world()
            .contains_resource::<EndingTransition<TestEffectOneShot>>(),
        "EndingTransition<TestEffectOneShot> should be cleaned up"
    );
    // ActiveTransition should persist during the effect then be removed
    assert!(
        !app.world().contains_resource::<ActiveTransition>(),
        "ActiveTransition should be removed after full OneShot lifecycle"
    );
}

// --- Behavior 27: OneShot unpauses Time<Virtual> after completion ---

#[test]
fn oneshot_transition_unpauses_time_virtual_after_completion() {
    let mut app = transition_test_app();
    add_oneshot_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    for _ in 0..10 {
        app.update();
    }

    assert!(
        !app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be unpaused after OneShot transition completes"
    );
}

// --- Behavior 28: OneShot full lifecycle end-to-end ---

#[test]
fn oneshot_transition_full_lifecycle_end_to_end() {
    let mut app = transition_test_app();
    add_oneshot_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    for _ in 0..10 {
        app.update();
    }

    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::B,
        "state should be B"
    );
    assert!(
        !app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be unpaused"
    );
    assert!(
        !app.world().contains_resource::<ActiveTransition>(),
        "ActiveTransition should be removed"
    );
}
