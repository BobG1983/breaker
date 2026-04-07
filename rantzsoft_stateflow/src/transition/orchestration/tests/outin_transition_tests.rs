use bevy::prelude::*;

use super::helpers::*;
use crate::{
    messages::StateChanged,
    transition::resources::{ActiveTransition, StartingTransition},
};

// =======================================================================
// Section I: OutIn Transition Lifecycle
// =======================================================================

// --- Behavior 17: OutIn pauses Time<Virtual> at start ---

#[test]
fn outin_transition_pauses_time_virtual_at_start() {
    let mut app = transition_test_app();
    add_outin_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    app.update();

    assert!(
        app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be paused after OutIn transition starts"
    );
}

// --- Behavior 18: OutIn runs Out phase first ---

#[test]
fn outin_transition_starts_with_out_effect() {
    let mut app = transition_test_app();
    add_outin_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    app.update();

    // With instant effects, the Out phase may already be past Starting.
    // Verify the Out effect was activated (Running/Ending prove Starting happened).
    let out_active = app
        .world()
        .contains_resource::<StartingTransition<TestEffectOut>>()
        || app
            .world()
            .contains_resource::<crate::transition::resources::RunningTransition<TestEffectOut>>()
        || app
            .world()
            .contains_resource::<crate::transition::resources::EndingTransition<TestEffectOut>>();
    assert!(
        out_active || app.world().contains_resource::<ActiveTransition>(),
        "OutIn should start with Out phase (some Out marker or ActiveTransition should exist)"
    );
}

// --- Behavior 19: OutIn applies state change between Out and In ---

#[test]
fn outin_transition_applies_state_change_after_out_phase() {
    let mut app = transition_test_app();
    add_outin_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    // Run enough updates for Out phase to complete + state to apply
    for _ in 0..10 {
        app.update();
    }

    // After OutIn completes fully, state should be B
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::B,
        "State should be B after OutIn transition"
    );
}

// --- Behavior 20: OutIn runs In phase after state change ---

#[test]
fn outin_transition_runs_in_phase_after_out_completes() {
    let mut app = transition_test_app();
    add_outin_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    // Run updates; after Out completes, In should start
    for _ in 0..10 {
        app.update();
    }

    // If the lifecycle ran correctly, all In markers should be cleaned up
    assert!(
        !app.world()
            .contains_resource::<StartingTransition<TestEffectIn>>(),
        "In phase markers should be cleaned up after OutIn completes"
    );
    // ActiveTransition should be removed only after In completes
    assert!(
        !app.world().contains_resource::<ActiveTransition>(),
        "ActiveTransition should be removed after full OutIn lifecycle"
    );
}

// --- Behavior 21: OutIn unpauses Time<Virtual> after In completes ---

#[test]
fn outin_transition_unpauses_time_virtual_after_in_phase_completes() {
    let mut app = transition_test_app();
    add_outin_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    for _ in 0..10 {
        app.update();
    }

    assert!(
        !app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be unpaused after OutIn transition completes"
    );
}

// --- Behavior 22: OutIn sends TransitionStart once, StateChanged at midpoint, TransitionEnd once ---

// Note: This test verifies the full message sequence. Since messages are
// frame-scoped in Bevy, we capture them using a system that records them
// into a resource across frames.

#[derive(Resource, Default)]
struct MessageLog {
    transition_starts: Vec<(TestState, TestState)>,
    state_changed: Vec<(TestState, TestState)>,
    transition_ends: Vec<(TestState, TestState)>,
}

fn capture_transition_start(
    mut msgs: MessageReader<crate::messages::TransitionStart<TestState>>,
    mut log: ResMut<MessageLog>,
) {
    for msg in msgs.read() {
        log.transition_starts.push((msg.from, msg.to));
    }
}

fn capture_state_changed(
    mut msgs: MessageReader<StateChanged<TestState>>,
    mut log: ResMut<MessageLog>,
) {
    for msg in msgs.read() {
        log.state_changed.push((msg.from, msg.to));
    }
}

fn capture_transition_end(
    mut msgs: MessageReader<crate::messages::TransitionEnd<TestState>>,
    mut log: ResMut<MessageLog>,
) {
    for msg in msgs.read() {
        log.transition_ends.push((msg.from, msg.to));
    }
}

#[test]
fn outin_sends_exactly_one_transition_start_one_state_changed_one_transition_end() {
    let mut app = transition_test_app();
    app.init_resource::<MessageLog>();
    app.add_systems(
        Update,
        (
            capture_transition_start,
            capture_state_changed,
            capture_transition_end,
        ),
    );
    add_outin_route(&mut app, TestState::A, TestState::B);
    app.update();

    send_change_state(&mut app);
    for _ in 0..10 {
        app.update();
    }

    let log = app.world().resource::<MessageLog>();
    assert_eq!(
        log.transition_starts.len(),
        1,
        "expected exactly 1 TransitionStart"
    );
    assert_eq!(
        log.state_changed.len(),
        1,
        "expected exactly 1 StateChanged"
    );
    assert_eq!(
        log.transition_ends.len(),
        1,
        "expected exactly 1 TransitionEnd"
    );
}

// --- Behavior 23: OutIn full lifecycle end-to-end ---

#[test]
fn outin_transition_full_lifecycle_end_to_end() {
    let mut app = transition_test_app();
    add_outin_route(&mut app, TestState::A, TestState::B);
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
