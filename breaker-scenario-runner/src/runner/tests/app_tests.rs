//! Tests for `is_timed_out`, `drain_remaining_logs`, `guarded_update`, `snapshot_eval_data`,
//! `sync_ui_scale`, `should_fail_fast`, and `collect_and_evaluate`.

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    invariants::{ScenarioFrame, ScenarioStats, ViolationEntry, ViolationLog},
    lifecycle::ScenarioConfig,
    log_capture::{CapturedLogs, LogBuffer},
    runner::app::{
        EvalSnapshot, SharedEvalBuffer, collect_and_evaluate, drain_remaining_logs, guarded_update,
        is_timed_out, should_fail_fast, snapshot_eval_data, sync_ui_scale,
    },
    types::{InputStrategy, InvariantKind, ScenarioDefinition, ScriptedParams},
};

// -------------------------------------------------------------------------
// is_timed_out — returns true when start is in the past beyond timeout
// -------------------------------------------------------------------------

/// A start `Instant` 5 seconds in the past with a 1-second timeout must
/// return `true` from `is_timed_out`.
#[test]
fn is_timed_out_returns_true_when_timeout_exceeded() {
    let start = Instant::now()
        .checked_sub(Duration::from_secs(5))
        .expect("5s subtraction must succeed");
    let timeout = Duration::from_secs(1);

    let result = is_timed_out(start, timeout);

    assert!(
        result,
        "expected is_timed_out to return true when 5s elapsed against a 1s timeout"
    );
}

// -------------------------------------------------------------------------
// is_timed_out — returns false when timeout has not yet elapsed
// -------------------------------------------------------------------------

/// A start `Instant::now()` with a 60-second timeout must return `false`
/// from `is_timed_out` immediately.
#[test]
fn is_timed_out_returns_false_when_timeout_not_exceeded() {
    let start = Instant::now();
    let timeout = Duration::from_mins(1);

    let result = is_timed_out(start, timeout);

    assert!(
        !result,
        "expected is_timed_out to return false when called immediately after start with a 60s timeout"
    );
}

// -------------------------------------------------------------------------
// drain_remaining_logs — transfers buffered entries into CapturedLogs
// -------------------------------------------------------------------------

/// `drain_remaining_logs` must move all entries from `LogBuffer` into
/// `CapturedLogs` with the frame number from `ScenarioFrame`, and leave
/// the buffer empty afterward.
#[test]
fn drain_remaining_logs_transfers_buffered_entries_to_captured_logs() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Populate the LogBuffer with 2 entries before inserting as resource.
    let buffer_entries: Vec<(bevy::log::Level, String, String)> = vec![
        (
            bevy::log::Level::WARN,
            "breaker::test".to_owned(),
            "msg1".to_owned(),
        ),
        (
            bevy::log::Level::ERROR,
            "breaker::test".to_owned(),
            "msg2".to_owned(),
        ),
    ];
    let log_buffer = LogBuffer(Arc::new(Mutex::new(buffer_entries)));
    app.insert_resource(log_buffer)
        .insert_resource(CapturedLogs::default())
        .insert_resource(ScenarioFrame(42));

    drain_remaining_logs(&mut app);

    let captured = app.world().resource::<CapturedLogs>();
    assert_eq!(
        captured.0.len(),
        2,
        "expected 2 captured log entries after drain, got {}",
        captured.0.len()
    );
    assert_eq!(captured.0[0].frame, 42, "expected frame=42 on first entry");
    assert_eq!(captured.0[0].message, "msg1");
    assert_eq!(captured.0[1].message, "msg2");

    let buffer = app.world().resource::<LogBuffer>();
    assert!(
        buffer
            .0
            .lock()
            .expect("lock must not be poisoned")
            .is_empty(),
        "expected LogBuffer to be empty after drain"
    );
}

// -------------------------------------------------------------------------
// guarded_update — returns Err when a system panics
// -------------------------------------------------------------------------

/// `guarded_update` must return `Err` containing the panic message when a
/// registered system calls `panic!("test panic")`.
#[test]
fn guarded_update_returns_err_when_system_panics() {
    fn panicking_system() {
        panic!("test panic");
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, panicking_system);

    let result = guarded_update(&mut app);

    assert!(
        result.is_err(),
        "expected guarded_update to return Err when a system panics"
    );
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("test panic"),
        "expected error message to contain 'test panic', got: {err_msg:?}"
    );
}

// -------------------------------------------------------------------------
// guarded_update — returns Ok when update succeeds
// -------------------------------------------------------------------------

/// `guarded_update` must return `Ok(())` when `app.update()` completes
/// without a panic.
#[test]
fn guarded_update_returns_ok_when_update_succeeds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let result = guarded_update(&mut app);

    assert!(
        result.is_ok(),
        "expected guarded_update to return Ok when update completes normally, got: {result:?}"
    );
}

// -------------------------------------------------------------------------
// snapshot_eval_data — captures results into shared buffer
// -------------------------------------------------------------------------

#[test]
fn snapshot_eval_data_captures_results_into_shared_buffer() {
    use crate::types::{InputStrategy, InvariantParams, ScenarioDefinition, ScriptedParams};

    let shared = SharedEvalBuffer(Arc::new(Mutex::new(None)));

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ViolationLog(vec![ViolationEntry {
            frame: 42,
            invariant: InvariantKind::BoltInBounds,
            entity: None,
            message: "test violation".into(),
        }]))
        .insert_resource(CapturedLogs::default())
        .insert_resource(ScenarioStats {
            actions_injected: 100,
            invariant_checks: 50,
            max_frame: 500,
            entered_playing: true,
            bolts_tagged: 1,
            breakers_tagged: 1,
            ..Default::default()
        })
        .insert_resource(ScenarioConfig {
            definition: ScenarioDefinition {
                breaker: "Aegis".to_owned(),
                layout: "Corridor".to_owned(),
                input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
                max_frames: 1000,
                disallowed_failures: vec![],
                allowed_failures: None,
                debug_setup: None,
                invariant_params: InvariantParams {
                    max_bolt_count: 8,
                    ..InvariantParams::default()
                },
                allow_early_end: true,
                stress: None,
                ..Default::default()
            },
        })
        .insert_resource(shared.clone())
        .add_systems(Last, snapshot_eval_data);

    // Before tick: buffer is None
    assert!(shared.0.lock().unwrap().is_none());

    // Tick once to run the Last system
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();

    // After tick: buffer has the snapshot
    let snapshot = shared
        .0
        .lock()
        .unwrap()
        .take()
        .expect("snapshot should be Some after tick");
    assert_eq!(snapshot.violations.len(), 1);
    assert_eq!(snapshot.violations[0].frame, 42);
    assert_eq!(snapshot.stats.max_frame, 500);
    assert_eq!(snapshot.stats.actions_injected, 100);
}

// =========================================================================
// sync_ui_scale — sets UiScale from primary window dimensions
// =========================================================================

/// Helper: builds a minimal test app with `sync_ui_scale` in `Update`.
///
/// Initializes `UiScale` to a sentinel value (99.0) so we can verify the
/// system actively writes the correct value. Without this, tests where the
/// expected output happens to equal the default (1.0) would pass against
/// a no-op stub.
fn sync_ui_scale_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(UiScale(99.0));
    app.add_systems(Update, sync_ui_scale);
    app
}

/// Spawns a `Window` entity with the given resolution and a `PrimaryWindow` marker.
fn spawn_primary_window(app: &mut App, width: u32, height: u32) {
    app.world_mut().spawn((
        Window {
            resolution: bevy::window::WindowResolution::new(width, height),
            ..default()
        },
        PrimaryWindow,
    ));
}

// -------------------------------------------------------------------------
// Behavior 20: full HD window produces ui_scale 1.0
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_full_hd_window_produces_scale_1() {
    let mut app = sync_ui_scale_app();
    spawn_primary_window(&mut app, 1920, 1080);

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    assert!(
        (ui_scale.0 - 1.0).abs() < f32::EPSILON,
        "expected UiScale ~1.0 for 1920x1080, got {}",
        ui_scale.0
    );
}

// -------------------------------------------------------------------------
// Behavior 21: half-size window produces ui_scale 0.5
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_half_size_window_produces_scale_0_5() {
    let mut app = sync_ui_scale_app();
    spawn_primary_window(&mut app, 960, 540);

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    assert!(
        (ui_scale.0 - 0.5).abs() < f32::EPSILON,
        "expected UiScale ~0.5 for 960x540, got {}",
        ui_scale.0
    );
}

// -------------------------------------------------------------------------
// Behavior 22: width-limited window uses width ratio
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_width_limited_window_uses_width_ratio() {
    let mut app = sync_ui_scale_app();
    spawn_primary_window(&mut app, 960, 1080);

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    // min(960/1920, 1080/1080) = min(0.5, 1.0) = 0.5
    assert!(
        (ui_scale.0 - 0.5).abs() < f32::EPSILON,
        "expected UiScale ~0.5 for width-limited 960x1080, got {}",
        ui_scale.0
    );
}

// -------------------------------------------------------------------------
// Behavior 23: height-limited window uses height ratio
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_height_limited_window_uses_height_ratio() {
    let mut app = sync_ui_scale_app();
    spawn_primary_window(&mut app, 1920, 540);

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    // min(1920/1920, 540/1080) = min(1.0, 0.5) = 0.5
    assert!(
        (ui_scale.0 - 0.5).abs() < f32::EPSILON,
        "expected UiScale ~0.5 for height-limited 1920x540, got {}",
        ui_scale.0
    );
}

// -------------------------------------------------------------------------
// Behavior 24: 4K window produces ui_scale 2.0
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_4k_window_produces_scale_2() {
    let mut app = sync_ui_scale_app();
    spawn_primary_window(&mut app, 3840, 2160);

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    // min(3840/1920, 2160/1080) = min(2.0, 2.0) = 2.0
    assert!(
        (ui_scale.0 - 2.0).abs() < f32::EPSILON,
        "expected UiScale ~2.0 for 3840x2160, got {}",
        ui_scale.0
    );
}

// -------------------------------------------------------------------------
// Behavior 25: ultrawide window limited by smaller ratio
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_ultrawide_window_limited_by_height() {
    let mut app = sync_ui_scale_app();
    spawn_primary_window(&mut app, 3840, 1080);

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    // min(3840/1920, 1080/1080) = min(2.0, 1.0) = 1.0
    assert!(
        (ui_scale.0 - 1.0).abs() < f32::EPSILON,
        "expected UiScale ~1.0 for ultrawide 3840x1080, got {}",
        ui_scale.0
    );
}

// -------------------------------------------------------------------------
// Behavior 26: no primary window does not panic, UiScale unchanged
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_no_primary_window_does_not_panic_and_leaves_ui_scale_unchanged() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Insert UiScale manually per spec: "UiScale inserted manually with value 1.0"
    app.insert_resource(UiScale(1.0));
    app.add_systems(Update, sync_ui_scale);
    // No window spawned — the system should silently return.

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    assert!(
        (ui_scale.0 - 1.0).abs() < f32::EPSILON,
        "expected UiScale to remain 1.0 when no primary window exists, got {}",
        ui_scale.0
    );
}

// =========================================================================
// should_fail_fast — pure function tests
// =========================================================================

/// Helper: builds a minimal `ScenarioDefinition` with the given `allowed_failures`.
fn definition_with_allowed_failures(
    allowed_failures: Option<Vec<InvariantKind>>,
) -> ScenarioDefinition {
    ScenarioDefinition {
        breaker: "test".into(),
        layout: "test".into(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 100,
        disallowed_failures: vec![],
        allowed_failures,
        ..Default::default()
    }
}

/// Helper: builds a `ViolationLog` with the given entries.
fn violation_log_with(entries: Vec<ViolationEntry>) -> ViolationLog {
    ViolationLog(entries)
}

/// Helper: builds a single `ViolationEntry` for `BoltInBounds` at the given frame.
fn bolt_oob_violation(frame: u32) -> ViolationEntry {
    ViolationEntry {
        frame,
        invariant: InvariantKind::BoltInBounds,
        entity: None,
        message: "bolt OOB".into(),
    }
}

// -------------------------------------------------------------------------
// Behavior 5: returns true when fail-fast active, violations exist, no
//             allowed_failures
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_true_when_active_with_violations_and_no_allowed_failures() {
    let log = violation_log_with(vec![bolt_oob_violation(5)]);
    let definition = definition_with_allowed_failures(None);

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        result,
        "should_fail_fast must return true when fail_fast=true, violations exist, and allowed_failures=None"
    );
}

// -------------------------------------------------------------------------
// Behavior 5 edge: allowed_failures = Some(vec![]) also returns true
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_true_when_allowed_failures_is_empty_vec() {
    let log = violation_log_with(vec![bolt_oob_violation(5)]);
    let definition = definition_with_allowed_failures(Some(vec![]));

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        result,
        "should_fail_fast must return true when allowed_failures=Some(vec![]) because empty vec means no expected violations"
    );
}

// -------------------------------------------------------------------------
// Behavior 6: returns false when all violations are in allowed_failures
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_false_when_violation_is_in_allowed_failures() {
    let log = violation_log_with(vec![ViolationEntry {
        frame: 10,
        invariant: InvariantKind::BoltInBounds,
        entity: None,
        message: "expected violation".into(),
    }]);
    let definition = definition_with_allowed_failures(Some(vec![InvariantKind::BoltInBounds]));

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        !result,
        "should_fail_fast must return false when violation is in allowed_failures (expected self-test violation)"
    );
}

// -------------------------------------------------------------------------
// Behavior 6 edge: multiple allowed_failures covering the violation
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_false_when_violation_covered_by_multiple_allowed() {
    let log = violation_log_with(vec![bolt_oob_violation(10)]);
    let definition = definition_with_allowed_failures(Some(vec![
        InvariantKind::BoltInBounds,
        InvariantKind::NoNaN,
    ]));

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        !result,
        "should_fail_fast must return false when violation is in allowed_failures list"
    );
}

// -------------------------------------------------------------------------
// Behavior 6 edge: disallowed violation in self-test triggers fail-fast
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_true_when_violation_not_in_allowed_failures() {
    // Self-test allows BoltInBounds but gets NoNaN — should fail-fast
    let log = violation_log_with(vec![ViolationEntry {
        frame: 10,
        invariant: InvariantKind::NoNaN,
        entity: None,
        message: "unexpected NaN".into(),
    }]);
    let definition = definition_with_allowed_failures(Some(vec![InvariantKind::BoltInBounds]));

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        result,
        "should_fail_fast must return true when violation is NOT in allowed_failures (disallowed violation in self-test)"
    );
}

// -------------------------------------------------------------------------
// Behavior 6 edge: mixed allowed and disallowed violations triggers fail-fast
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_true_when_any_violation_not_in_allowed_failures() {
    // Self-test allows BoltInBounds, gets both BoltInBounds (allowed) and NoNaN (disallowed)
    let log = violation_log_with(vec![
        bolt_oob_violation(5),
        ViolationEntry {
            frame: 10,
            invariant: InvariantKind::NoNaN,
            entity: None,
            message: "unexpected NaN".into(),
        },
    ]);
    let definition = definition_with_allowed_failures(Some(vec![InvariantKind::BoltInBounds]));

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        result,
        "should_fail_fast must return true when any violation is NOT in allowed_failures"
    );
}

// -------------------------------------------------------------------------
// Behavior 7: returns false when fail_fast flag is false
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_false_when_flag_is_false() {
    let log = violation_log_with(vec![bolt_oob_violation(5)]);
    let definition = definition_with_allowed_failures(None);

    let result = should_fail_fast(&log, &definition, false);

    assert!(
        !result,
        "should_fail_fast must return false when fail_fast=false regardless of violations"
    );
}

// -------------------------------------------------------------------------
// Behavior 7 edge: multiple violations, still false when flag is false
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_false_with_multiple_violations_when_flag_is_false() {
    let log = violation_log_with(vec![bolt_oob_violation(5), bolt_oob_violation(10)]);
    let definition = definition_with_allowed_failures(None);

    let result = should_fail_fast(&log, &definition, false);

    assert!(
        !result,
        "should_fail_fast must return false when fail_fast=false even with multiple violations"
    );
}

// -------------------------------------------------------------------------
// Behavior 8: returns false when ViolationLog is empty
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_false_when_violation_log_is_empty() {
    let log = violation_log_with(vec![]);
    let definition = definition_with_allowed_failures(None);

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        !result,
        "should_fail_fast must return false when violation log is empty even with fail_fast=true"
    );
}

// -------------------------------------------------------------------------
// Behavior 8 edge: empty log + empty allowed_failures still returns false
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_false_when_log_empty_and_allowed_failures_empty() {
    let log = violation_log_with(vec![]);
    let definition = definition_with_allowed_failures(Some(vec![]));

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        !result,
        "should_fail_fast must return false when log is empty (empty log takes precedence)"
    );
}

// =========================================================================
// collect_and_evaluate — contract test for early-exit data
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 9: collect_and_evaluate processes violations normally after
//             fail-fast early exit (snapshot with violations, low frame count)
// -------------------------------------------------------------------------

#[test]
fn collect_and_evaluate_reports_failure_for_early_exit_snapshot_with_violations() {
    let definition = ScenarioDefinition {
        breaker: "test".into(),
        layout: "test".into(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 20000,
        disallowed_failures: vec![],
        allowed_failures: None,
        ..Default::default()
    };
    let stats = ScenarioStats {
        actions_injected: 0,
        invariant_checks: 5,
        max_frame: 5,
        entered_playing: true,
        bolts_tagged: 1,
        breakers_tagged: 1,
        ..Default::default()
    };
    let snapshot = EvalSnapshot {
        violations: vec![ViolationEntry {
            frame: 5,
            invariant: InvariantKind::BoltInBounds,
            entity: None,
            message: "bolt OOB at (999.0, 0.0)".into(),
        }],
        logs: vec![],
        stats,
        definition,
    };
    let buffer = SharedEvalBuffer(Arc::new(Mutex::new(Some(snapshot))));

    let passed = collect_and_evaluate(&buffer, "early_exit_test", false, None);

    assert!(
        !passed,
        "collect_and_evaluate must report failure when snapshot contains violations from early exit"
    );
}
