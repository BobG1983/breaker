//! Tests for noop, `SetDashState`, and `SetTimerRemaining` frame mutations.

use super::super::helpers::*;

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — no-op when frame_mutations is None
// -------------------------------------------------------------------------

/// When `frame_mutations` is `None`, `apply_debug_frame_mutations` must
/// do nothing and not panic at any frame.
#[test]
fn apply_debug_frame_mutations_noop_when_none() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        disallowed_failures: vec![],
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    // Must not panic
    app.update();
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SetDashState at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetDashState(Braking)` at frame 3 and
/// the current frame is 3, the breaker entity's `DashState` must
/// become `DashState::Braking`.
#[test]
fn apply_debug_frame_mutations_set_breaker_state_at_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        disallowed_failures: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 3,
            mutation: MutationKind::SetDashState(ScenarioDashState::Braking),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(3))
        .add_systems(Update, apply_debug_frame_mutations);

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, DashState::Idle))
        .id();

    app.update();

    let state = app
        .world()
        .entity(entity)
        .get::<DashState>()
        .expect("entity must still have DashState");
    assert_eq!(
        *state,
        DashState::Braking,
        "expected DashState::Braking at frame 3, got {state:?}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SetDashState does NOT apply at non-matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetDashState(Braking)` at frame 3 but
/// the current frame is 2, the breaker must remain `Idle`.
#[test]
fn apply_debug_frame_mutations_set_breaker_state_skips_non_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        disallowed_failures: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 3,
            mutation: MutationKind::SetDashState(ScenarioDashState::Braking),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(2))
        .add_systems(Update, apply_debug_frame_mutations);

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, DashState::Idle))
        .id();

    app.update();

    let state = app
        .world()
        .entity(entity)
        .get::<DashState>()
        .expect("entity must still have DashState");
    assert_eq!(
        *state,
        DashState::Idle,
        "expected DashState::Idle at frame 2 (mutation at frame 3), got {state:?}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SetTimerRemaining at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetTimerRemaining(61.0)` at frame 5 and
/// the current frame is 5, `NodeTimer.remaining` must be set to 61.0.
#[test]
fn apply_debug_frame_mutations_set_timer_remaining_at_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        disallowed_failures: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 5,
            mutation: MutationKind::SetTimerRemaining(61.0),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .insert_resource(NodeTimer {
            remaining: 55.0,
            total: 60.0,
        })
        .add_systems(Update, apply_debug_frame_mutations);

    app.update();

    let timer = app.world().resource::<NodeTimer>();
    assert!(
        (timer.remaining - 61.0_f32).abs() < f32::EPSILON,
        "expected NodeTimer.remaining == 61.0, got {}",
        timer.remaining
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SetTimerRemaining no-op when no NodeTimer
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetTimerRemaining(61.0)` at frame 5 but
/// no `NodeTimer` resource exists, the system must not panic.
#[test]
fn apply_debug_frame_mutations_set_timer_remaining_noop_when_no_timer() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        disallowed_failures: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 5,
            mutation: MutationKind::SetTimerRemaining(61.0),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    // Deliberately do NOT insert NodeTimer — must not panic
    app.update();
}
