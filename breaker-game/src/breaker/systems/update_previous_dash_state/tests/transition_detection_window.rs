//! Behaviors 6–9: ordering-window tests for `update_previous_dash_state`.
//!
//! These tests build a minimal `FixedUpdate` `App` that does NOT load
//! `BreakerPlugin`. They register three (or four) systems with explicit set
//! membership and ordering to validate the
//! `BreakerSystems::UpdateState` → (window) → `BreakerSystems::UpdatePreviousState`
//! contract that Wave 5 (Reckless Dash) will rely on.

use bevy::prelude::*;

use super::super::update_previous_dash_state;
use crate::{
    breaker::{
        components::{DashState, PreviousDashState},
        sets::BreakerSystems,
    },
    prelude::*,
};

// ── Plumbing ────────────────────────────────────────────────────────────────

/// Drives a forced `DashState` mutation during the `BreakerSystems::UpdateState`
/// slot. Stand-in for `update_breaker_state` in tests.
#[derive(Resource, Default)]
struct ForceTransition(Option<DashState>);

fn force_transition_system(force: Res<ForceTransition>, mut q: Query<&mut DashState>) {
    if let Some(target) = force.0 {
        for mut state in &mut q {
            *state = target;
        }
    }
}

/// Records what `detect_system` saw in the window between
/// `BreakerSystems::UpdateState` and `BreakerSystems::UpdatePreviousState`.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
struct DetectedTransition {
    seen_dash:     Option<DashState>,
    seen_previous: Option<DashState>,
    ran:           bool,
}

fn detect_system(
    mut detected: ResMut<DetectedTransition>,
    q: Query<(&DashState, &PreviousDashState)>,
) {
    detected.ran = true;
    for (state, prev) in &q {
        detected.seen_dash = Some(*state);
        detected.seen_previous = Some(prev.0);
    }
}

/// Same shape as `DetectedTransition` but written by `late_detect_system`,
/// which runs `.after(BreakerSystems::UpdatePreviousState)` for Behavior 9.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
struct LateDetectedTransition {
    seen_dash:     Option<DashState>,
    seen_previous: Option<DashState>,
    ran:           bool,
}

fn late_detect_system(
    mut detected: ResMut<LateDetectedTransition>,
    q: Query<(&DashState, &PreviousDashState)>,
) {
    detected.ran = true;
    for (state, prev) in &q {
        detected.seen_dash = Some(*state);
        detected.seen_previous = Some(prev.0);
    }
}

/// Builds a minimal `FixedUpdate` test app without `BreakerPlugin`. Registers
/// `force_transition_system` (`UpdateState`), `update_previous_dash_state`
/// (`UpdatePreviousState`, `.after(UpdateState)`), and `detect_system`
/// (`.after(UpdateState).before(UpdatePreviousState)`). Inserts default
/// `ForceTransition` and `DetectedTransition` resources.
fn window_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<ForceTransition>();
    app.init_resource::<DetectedTransition>();

    app.configure_sets(
        FixedUpdate,
        BreakerSystems::UpdateState.before(BreakerSystems::UpdatePreviousState),
    );

    app.add_systems(
        FixedUpdate,
        (
            force_transition_system.in_set(BreakerSystems::UpdateState),
            update_previous_dash_state
                .in_set(BreakerSystems::UpdatePreviousState)
                .after(BreakerSystems::UpdateState),
            detect_system
                .after(BreakerSystems::UpdateState)
                .before(BreakerSystems::UpdatePreviousState),
        ),
    );

    app
}

/// Same as [`window_test_app`] but additionally registers `late_detect_system`
/// scheduled `.after(BreakerSystems::UpdatePreviousState)` and inserts a
/// `LateDetectedTransition` resource. Used by Behavior 9.
fn window_test_app_with_late_detect() -> App {
    let mut app = window_test_app();
    app.init_resource::<LateDetectedTransition>();
    app.add_systems(
        FixedUpdate,
        late_detect_system.after(BreakerSystems::UpdatePreviousState),
    );
    app
}

// ── Behavior 6: Idle → Dashing transition is observable in the window ──

#[test]
fn idle_to_dashing_transition_visible_in_window() {
    let mut app = window_test_app();

    let entity = app
        .world_mut()
        .spawn((DashState::Idle, PreviousDashState(DashState::Idle)))
        .id();

    app.world_mut().resource_mut::<ForceTransition>().0 = Some(DashState::Dashing);

    tick(&mut app);

    let detected = *app.world().resource::<DetectedTransition>();
    assert!(
        detected.ran,
        "detect_system must have run during the FixedUpdate tick",
    );
    assert_eq!(
        detected.seen_dash,
        Some(DashState::Dashing),
        "detect_system should have seen the NEW DashState (Dashing) written by force_transition_system",
    );
    assert_eq!(
        detected.seen_previous,
        Some(DashState::Idle),
        "detect_system should have seen the OLD PreviousDashState (Idle) before update_previous_dash_state ran",
    );

    // After the tick, update_previous_dash_state has run and overwritten
    // PreviousDashState with the now-current DashState.
    let prev = app.world().get::<PreviousDashState>(entity).unwrap();
    assert_eq!(
        prev.0,
        DashState::Dashing,
        "post-tick PreviousDashState.0 should be Dashing (overwritten by update_previous_dash_state)",
    );

    // Edge case: DashState should still be Dashing — none of the systems
    // should have reverted it.
    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(
        *state,
        DashState::Dashing,
        "post-tick DashState should remain Dashing",
    );
}

// ── Behavior 7: Dashing → Braking transition is observable in the window;
//                 second tick with no force shows no transition ──

#[test]
fn dashing_to_braking_transition_visible_in_window_then_no_transition_next_tick() {
    let mut app = window_test_app();

    let entity = app
        .world_mut()
        .spawn((DashState::Dashing, PreviousDashState(DashState::Dashing)))
        .id();

    app.world_mut().resource_mut::<ForceTransition>().0 = Some(DashState::Braking);

    tick(&mut app);

    {
        let detected = *app.world().resource::<DetectedTransition>();
        assert!(
            detected.ran,
            "detect_system must have run during the first tick",
        );
        assert_eq!(
            detected.seen_dash,
            Some(DashState::Braking),
            "detect_system should have seen NEW DashState Braking",
        );
        assert_eq!(
            detected.seen_previous,
            Some(DashState::Dashing),
            "detect_system should have seen OLD PreviousDashState Dashing",
        );
    }

    let prev = app.world().get::<PreviousDashState>(entity).unwrap();
    assert_eq!(
        prev.0,
        DashState::Braking,
        "post-tick PreviousDashState.0 should be Braking",
    );

    // Edge case: second tick with no forced transition. detect_system should
    // see equal current/previous (both Braking) — no transition observed.
    app.world_mut().resource_mut::<ForceTransition>().0 = None;

    tick(&mut app);

    let detected = *app.world().resource::<DetectedTransition>();
    assert_eq!(
        detected.seen_dash,
        Some(DashState::Braking),
        "second tick: detect_system should see DashState Braking (unchanged)",
    );
    assert_eq!(
        detected.seen_previous,
        Some(DashState::Braking),
        "second tick: detect_system should see PreviousDashState Braking (no transition)",
    );
}

// ── Behavior 8: Non-transition tick — window shows equal current and previous ──

#[test]
fn non_transition_tick_shows_equal_current_and_previous() {
    let mut app = window_test_app();

    let entity = app
        .world_mut()
        .spawn((DashState::Idle, PreviousDashState(DashState::Braking)))
        .id();

    // ForceTransition stays at default (None) — no forced mutation.
    // Tick 1 primes PreviousDashState; tick 2 is the non-transition tick.
    tick(&mut app);
    tick(&mut app);

    let detected = *app.world().resource::<DetectedTransition>();
    assert!(
        detected.ran,
        "detect_system must have run during the FixedUpdate tick",
    );
    assert_eq!(
        detected.seen_dash,
        Some(DashState::Idle),
        "detect_system should see DashState Idle on a non-transition tick",
    );
    assert_eq!(
        detected.seen_previous,
        Some(DashState::Idle),
        "detect_system should see PreviousDashState Idle on a non-transition tick",
    );

    let prev = app.world().get::<PreviousDashState>(entity).unwrap();
    assert_eq!(
        prev.0,
        DashState::Idle,
        "post-tick PreviousDashState.0 should remain Idle",
    );
}

// ── Behavior 9: update_previous_dash_state is in the UpdatePreviousState set
//                 (a system .after(UpdatePreviousState) sees overwritten previous) ──

#[test]
fn late_system_after_updateprevious_sees_overwritten_previous() {
    let mut app = window_test_app_with_late_detect();

    let _entity = app
        .world_mut()
        .spawn((DashState::Idle, PreviousDashState(DashState::Idle)))
        .id();

    app.world_mut().resource_mut::<ForceTransition>().0 = Some(DashState::Dashing);

    tick(&mut app);

    let late = *app.world().resource::<LateDetectedTransition>();
    assert!(
        late.ran,
        "late_detect_system must have run during the FixedUpdate tick",
    );
    assert_eq!(
        late.seen_dash,
        Some(DashState::Dashing),
        "late_detect_system should see DashState Dashing",
    );
    assert_eq!(
        late.seen_previous,
        Some(DashState::Dashing),
        "late_detect_system should see PreviousDashState Dashing — \
         update_previous_dash_state must have run BEFORE late_detect_system, \
         confirming it is in the UpdatePreviousState set",
    );
}
