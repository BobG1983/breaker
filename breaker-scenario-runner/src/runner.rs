//! App construction and multi-scenario execution.
//!
//! Builds either a visual or headless [`App`] for each scenario and runs it to
//! completion, then prints a structured summary and returns the exit code.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use bevy::{
    app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*, time::TimeUpdateStrategy,
    window::ExitCondition, winit::WinitPlugin,
};
use breaker::game::Game;
use tracing::{debug, info, warn};

use crate::{
    invariants::ViolationLog,
    lifecycle::{ScenarioConfig, ScenarioLifecycle},
    log_capture::{CapturedLogs, poll_log_buffer, scenario_log_layer_factory},
    types::ScenarioDefinition,
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

    for path in &scenario_paths {
        let result = run_scenario(path, headless);
        any_failed |= !result;
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

/// Determines whether a scenario passed given its results and definition.
///
/// When `expected_violations` is `None`, the scenario passes only if there are no
/// violations and no captured logs.
///
/// When `expected_violations` is `Some(expected)`, the scenario passes only if:
/// - every listed invariant fired at least once
/// - no unlisted invariants fired
/// - no unexpected logs were captured
fn evaluate_pass(
    violations: &[crate::invariants::ViolationEntry],
    logs: &[crate::log_capture::LogEntry],
    definition: Option<&ScenarioDefinition>,
) -> bool {
    definition
        .and_then(|d| d.expected_violations.as_deref())
        .map_or(violations.is_empty() && logs.is_empty(), |expected| {
            let all_expected_fired = expected
                .iter()
                .all(|ev| violations.iter().any(|v| &v.invariant == ev));
            let no_unexpected = violations
                .iter()
                .all(|v| expected.iter().any(|ev| ev == &v.invariant));
            all_expected_fired && no_unexpected && logs.is_empty()
        })
}

/// Builds and runs one scenario app. Returns `true` if passed, `false` if failed.
fn run_scenario(path: &Path, headless: bool) -> bool {
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

    let mut app = build_app(headless);
    app.insert_resource(ScenarioConfig { definition });
    app.add_plugins(ScenarioLifecycle);
    app.init_resource::<CapturedLogs>();
    app.add_systems(FixedUpdate, poll_log_buffer);

    app.run();

    // Collect results
    let violations = app
        .world()
        .get_resource::<ViolationLog>()
        .map(|v| v.0.clone())
        .unwrap_or_default();
    let logs = app
        .world()
        .get_resource::<CapturedLogs>()
        .map(|l| l.0.clone())
        .unwrap_or_default();
    let definition = app
        .world()
        .get_resource::<ScenarioConfig>()
        .map(|c| c.definition.clone());

    let passed = evaluate_pass(&violations, &logs, definition.as_ref());

    if passed {
        println!("PASS [{scenario_name}]");
        info!(target: "breaker_scenario_runner", "scenario pass name={scenario_name}");
    } else {
        println!(
            "FAIL [{scenario_name}]: {} violations, {} captured logs",
            violations.len(),
            logs.len()
        );
        warn!(
            target: "breaker_scenario_runner",
            "scenario fail name={scenario_name} violations={} logs={}",
            violations.len(), logs.len()
        );
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

    passed
}

/// Builds a Bevy app configured for scenario running.
///
/// In headless mode, disables winit and uses `ScheduleRunnerPlugin` to drive
/// the app without a display server. In visual mode, uses normal `DefaultPlugins`.
fn build_app(headless: bool) -> App {
    let mut app = App::new();

    if headless {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .set(LogPlugin {
                    filter: "warn,breaker=info".to_owned(),
                    custom_layer: scenario_log_layer_factory,
                    ..default()
                })
                .set(bevy::asset::AssetPlugin {
                    // Point to the game crate's assets directory so scenarios
                    // load real RON config files rather than code defaults.
                    file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/../breaker-game/assets")
                        .to_owned(),
                    ..default()
                })
                .disable::<WinitPlugin>(),
        )
        // No sleep between Update ticks — run as fast as possible.
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::ZERO));

        // Advance simulated time by exactly one fixed timestep per Update tick.
        // Without this, Time<Fixed> accumulates based on real wall-clock elapsed
        // time, so a 20k-frame scenario would take ~5 minutes. With ManualDuration,
        // each Update tick instantly advances virtual time by 1/64 s, and all
        // Fixed ticks execute in sequence at CPU speed.
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 64.0,
        )));
    } else {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Scenario Runner".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "warn,breaker=info".to_owned(),
                    custom_layer: scenario_log_layer_factory,
                    ..default()
                }),
        );
    }

    app.add_plugins(Game);
    app
}
