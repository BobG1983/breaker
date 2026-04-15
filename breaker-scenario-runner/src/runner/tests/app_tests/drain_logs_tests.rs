//! Tests for `drain_remaining_logs`.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::{
    invariants::ScenarioFrame,
    log_capture::{CapturedLogs, LogBuffer},
    runner::app::drain_remaining_logs,
};

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
