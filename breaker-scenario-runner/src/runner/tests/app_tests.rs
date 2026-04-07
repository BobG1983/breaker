//! Tests for `is_timed_out`, `drain_remaining_logs`, `guarded_update`, `snapshot_eval_data`,
//! and `sync_ui_scale`.

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
        SharedEvalBuffer, drain_remaining_logs, guarded_update, is_timed_out, snapshot_eval_data,
        sync_ui_scale,
    },
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
