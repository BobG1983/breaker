//! Tests for `NodeTimer` override via `apply_debug_setup`.

use crate::lifecycle::tests::helpers::*;

// -------------------------------------------------------------------------
// apply_debug_setup — sets NodeTimer.remaining
// -------------------------------------------------------------------------

/// When `node_timer_remaining: Some(-1.0)`, `apply_debug_setup` must set
/// `NodeTimer.remaining` to -1.0.
#[test]
fn apply_debug_setup_sets_node_timer_remaining() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        disallowed_failures: vec![],
        debug_setup: Some(DebugSetup {
            node_timer_remaining: Some(-1.0),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);
    app.world_mut().insert_resource(NodeTimer {
        remaining: 60.0,
        total: 60.0,
    });

    app.update();
    app.update();

    let timer = app.world().resource::<NodeTimer>();
    assert!(
        (timer.remaining - (-1.0_f32)).abs() < f32::EPSILON,
        "expected NodeTimer.remaining == -1.0, got {}",
        timer.remaining
    );
}

// -------------------------------------------------------------------------
// apply_debug_setup — ignores node_timer_remaining when no NodeTimer
// -------------------------------------------------------------------------

/// When `node_timer_remaining: Some(-1.0)` but no `NodeTimer` resource is
/// present, `apply_debug_setup` must not panic.
#[test]
fn apply_debug_setup_ignores_node_timer_remaining_when_no_resource() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        disallowed_failures: vec![],
        debug_setup: Some(DebugSetup {
            node_timer_remaining: Some(-1.0),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);
    // Deliberately do NOT insert NodeTimer

    // Must not panic
    app.update();
    app.update();
}
