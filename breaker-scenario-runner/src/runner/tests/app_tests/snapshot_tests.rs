//! Tests for `snapshot_eval_data`.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::{
    invariants::{ScenarioStats, ViolationEntry, ViolationLog},
    lifecycle::ScenarioConfig,
    log_capture::CapturedLogs,
    runner::app::{SharedEvalBuffer, snapshot_eval_data},
    types::{InputStrategy, InvariantKind, InvariantParams, ScenarioDefinition, ScriptedParams},
};

// -------------------------------------------------------------------------
// snapshot_eval_data — captures results into shared buffer
// -------------------------------------------------------------------------

#[test]
fn snapshot_eval_data_captures_results_into_shared_buffer() {
    let shared = SharedEvalBuffer(Arc::new(Mutex::new(None)));

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ViolationLog(vec![ViolationEntry {
            frame:     42,
            invariant: InvariantKind::BoltInBounds,
            entity:    None,
            message:   "test violation".into(),
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
