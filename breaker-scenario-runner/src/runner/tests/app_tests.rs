//! Tests for `is_timed_out`, `drain_remaining_logs`, `guarded_update`, and `snapshot_eval_data`.

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use bevy::prelude::*;

use super::super::app::{
    SharedEvalBuffer, drain_remaining_logs, guarded_update, is_timed_out, snapshot_eval_data,
};
use crate::{
    invariants::{ScenarioFrame, ScenarioStats, ViolationEntry, ViolationLog},
    lifecycle::ScenarioConfig,
    log_capture::{CapturedLogs, LogBuffer},
    types::InvariantKind,
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
