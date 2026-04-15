//! Scenario run orchestration — builds the app, drives the loop, evaluates results.

use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use bevy::prelude::*;
use tracing::info;

use super::{
    build::build_app,
    evaluate::{
        collect_and_evaluate, drain_remaining_logs, guarded_update, is_timed_out, should_fail_fast,
    },
    types::{SharedEvalBuffer, snapshot_eval_data, snapshot_eval_data_from_world},
};
use crate::{
    invariants::{
        ScenarioFrame, ScenarioName, ScreenshotOutputDir, ScreenshotTracker, ViolationLog,
        capture_violation_screenshots,
    },
    lifecycle::{ScenarioConfig, ScenarioLifecycle},
    log_capture::{CapturedLogs, LogBuffer, poll_log_buffer},
    runner::{
        discovery::{load_scenario, scenario_name},
        run_log::RunLog,
    },
};

/// Builds and runs one scenario app. Returns `true` if passed, `false` if failed.
///
/// The `shared_log_buffer` persists across scenarios so the global tracing
/// subscriber (installed once) always writes to the same buffer that each app's
/// `poll_log_buffer` system reads from.
pub(crate) fn run_scenario(
    path: &Path,
    headless: bool,
    verbose: bool,
    shared_log_buffer: &mut Option<LogBuffer>,
    run_log: Option<&RunLog>,
    fail_fast: bool,
) -> bool {
    let sname = scenario_name(path);

    let Some(definition) = load_scenario(path) else {
        eprintln!("FAIL [{sname}]: could not load scenario file");
        return false;
    };

    let running_line = format!(
        "Running [{sname}] breaker={} layout={}",
        definition.breaker, definition.layout
    );
    println!("{running_line}");
    if let Some(log) = run_log {
        log.write_line(&running_line);
    }
    info!(
        target: "breaker_scenario_runner",
        "scenario start name={sname} breaker={} layout={}",
        definition.breaker, definition.layout
    );

    let first_run = shared_log_buffer.is_none();
    let mut app = build_app(headless, first_run);

    if first_run {
        // Extract the buffer that LogPlugin's factory created so we can reuse it.
        *shared_log_buffer = app.world().get_resource::<LogBuffer>().cloned();
    } else if let Some(buf) = shared_log_buffer {
        // Clear leftover entries and insert the shared buffer so poll_log_buffer
        // reads from the same Arc the global ScenarioLogLayer writes to.
        if let Ok(mut guard) = buf.0.lock() {
            guard.clear();
        }
        app.insert_resource(buf.clone());
    }

    let eval_buffer = SharedEvalBuffer(Arc::new(Mutex::new(None)));

    app.insert_resource(ScenarioConfig { definition })
        .add_plugins(ScenarioLifecycle)
        .init_resource::<CapturedLogs>()
        .insert_resource(eval_buffer.clone())
        .insert_resource(ScenarioName(sname.clone()))
        .add_systems(FixedUpdate, poll_log_buffer)
        .add_systems(
            Last,
            capture_violation_screenshots.run_if(resource_exists::<ScreenshotOutputDir>),
        );

    if !headless {
        // Visual mode needs per-frame snapshots — app.run() replaces self
        // with App::empty(), so Last-schedule is the only capture path.
        app.add_systems(Last, snapshot_eval_data);
    }

    // Screenshots go alongside the log file — fall back to CWD if path has no parent.
    if !headless && let Some(log) = run_log {
        let output_dir = log
            .path()
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        app.insert_resource(ScreenshotOutputDir(output_dir));
        app.init_resource::<ScreenshotTracker>();
    }

    if headless {
        app.finish();
        app.cleanup();

        let wall_clock = Instant::now();
        let timeout = Duration::from_mins(5);

        loop {
            match guarded_update(&mut app) {
                Ok(()) => {}
                Err(msg) => {
                    eprintln!("FAIL [{sname}]: system panic: {msg}");
                    break;
                }
            }
            if let Some(config) = app.world().get_resource::<ScenarioConfig>()
                && let Some(log) = app.world().get_resource::<ViolationLog>()
                && should_fail_fast(log, &config.definition, fail_fast)
            {
                break;
            }
            if app.should_exit().is_some() {
                break;
            }
            if is_timed_out(wall_clock, timeout) {
                let frame = app
                    .world()
                    .get_resource::<ScenarioFrame>()
                    .map_or(0, |f| f.0);
                eprintln!("FAIL [{sname}]: wall-clock timeout ({timeout:?}) at frame {frame}");
                break;
            }
        }

        // Drain any logs emitted after the last FixedUpdate tick.
        // Run one more snapshot to capture the drained logs.
        drain_remaining_logs(&mut app);
        snapshot_eval_data_from_world(app.world(), &eval_buffer);
    } else {
        // Visual mode — Winit needs app.run() for the event loop.
        // App::run() replaces self with App::empty(), losing all resources.
        // The Last-schedule snapshot_eval_data captures results each frame,
        // so the shared buffer holds the final state when run() returns.
        app.run();
    }

    collect_and_evaluate(&eval_buffer, &sname, verbose, run_log)
}
