//! Tests for noop, `SetBreakerState`, and `SetTimerRemaining` frame mutations.

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
// apply_debug_frame_mutations — SetBreakerState at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetBreakerState(Braking)` at frame 3 and
/// the current frame is 3, the breaker entity's `BreakerState` must
/// become `BreakerState::Braking`.
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
            mutation: MutationKind::SetBreakerState(ScenarioBreakerState::Braking),
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
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();

    app.update();

    let state = app
        .world()
        .entity(entity)
        .get::<BreakerState>()
        .expect("entity must still have BreakerState");
    assert_eq!(
        *state,
        BreakerState::Braking,
        "expected BreakerState::Braking at frame 3, got {state:?}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SetBreakerState does NOT apply at non-matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetBreakerState(Braking)` at frame 3 but
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
            mutation: MutationKind::SetBreakerState(ScenarioBreakerState::Braking),
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
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();

    app.update();

    let state = app
        .world()
        .entity(entity)
        .get::<BreakerState>()
        .expect("entity must still have BreakerState");
    assert_eq!(
        *state,
        BreakerState::Idle,
        "expected BreakerState::Idle at frame 2 (mutation at frame 3), got {state:?}"
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
