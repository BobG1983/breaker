//! App construction and multi-scenario execution.
//!
//! Builds either a visual or headless [`App`] for each scenario and runs it to
//! completion, then prints a structured summary and returns the exit code.

use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    log::LogPlugin, prelude::*, time::TimeUpdateStrategy, window::ExitCondition, winit::WinitPlugin,
};
use breaker::game::Game;
use tracing::{debug, info, warn};

use crate::{
    invariants::{ScenarioFrame, ScenarioStats, ViolationLog},
    lifecycle::{ScenarioConfig, ScenarioLifecycle},
    log_capture::{CapturedLogs, LogBuffer, LogEntry, poll_log_buffer, scenario_log_layer_factory},
    types::ScenarioDefinition,
    verdict::ScenarioVerdict,
};

/// Entry point called by `main`. Returns process exit code (0 = all pass, 1 = any fail).
#[must_use]
pub fn run_with_args(scenario: Option<&str>, all: bool, headless: bool) -> i32 {
    let scenario_paths = collect_scenario_paths(scenario, all);

    if scenario_paths.is_empty() {
        eprintln!("No scenarios found. Use -s <name> or --all.");
        return 1;
    }

    let mut any_failed = false;
    let mut shared_log_buffer: Option<LogBuffer> = None;

    for path in &scenario_paths {
        let result = run_scenario(path, headless, &mut shared_log_buffer);
        any_failed = any_failed || !result;
    }

    i32::from(any_failed)
}

/// Returns the path to the `scenarios/` directory relative to this crate's manifest.
#[must_use]
pub fn scenarios_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scenarios")
}

fn collect_scenario_paths(scenario: Option<&str>, all: bool) -> Vec<PathBuf> {
    let dir = scenarios_dir();

    if all {
        collect_all_scenarios(&dir)
    } else if let Some(name) = scenario {
        find_scenario_by_name(&dir, name).map_or_else(
            || {
                eprintln!("Scenario '{name}' not found in {}", dir.display());
                vec![]
            },
            |p| vec![p],
        )
    } else {
        vec![]
    }
}

fn collect_all_scenarios(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_scenarios_recursive(dir, &mut paths);
    paths.sort();
    paths
}

fn collect_scenarios_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_scenarios_recursive(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("ron")
            && path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".scenario.ron"))
        {
            out.push(path);
        }
    }
}

fn find_scenario_by_name(dir: &Path, name: &str) -> Option<PathBuf> {
    let target = format!("{name}.scenario.ron");
    let mut all = Vec::new();
    collect_scenarios_recursive(dir, &mut all);
    all.into_iter().find(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n == target)
    })
}

fn load_scenario(path: &Path) -> Option<ScenarioDefinition> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| eprintln!("Failed to read {}: {e}", path.display()))
        .ok()?;
    ron::de::from_str(&content)
        .map_err(|e| eprintln!("Failed to parse {}: {e}", path.display()))
        .ok()
}

/// Builds and runs one scenario app. Returns `true` if passed, `false` if failed.
///
/// The `shared_log_buffer` persists across scenarios so the global tracing
/// subscriber (installed once) always writes to the same buffer that each app's
/// `poll_log_buffer` system reads from.
fn run_scenario(path: &Path, headless: bool, shared_log_buffer: &mut Option<LogBuffer>) -> bool {
    let scenario_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .trim_end_matches(".scenario")
        .to_owned();

    let Some(definition) = load_scenario(path) else {
        eprintln!("FAIL [{scenario_name}]: could not load scenario file");
        return false;
    };

    println!(
        "Running [{scenario_name}] breaker={} layout={}",
        definition.breaker, definition.layout
    );
    info!(
        target: "breaker_scenario_runner",
        "scenario start name={scenario_name} breaker={} layout={}",
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

    app.insert_resource(ScenarioConfig { definition });
    app.add_plugins(ScenarioLifecycle);
    app.init_resource::<CapturedLogs>();
    app.add_systems(FixedUpdate, poll_log_buffer);

    if headless {
        // Run manually so we retain access to the World after exit.
        // App::run() replaces self with App::empty(), losing all resources.
        app.finish();
        app.cleanup();

        let wall_clock = Instant::now();
        let timeout = Duration::from_mins(2);

        loop {
            match guarded_update(&mut app) {
                Ok(()) => {}
                Err(msg) => {
                    eprintln!("FAIL [{scenario_name}]: system panic: {msg}");
                    break;
                }
            }
            if app.should_exit().is_some() {
                break;
            }
            if is_timed_out(wall_clock, timeout) {
                let frame = app
                    .world()
                    .get_resource::<ScenarioFrame>()
                    .map_or(0, |f| f.0);
                eprintln!(
                    "FAIL [{scenario_name}]: wall-clock timeout ({timeout:?}) at frame {frame}"
                );
                break;
            }
        }

        // Drain any logs emitted after the last FixedUpdate tick
        drain_remaining_logs(&mut app);
    } else {
        // Visual mode — Winit needs app.run() for the event loop.
        // Results cannot be read after run(); visual mode is for debugging only.
        app.run();
        println!("  [{scenario_name}] visual mode — pass/fail not evaluated");
        return true;
    }

    collect_and_evaluate(&app, &scenario_name)
}

/// Extracts results from the app world and evaluates pass/fail using [`ScenarioVerdict`].
///
/// Returns `false` if any expected resource is missing, any health check fails,
/// any invariant violation is unexpected, or any log was captured.
fn collect_and_evaluate(app: &App, scenario_name: &str) -> bool {
    let mut verdict = ScenarioVerdict::default();

    let vl = app.world().get_resource::<ViolationLog>();
    let cl = app.world().get_resource::<CapturedLogs>();
    let cfg = app.world().get_resource::<ScenarioConfig>();
    let st = app.world().get_resource::<ScenarioStats>();

    if let (Some(vl), Some(cl), Some(cfg), Some(st)) = (vl, cl, cfg, st) {
        verdict.evaluate(&vl.0, &cl.0, st, &cfg.definition);
    } else {
        if vl.is_none() {
            verdict.add_fail_reason("ViolationLog resource missing after run".into());
        }
        if cl.is_none() {
            verdict.add_fail_reason("CapturedLogs resource missing after run".into());
        }
        if cfg.is_none() {
            verdict.add_fail_reason("ScenarioConfig resource missing after run".into());
        }
        if st.is_none() {
            verdict.add_fail_reason("ScenarioStats resource missing after run".into());
        }
    }

    // Clone data for printing (resources are borrowed from world above).
    let violations = vl.map(|v| v.0.clone()).unwrap_or_default();
    let logs = cl.map(|c| c.0.clone()).unwrap_or_default();
    let stats = st.cloned().unwrap_or_default();

    println!(
        "  [{scenario_name}] frames={} actions={} violations={} logs={} bolts={} breakers={} entered_playing={}",
        stats.max_frame,
        stats.actions_injected,
        violations.len(),
        logs.len(),
        stats.bolts_tagged,
        stats.breakers_tagged,
        stats.entered_playing
    );

    if verdict.passed() {
        println!("PASS [{scenario_name}]");
        info!(target: "breaker_scenario_runner", "scenario pass name={scenario_name}");
    } else {
        let reason_count = verdict.reasons.len();
        println!("FAIL [{scenario_name}]: {reason_count} reason(s)");
        warn!(
            target: "breaker_scenario_runner",
            "scenario fail name={scenario_name} reasons={reason_count}",
        );
        for reason in &verdict.reasons {
            println!("  REASON [{scenario_name}]: {reason}");
        }
        for v in &violations {
            println!(
                "  VIOLATION frame={} {:?} entity={:?}: {}",
                v.frame, v.invariant, v.entity, v.message
            );
            debug!(
                target: "breaker_scenario_runner",
                "violation frame={} invariant={:?} entity={:?}: {}",
                v.frame, v.invariant, v.entity, v.message
            );
        }
        for l in &logs {
            println!(
                "  LOG frame={} {:?} target={}: {}",
                l.frame, l.level, l.target, l.message
            );
        }
    }

    verdict.passed()
}

/// Builds a Bevy app configured for scenario running.
///
/// In headless mode, disables winit so the app runs without a display server.
/// On the first run, installs `LogPlugin` with a custom tracing layer.
/// On subsequent runs, disables `LogPlugin` to avoid the "global logger already
/// set" error — the shared `LogBuffer` is inserted by `run_scenario` instead.
fn build_app(headless: bool, first_run: bool) -> App {
    let mut app = App::new();

    let window = if headless {
        WindowPlugin {
            primary_window: None,
            exit_condition: ExitCondition::DontExit,
            ..default()
        }
    } else {
        WindowPlugin {
            primary_window: Some(Window {
                title: "Scenario Runner".into(),
                ..default()
            }),
            ..default()
        }
    };

    // Point to the game crate's assets directory so scenarios
    // load real RON config files rather than code defaults.
    let mut defaults = DefaultPlugins.set(window).set(bevy::asset::AssetPlugin {
        file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/../breaker-game/assets").to_owned(),
        ..default()
    });

    if first_run {
        defaults = defaults.set(LogPlugin {
            filter: "warn,bevy_egui=error,breaker=info".to_owned(),
            custom_layer: scenario_log_layer_factory,
            ..default()
        });
    } else {
        defaults = defaults.disable::<LogPlugin>();
    }

    if headless {
        defaults = defaults.disable::<WinitPlugin>();
    }

    app.add_plugins(defaults);

    if headless {
        // Advance simulated time by exactly one fixed timestep per Update tick.
        // Without this, Time<Fixed> accumulates based on real wall-clock elapsed
        // time, so a 20k-frame scenario would take ~5 minutes. With ManualDuration,
        // each Update tick instantly advances virtual time by 1/64 s, and all
        // Fixed ticks execute in sequence at CPU speed.
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 64.0,
        )));

        app.add_plugins(Game::headless());
    } else {
        app.add_plugins(Game::default());
    }

    app
}

/// Returns `true` if `start` elapsed longer ago than `timeout`.
///
/// Used by the run loop to detect wall-clock timeouts without blocking.
#[must_use]
pub fn is_timed_out(start: Instant, timeout: Duration) -> bool {
    start.elapsed() > timeout
}

/// Drains any buffered log entries from [`LogBuffer`] into [`CapturedLogs`].
///
/// Called after the run loop exits to ensure entries captured after the last
/// `poll_log_buffer` tick are not silently discarded.
pub fn drain_remaining_logs(app: &mut App) {
    // Extract buffer entries into a local vec first — cannot hold &World and &mut World
    // simultaneously, so we must release the immutable borrow before writing CapturedLogs.
    let buffered: Vec<(bevy::log::Level, String, String)> = app
        .world()
        .get_resource::<LogBuffer>()
        .map(|buf| {
            buf.0
                .lock()
                .map(|mut guard| guard.drain(..).collect())
                .unwrap_or_default()
        })
        .unwrap_or_default();

    if buffered.is_empty() {
        return;
    }

    let frame = app
        .world()
        .get_resource::<ScenarioFrame>()
        .map_or(0, |f| f.0);

    if let Some(mut logs) = app.world_mut().get_resource_mut::<CapturedLogs>() {
        for (level, target, message) in buffered {
            logs.0.push(LogEntry {
                level,
                target,
                message,
                frame,
            });
        }
    }
}

/// Runs a single `app.update()` call, catching any panics and returning them as `Err`.
///
/// Returns `Ok(())` on a clean update, or `Err(message)` if the update panicked.
///
/// # Errors
///
/// Returns the panic message as a `String` if any system panics during the update.
pub fn guarded_update(app: &mut App) -> Result<(), String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        app.update();
    }))
    .map_err(|payload| {
        payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_owned())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic".to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{invariants::ScenarioFrame, log_capture::LogBuffer};

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
        use std::sync::{Arc, Mutex};

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
        app.insert_resource(log_buffer);
        app.insert_resource(CapturedLogs::default());
        app.insert_resource(ScenarioFrame(42));

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
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, panicking_system);

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
}
