use bevy::prelude::*;

use super::helpers::*;
use crate::transition::resources::ActiveTransition;

// =======================================================================
// Section K (behavior 31): Deferred ChangeState re-queue
// =======================================================================

#[test]
fn deferred_change_state_is_processed_after_transition_completes() {
    let mut app = transition_test_app();
    // Route A -> B with Out transition, B -> C with no transition
    add_out_route(&mut app, TestState::A, TestState::B);
    add_plain_route(&mut app, TestState::B, TestState::C);
    app.update();

    // Start A->B transition
    send_change_state(&mut app);
    app.update(); // begin Out transition

    // While transition is active, send another ChangeState
    send_change_state(&mut app);

    // Run enough updates for Out to complete + deferred to fire
    for _ in 0..15 {
        app.update();
    }

    // After Out completes (B), the deferred ChangeState fires B->C
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::C,
        "Deferred ChangeState should route B -> C after Out transition completes"
    );
}

// --- Section K (behavior 32): ChangeState during OutIn defers until completion ---

#[test]
fn change_state_during_outin_defers_until_entire_lifecycle_completes() {
    let mut app = transition_test_app();
    add_outin_route(&mut app, TestState::A, TestState::B);
    add_plain_route(&mut app, TestState::B, TestState::C);
    app.update();

    send_change_state(&mut app);
    app.update(); // begin OutIn

    // Send deferred ChangeState during the transition
    send_change_state(&mut app);

    for _ in 0..15 {
        app.update();
    }

    // After OutIn completes (B), deferred fires B->C
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::C,
        "Deferred ChangeState should process after OutIn completes"
    );
}

// =======================================================================
// Section N: Time<Virtual> Pause/Unpause Semantics
// =======================================================================

// --- Behavior 41: Out does not unpause, In pauses (idempotent) then unpauses ---

#[test]
fn out_then_in_paired_sequence_pause_unpause() {
    let mut app = transition_test_app();
    add_out_route(&mut app, TestState::A, TestState::B);
    add_in_route(&mut app, TestState::B, TestState::C);
    app.update();

    // Step 1: Out transition A -> B
    send_change_state(&mut app);
    for _ in 0..10 {
        app.update();
    }

    // After Out: paused, state B
    assert!(
        app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be paused after Out"
    );
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::B,
        "State should be B after Out"
    );
    assert!(
        !app.world().contains_resource::<ActiveTransition>(),
        "ActiveTransition should be removed after Out completes"
    );

    // Step 2: In transition B -> C
    send_change_state(&mut app);
    for _ in 0..10 {
        app.update();
    }

    // After In: unpaused, state C
    assert!(
        !app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be unpaused after In"
    );
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::C,
        "State should be C after In"
    );
}

// --- Behavior 42: OutIn pauses at start and unpauses at end ---

#[test]
fn outin_pauses_at_start_and_unpauses_at_end() {
    let mut app = transition_test_app();
    add_outin_route(&mut app, TestState::A, TestState::B);
    app.update();

    // Before transition: not paused
    assert!(
        !app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should not be paused before transition"
    );

    send_change_state(&mut app);
    app.update(); // begin transition

    // During transition: paused
    assert!(
        app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be paused during OutIn"
    );

    for _ in 0..10 {
        app.update();
    }

    // After completion: unpaused
    assert!(
        !app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be unpaused after OutIn completes"
    );
}

// --- Behavior 43: OneShot pauses at start and unpauses at end ---

#[test]
fn oneshot_pauses_at_start_and_unpauses_at_end() {
    let mut app = transition_test_app();
    add_oneshot_route(&mut app, TestState::A, TestState::B);
    app.update();

    assert!(
        !app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should not be paused before transition"
    );

    send_change_state(&mut app);
    app.update();

    assert!(
        app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be paused during OneShot"
    );

    for _ in 0..10 {
        app.update();
    }

    assert!(
        !app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be unpaused after OneShot completes"
    );
}
