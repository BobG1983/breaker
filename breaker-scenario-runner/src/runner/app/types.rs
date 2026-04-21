//! Shared eval types and the snapshot system that captures them each frame.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::{
    invariants::{ScenarioStats, ViolationEntry, ViolationLog},
    lifecycle::ScenarioConfig,
    log_capture::{CapturedLogs, LogEntry},
    types::ScenarioDefinition,
};

/// Cloned snapshot of evaluation data, captured by a `Last` system so results
/// survive `App::run()` (which replaces self with `App::empty()`).
pub(crate) struct EvalSnapshot {
    pub(crate) violations: Vec<ViolationEntry>,
    pub(crate) logs:       Vec<LogEntry>,
    pub(crate) stats:      ScenarioStats,
    pub(crate) definition: ScenarioDefinition,
}

/// Shared buffer inserted as a resource so the snapshot system can write to it
/// and the caller can read it after `app.run()` returns.
#[derive(Resource, Clone)]
pub(crate) struct SharedEvalBuffer(pub(crate) Arc<Mutex<Option<EvalSnapshot>>>);

/// Snapshots evaluation data every frame into the shared buffer.
///
/// Runs in `Last` so it captures the final state even on the exit frame.
pub(crate) fn snapshot_eval_data(
    vl: Option<Res<ViolationLog>>,
    cl: Option<Res<CapturedLogs>>,
    stats: Option<Res<ScenarioStats>>,
    config: Option<Res<ScenarioConfig>>,
    shared: Res<SharedEvalBuffer>,
) {
    let (Some(vl), Some(cl), Some(stats), Some(config)) = (vl, cl, stats, config) else {
        return;
    };
    if let Ok(mut guard) = shared.0.lock() {
        *guard = Some(EvalSnapshot {
            violations: vl.0.clone(),
            logs:       cl.0.clone(),
            stats:      stats.clone(),
            definition: config.definition.clone(),
        });
    }
}

/// Non-system version of [`snapshot_eval_data`] for direct world access.
///
/// Called after `drain_remaining_logs` in headless mode to capture the final
/// state including any logs drained after the last `FixedUpdate` tick.
pub(crate) fn snapshot_eval_data_from_world(world: &World, shared: &SharedEvalBuffer) {
    let (Some(vl), Some(cl), Some(stats), Some(config)) = (
        world.get_resource::<ViolationLog>(),
        world.get_resource::<CapturedLogs>(),
        world.get_resource::<ScenarioStats>(),
        world.get_resource::<ScenarioConfig>(),
    ) else {
        return;
    };
    if let Ok(mut guard) = shared.0.lock() {
        *guard = Some(EvalSnapshot {
            violations: vl.0.clone(),
            logs:       cl.0.clone(),
            stats:      stats.clone(),
            definition: config.definition.clone(),
        });
    }
}
