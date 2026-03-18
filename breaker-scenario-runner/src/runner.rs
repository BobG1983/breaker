//! App construction and multi-scenario execution.
//!
//! Builds either a visual or headless [`App`] for each scenario and runs it to
//! completion, then prints a structured summary and returns the exit code.

use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{log::LogPlugin, prelude::*, time::TimeUpdateStrategy};
use breaker::game::Game;
use tracing::{debug, info, warn};

use crate::{
    invariants::{ScenarioFrame, ScenarioStats, ViolationEntry, ViolationLog},
    lifecycle::{ScenarioConfig, ScenarioLifecycle},
    log_capture::{CapturedLogs, LogBuffer, LogEntry, poll_log_buffer, scenario_log_layer_factory},
    types::{InvariantKind, ScenarioDefinition},
    verdict::ScenarioVerdict,
};

/// Entry point called by `main`. Returns process exit code (0 = all pass, 1 = any fail).
#[must_use]
pub fn run_with_args(scenario: Option<&str>, all: bool, headless: bool, verbose: bool) -> i32 {
    let scenario_paths = collect_scenario_paths(scenario, all);

    if scenario_paths.is_empty() {
        eprintln!("No scenarios found. Use -s <name> or --all.");
        return 1;
    }

    let mut results: Vec<(String, bool)> = Vec::new();
    let mut shared_log_buffer: Option<LogBuffer> = None;

    for path in &scenario_paths {
        let name = scenario_name(path);
        let passed = run_scenario(path, headless, verbose, &mut shared_log_buffer);
        results.push((name, passed));
    }

    // Cross-scenario summary.
    let passed_count = results.iter().filter(|(_, p)| *p).count();
    let failed_count = results.len() - passed_count;
    let failures: Vec<&str> = results
        .iter()
        .filter(|(_, p)| !*p)
        .map(|(name, _)| name.as_str())
        .collect();

    println!("\n---");
    if failures.is_empty() {
        println!("scenario result: ok. {passed_count} passed; {failed_count} failed");
    } else {
        println!("scenario result: FAIL. {passed_count} passed; {failed_count} failed");
        println!("\nfailures:");
        for name in &failures {
            println!("  {name}");
        }
    }

    i32::from(failed_count > 0)
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

fn scenario_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .trim_end_matches(".scenario")
        .to_owned()
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
fn run_scenario(
    path: &Path,
    headless: bool,
    verbose: bool,
    shared_log_buffer: &mut Option<LogBuffer>,
) -> bool {
    let sname = scenario_name(path);

    let Some(definition) = load_scenario(path) else {
        eprintln!("FAIL [{sname}]: could not load scenario file");
        return false;
    };

    println!(
        "Running [{sname}] breaker={} layout={}",
        definition.breaker, definition.layout
    );
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
                    eprintln!("FAIL [{sname}]: system panic: {msg}");
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
                eprintln!("FAIL [{sname}]: wall-clock timeout ({timeout:?}) at frame {frame}");
                break;
            }
        }

        // Drain any logs emitted after the last FixedUpdate tick
        drain_remaining_logs(&mut app);
    } else {
        // Visual mode — Winit needs app.run() for the event loop.
        // Results cannot be read after run(); visual mode is for debugging only.
        app.run();
        println!("  [{sname}] visual mode — pass/fail not evaluated");
        return true;
    }

    collect_and_evaluate(&app, &sname, verbose)
}

/// Extracts results from the app world and evaluates pass/fail using [`ScenarioVerdict`].
///
/// Returns `false` if any expected resource is missing, any health check fails,
/// any invariant violation is unexpected, or any log was captured.
fn collect_and_evaluate(app: &App, scenario_name: &str, verbose: bool) -> bool {
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
        println!("FAIL [{scenario_name}]: {reason_count} failure(s)");
        warn!(
            target: "breaker_scenario_runner",
            "scenario fail name={scenario_name} reasons={reason_count}",
        );

        if verbose {
            print_verbose_failures(scenario_name, &verdict, &violations, &logs);
        } else {
            print_compact_failures(&verdict, &violations, &logs);
        }
    }

    verdict.passed()
}

// ---------------------------------------------------------------------------
// Verbose output (--verbose flag)
// ---------------------------------------------------------------------------

fn print_verbose_failures(
    scenario_name: &str,
    verdict: &ScenarioVerdict,
    violations: &[ViolationEntry],
    logs: &[LogEntry],
) {
    for reason in &verdict.reasons {
        println!("  REASON [{scenario_name}]: {reason}");
    }
    for v in violations {
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
    for l in logs {
        println!(
            "  LOG frame={} {:?} target={}: {}",
            l.frame, l.level, l.target, l.message
        );
    }
}

// ---------------------------------------------------------------------------
// Compact output (default)
// ---------------------------------------------------------------------------

fn print_compact_failures(
    verdict: &ScenarioVerdict,
    violations: &[ViolationEntry],
    logs: &[LogEntry],
) {
    // Grouped violations.
    let violation_groups = group_violations(violations);
    for g in &violation_groups {
        println!(
            "  {:30} x{:<5} {}",
            format!("{:?}", g.invariant),
            g.count,
            format_frame_range(g.count, g.first_frame, g.last_frame)
        );
    }

    // Grouped logs.
    let log_groups = group_logs(logs);
    for g in &log_groups {
        println!(
            "  {:30} x{:<5} {}",
            format!("captured {:?} log", g.level),
            g.count,
            format_frame_range(g.count, g.first_frame, g.last_frame)
        );
        if g.count == 1 {
            println!("    {}", g.message);
        }
    }

    // Health-check reasons (those not covered by violations or logs).
    for reason in &verdict.reasons {
        if is_health_check_reason(reason) {
            println!("  {reason}");
        }
    }
}

fn format_frame_range(count: u32, first: u32, last: u32) -> String {
    if count == 1 {
        format!("frame {first}")
    } else {
        format!("frames {first}..{last}")
    }
}

/// Returns `true` if the reason is a health-check (not a violation or log reason).
fn is_health_check_reason(reason: &str) -> bool {
    // Violation reasons come from InvariantKind::fail_reason() and log reasons
    // start with "captured". Health checks are everything else.
    !reason.starts_with("captured ") && !is_invariant_fail_reason(reason)
}

/// Returns `true` if the reason matches any `InvariantKind::fail_reason()`.
fn is_invariant_fail_reason(reason: &str) -> bool {
    InvariantKind::ALL.iter().any(|v| v.fail_reason() == reason)
}

// ---------------------------------------------------------------------------
// Grouping types and functions
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
struct ViolationGroup {
    invariant: InvariantKind,
    count: u32,
    first_frame: u32,
    last_frame: u32,
}

#[derive(Debug, PartialEq)]
struct LogGroup {
    level: bevy::log::Level,
    message: String,
    count: u32,
    first_frame: u32,
    last_frame: u32,
}

fn group_violations(violations: &[ViolationEntry]) -> Vec<ViolationGroup> {
    use std::collections::HashMap;

    let mut map: HashMap<InvariantKind, (u32, u32, u32)> = HashMap::new();
    let mut insertion_order: Vec<InvariantKind> = Vec::new();

    for v in violations {
        let entry = map.entry(v.invariant).or_insert_with(|| {
            insertion_order.push(v.invariant);
            (0, v.frame, v.frame)
        });
        entry.0 += 1;
        entry.1 = entry.1.min(v.frame);
        entry.2 = entry.2.max(v.frame);
    }

    insertion_order
        .into_iter()
        .filter_map(|kind| {
            map.get(&kind).map(|&(count, first, last)| ViolationGroup {
                invariant: kind,
                count,
                first_frame: first,
                last_frame: last,
            })
        })
        .collect()
}

fn group_logs(logs: &[LogEntry]) -> Vec<LogGroup> {
    use std::collections::HashMap;

    type Key = (bevy::log::Level, String);
    let mut map: HashMap<Key, (u32, u32, u32)> = HashMap::new();
    let mut insertion_order: Vec<Key> = Vec::new();

    for l in logs {
        let key: Key = (l.level, l.message.clone());
        let entry = map.entry(key.clone()).or_insert_with(|| {
            insertion_order.push(key);
            (0, l.frame, l.frame)
        });
        entry.0 += 1;
        entry.1 = entry.1.min(l.frame);
        entry.2 = entry.2.max(l.frame);
    }

    insertion_order
        .into_iter()
        .filter_map(|key| {
            map.get(&key).map(|&(count, first, last)| LogGroup {
                level: key.0,
                message: key.1,
                count,
                first_frame: first,
                last_frame: last,
            })
        })
        .collect()
}

/// Bevy's default fixed timestep frequency (Hz).
const FIXED_TIMESTEP_HZ: f64 = 64.0;

/// Scenario runner log plugin — captures warnings via [`scenario_log_layer_factory`].
fn scenario_log_plugin() -> LogPlugin {
    LogPlugin {
        level: bevy::log::Level::WARN,
        filter: "warn,bevy_egui=error".to_owned(),
        custom_layer: scenario_log_layer_factory,
        ..default()
    }
}

/// Speed multiplier for visual mode — each rendered frame advances virtual
/// time by this many fixed timesteps.
const VISUAL_SPEED_MULTIPLIER: f64 = 10.0;

/// Builds a Bevy app configured for scenario running.
///
/// In headless mode, uses [`MinimalPlugins`] with only the specific plugins
/// game systems need (states, assets, input, mesh). This avoids pulling in the
/// full render pipeline, winit event loop, and GPU initialization — none of
/// which are needed when running scenarios at CPU speed on CI.
///
/// In visual mode, uses [`DefaultPlugins`] for full windowed rendering.
///
/// On the first run, installs `LogPlugin` with a custom tracing layer.
/// On subsequent runs, skips `LogPlugin` to avoid the "global logger already
/// set" error — the shared `LogBuffer` is inserted by `run_scenario` instead.
fn build_app(headless: bool, first_run: bool) -> App {
    let mut app = App::new();

    // Point to the game crate's assets directory so scenarios
    // load real RON config files rather than code defaults.
    let game_asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../breaker-game/assets").to_owned();

    if headless {
        // Minimal plugin set — no render pipeline, no window, no GPU.
        app.add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            bevy::asset::AssetPlugin {
                file_path: game_asset_path,
                ..default()
            },
            bevy::input::InputPlugin,
            bevy::mesh::MeshPlugin,
        ));

        if first_run {
            app.add_plugins(scenario_log_plugin());
        }

        // Advance simulated time by exactly one fixed timestep per Update tick.
        // Without this, Time<Fixed> accumulates based on real wall-clock elapsed
        // time, so a 20k-frame scenario would take ~5 minutes. With ManualDuration,
        // each Update tick instantly advances virtual time by 1/64 s, and all
        // Fixed ticks execute in sequence at CPU speed.
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / FIXED_TIMESTEP_HZ,
        )));

        app.add_plugins(Game::headless());
    } else {
        // Visual mode — full DefaultPlugins for windowed rendering.
        let mut defaults = DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Scenario Runner".into(),
                    ..default()
                }),
                ..default()
            })
            .set(bevy::asset::AssetPlugin {
                file_path: game_asset_path,
                ..default()
            });

        if first_run {
            defaults = defaults.set(scenario_log_plugin());
        } else {
            defaults = defaults.disable::<LogPlugin>();
        }

        app.add_plugins(defaults);

        // Visual mode runs at 10x speed to avoid 5+ minute waits for
        // 20,000-frame scenarios. Each Update tick advances virtual time
        // by 10 fixed timesteps (10/64 s).
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            VISUAL_SPEED_MULTIPLIER / FIXED_TIMESTEP_HZ,
        )));
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

    // -------------------------------------------------------------------------
    // group_violations — groups by invariant kind
    // -------------------------------------------------------------------------

    fn make_violation(invariant: InvariantKind, frame: u32) -> ViolationEntry {
        ViolationEntry {
            frame,
            invariant,
            entity: None,
            message: format!("test: {invariant:?}"),
        }
    }

    #[test]
    fn group_violations_groups_by_invariant_kind() {
        let violations = vec![
            make_violation(InvariantKind::BoltInBounds, 100),
            make_violation(InvariantKind::BoltInBounds, 101),
            make_violation(InvariantKind::BoltInBounds, 105),
        ];

        let groups = group_violations(&violations);

        assert_eq!(
            groups.len(),
            1,
            "3 same-kind violations must produce 1 group"
        );
        assert_eq!(groups[0].invariant, InvariantKind::BoltInBounds);
        assert_eq!(groups[0].count, 3);
        assert_eq!(groups[0].first_frame, 100);
        assert_eq!(groups[0].last_frame, 105);
    }

    #[test]
    fn group_violations_separates_different_invariant_kinds() {
        let violations = vec![
            make_violation(InvariantKind::BoltInBounds, 10),
            make_violation(InvariantKind::NoNaN, 20),
            make_violation(InvariantKind::BoltInBounds, 30),
        ];

        let groups = group_violations(&violations);

        assert_eq!(
            groups.len(),
            2,
            "BoltInBounds + NoNaN must produce 2 groups"
        );
        let bolt = groups
            .iter()
            .find(|g| g.invariant == InvariantKind::BoltInBounds)
            .unwrap();
        let nan = groups
            .iter()
            .find(|g| g.invariant == InvariantKind::NoNaN)
            .unwrap();
        assert_eq!(bolt.count, 2);
        assert_eq!(bolt.first_frame, 10);
        assert_eq!(bolt.last_frame, 30);
        assert_eq!(nan.count, 1);
        assert_eq!(nan.first_frame, 20);
        assert_eq!(nan.last_frame, 20);
    }

    #[test]
    fn group_violations_single_entry_has_matching_first_last_frame() {
        let violations = vec![make_violation(InvariantKind::NoNaN, 42)];

        let groups = group_violations(&violations);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].first_frame, 42);
        assert_eq!(groups[0].last_frame, 42);
        assert_eq!(groups[0].count, 1);
    }

    // -------------------------------------------------------------------------
    // group_logs — groups by level + message
    // -------------------------------------------------------------------------

    fn make_log(level: bevy::log::Level, message: &str, frame: u32) -> LogEntry {
        LogEntry {
            level,
            target: "breaker::test".to_owned(),
            message: message.to_owned(),
            frame,
        }
    }

    #[test]
    fn group_logs_groups_by_level_and_message() {
        let logs = vec![
            make_log(bevy::log::Level::WARN, "bad thing", 100),
            make_log(bevy::log::Level::WARN, "bad thing", 200),
            make_log(bevy::log::Level::WARN, "bad thing", 300),
        ];

        let groups = group_logs(&logs);

        assert_eq!(groups.len(), 1, "3 identical logs must produce 1 group");
        assert_eq!(groups[0].count, 3);
        assert_eq!(groups[0].first_frame, 100);
        assert_eq!(groups[0].last_frame, 300);
        assert_eq!(groups[0].message, "bad thing");
    }

    #[test]
    fn group_logs_separates_different_messages() {
        let logs = vec![
            make_log(bevy::log::Level::WARN, "msg a", 10),
            make_log(bevy::log::Level::WARN, "msg b", 20),
        ];

        let groups = group_logs(&logs);

        assert_eq!(
            groups.len(),
            2,
            "2 different messages must produce 2 groups"
        );
    }

    #[test]
    fn group_logs_separates_different_levels_same_message() {
        let logs = vec![
            make_log(bevy::log::Level::WARN, "same msg", 10),
            make_log(bevy::log::Level::ERROR, "same msg", 20),
        ];

        let groups = group_logs(&logs);

        assert_eq!(
            groups.len(),
            2,
            "WARN + ERROR with same message must produce 2 groups"
        );
    }

    // -------------------------------------------------------------------------
    // is_invariant_fail_reason — matches all InvariantKind fail reasons
    // -------------------------------------------------------------------------

    #[test]
    fn is_invariant_fail_reason_returns_true_for_all_invariant_kinds() {
        for variant in InvariantKind::ALL {
            assert!(
                is_invariant_fail_reason(variant.fail_reason()),
                "is_invariant_fail_reason must return true for {:?} fail_reason: {:?}",
                variant,
                variant.fail_reason()
            );
        }
    }

    #[test]
    fn is_invariant_fail_reason_returns_false_for_health_check_strings() {
        assert!(
            !is_invariant_fail_reason("no actions were injected during scenario run"),
            "health check string must not match as invariant fail reason"
        );
        assert!(
            !is_invariant_fail_reason("scenario never entered Playing state"),
            "health check string must not match as invariant fail reason"
        );
    }
}
