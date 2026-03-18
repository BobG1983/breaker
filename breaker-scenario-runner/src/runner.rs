//! App construction and multi-scenario execution.
//!
//! Builds either a visual or headless [`App`] for each scenario and runs it to
//! completion, then prints a structured summary and returns the exit code.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use bevy::{
    log::LogPlugin, prelude::*, time::TimeUpdateStrategy,
    window::ExitCondition, winit::WinitPlugin,
};
use breaker::game::Game;
use tracing::{debug, info, warn};

use crate::{
    invariants::{ScenarioStats, ViolationLog},
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

/// Determines whether a scenario passed given its results and definition.
///
/// Captured logs always count as a failure regardless of `expected_violations`.
///
/// When `expected_violations` is `None`, the scenario passes only if there are no
/// violations and no captured logs.
///
/// When `expected_violations` is `Some(expected)`, the scenario passes only if:
/// - every listed invariant fired at least once
/// - no unlisted invariants fired
/// - no captured logs
///
/// `Some([])` (empty expected list) produces the same result as `None`, but makes
/// the expectation explicit in the RON file.
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

/// Produces human-readable health failure strings for a completed scenario run.
///
/// Any non-empty result causes the scenario to fail. This catches runs that
/// "pass" vacuously because the game never actually started or exercised the
/// systems under test.
///
/// Returns an empty `Vec` when no health issues are detected.
#[must_use]
pub fn scenario_health_warnings(
    stats: &crate::invariants::ScenarioStats,
    definition: &ScenarioDefinition,
) -> Vec<String> {
    use crate::types::{InputStrategy, ScriptedParams};

    let mut warnings = Vec::new();

    let is_empty_scripted = matches!(
        &definition.input,
        InputStrategy::Scripted(ScriptedParams { actions }) if actions.is_empty()
    );

    if stats.actions_injected == 0 && !is_empty_scripted {
        warnings.push(format!(
            "no actions were injected during scenario run (input strategy: {:?})",
            definition.input
        ));
    }

    if !stats.entered_playing {
        warnings.push("scenario never entered Playing state".to_owned());
    }

    if stats.bolts_tagged == 0 {
        warnings.push("no bolts were tagged — bolt invariants are vacuous".to_owned());
    }

    if stats.breakers_tagged == 0 {
        warnings.push("no breakers were tagged — breaker invariants are vacuous".to_owned());
    }

    if stats.max_frame < 10 {
        warnings.push(format!(
            "scenario exited very early (max_frame={})",
            stats.max_frame
        ));
    }

    if stats.invariant_checks == 0 {
        warnings.push(
            "no invariant checks ran — game loop may not have executed".to_owned(),
        );
    }

    warnings
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

    if headless {
        // Run manually so we retain access to the World after exit.
        // App::run() replaces self with App::empty(), losing all resources.
        app.finish();
        app.cleanup();
        loop {
            app.update();
            if app.should_exit().is_some() {
                break;
            }
        }
    } else {
        // Visual mode — Winit needs app.run() for the event loop.
        // Results cannot be read after run(); visual mode is for debugging only.
        app.run();
        println!("  [{scenario_name}] visual mode — pass/fail not evaluated");
        return true;
    }

    collect_and_evaluate(&app, &scenario_name)
}

/// Extracts results from the app world and evaluates pass/fail.
///
/// Returns `false` if any expected resource is missing, any health check fails,
/// any invariant violation is unexpected, or any log was captured.
fn collect_and_evaluate(app: &App, scenario_name: &str) -> bool {
    let Some(violation_log) = app.world().get_resource::<ViolationLog>() else {
        eprintln!("FAIL [{scenario_name}]: ViolationLog resource missing after run");
        return false;
    };
    let violations = violation_log.0.clone();

    let Some(captured_logs) = app.world().get_resource::<CapturedLogs>() else {
        eprintln!("FAIL [{scenario_name}]: CapturedLogs resource missing after run");
        return false;
    };
    let logs = captured_logs.0.clone();

    let Some(config) = app.world().get_resource::<ScenarioConfig>() else {
        eprintln!("FAIL [{scenario_name}]: ScenarioConfig resource missing after run");
        return false;
    };
    let definition = config.definition.clone();

    let Some(stats) = app.world().get_resource::<ScenarioStats>().cloned() else {
        eprintln!("FAIL [{scenario_name}]: ScenarioStats resource missing after run");
        return false;
    };

    let mut passed = evaluate_pass(&violations, &logs, Some(&definition));

    let health_issues = scenario_health_warnings(&stats, &definition);
    for issue in &health_issues {
        println!("  HEALTH FAIL [{scenario_name}]: {issue}");
    }
    if !health_issues.is_empty() {
        passed = false;
    }

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
/// In headless mode, disables winit so the app runs without a display server.
/// In visual mode, uses normal `DefaultPlugins`.
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
        );

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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::log::Level;
    use crate::{
        invariants::{ScenarioStats, ViolationEntry},
        log_capture::LogEntry,
        types::{ChaosParams, InputStrategy, InvariantKind, InvariantParams},
    };

    // -------------------------------------------------------------------------
    // Helpers for constructing test data
    // -------------------------------------------------------------------------

    fn make_violation(invariant: InvariantKind) -> ViolationEntry {
        ViolationEntry {
            frame: 42,
            invariant,
            entity: None,
            message: format!("test violation: {invariant:?}"),
        }
    }

    fn make_log_entry() -> LogEntry {
        LogEntry {
            level: Level::WARN,
            target: "breaker::bolt::systems".to_owned(),
            message: "unexpected condition".to_owned(),
            frame: 10,
        }
    }

    fn make_chaos_definition() -> ScenarioDefinition {
        ScenarioDefinition {
            breaker: "aegis".to_owned(),
            layout: "corridor".to_owned(),
            input: InputStrategy::Chaos(ChaosParams {
                seed: 0,
                action_prob: 0.3,
            }),
            max_frames: 1000,
            invariants: vec![],
            expected_violations: None,
            debug_setup: None,
            invariant_params: InvariantParams::default(),
        }
    }

    // -------------------------------------------------------------------------
    // scenario_health_warnings — warns when no actions injected with Chaos input
    // -------------------------------------------------------------------------

    /// When `stats.actions_injected == 0` and the input strategy is `Chaos`
    /// (which should produce actions), `scenario_health_warnings` must return
    /// at least one warning containing "no actions were injected".
    #[test]
    fn scenario_health_warnings_warns_when_no_actions_injected_with_chaos_input() {
        let stats = ScenarioStats {
            actions_injected: 0,
            invariant_checks: 100,
            max_frame: 1000,
            entered_playing: true,
            bolts_tagged: 1,
            breakers_tagged: 1,
        };
        let definition = make_chaos_definition();

        let warnings = scenario_health_warnings(&stats, &definition);

        assert!(
            !warnings.is_empty(),
            "expected at least one health warning when no actions were injected with Chaos input"
        );

        let has_no_actions_warning = warnings
            .iter()
            .any(|w| w.to_lowercase().contains("no actions were injected"));

        assert!(
            has_no_actions_warning,
            "expected warning containing 'no actions were injected', got: {warnings:?}"
        );
    }

    // -------------------------------------------------------------------------
    // scenario_health_warnings — warns when scenario never entered Playing
    // -------------------------------------------------------------------------

    /// When `stats.entered_playing == false`, `scenario_health_warnings` must
    /// return at least one warning containing "never entered Playing".
    #[test]
    fn scenario_health_warnings_warns_when_never_entered_playing() {
        let stats = ScenarioStats {
            actions_injected: 50,
            invariant_checks: 100,
            max_frame: 1000,
            entered_playing: false,
            bolts_tagged: 1,
            breakers_tagged: 1,
        };
        let definition = make_chaos_definition();

        let warnings = scenario_health_warnings(&stats, &definition);

        let has_playing_warning = warnings
            .iter()
            .any(|w| w.to_lowercase().contains("never entered playing"));

        assert!(
            has_playing_warning,
            "expected warning containing 'never entered Playing', got: {warnings:?}"
        );
    }

    // -------------------------------------------------------------------------
    // scenario_health_warnings — warns when no bolts tagged
    // -------------------------------------------------------------------------

    /// When `stats.bolts_tagged == 0`, `scenario_health_warnings` must return
    /// at least one warning containing "no bolts were tagged".
    #[test]
    fn scenario_health_warnings_warns_when_no_bolts_tagged() {
        let stats = ScenarioStats {
            actions_injected: 50,
            invariant_checks: 100,
            max_frame: 1000,
            entered_playing: true,
            bolts_tagged: 0,
            breakers_tagged: 1,
        };
        let definition = make_chaos_definition();

        let warnings = scenario_health_warnings(&stats, &definition);

        let has_bolt_warning = warnings
            .iter()
            .any(|w| w.to_lowercase().contains("no bolts were tagged"));

        assert!(
            has_bolt_warning,
            "expected warning containing 'no bolts were tagged', got: {warnings:?}"
        );
    }

    // -------------------------------------------------------------------------
    // scenario_health_warnings — warns when no breakers tagged
    // -------------------------------------------------------------------------

    /// When `stats.breakers_tagged == 0`, `scenario_health_warnings` must return
    /// at least one warning containing "no breakers were tagged".
    #[test]
    fn scenario_health_warnings_warns_when_no_breakers_tagged() {
        let stats = ScenarioStats {
            actions_injected: 50,
            invariant_checks: 100,
            max_frame: 1000,
            entered_playing: true,
            bolts_tagged: 1,
            breakers_tagged: 0,
        };
        let definition = make_chaos_definition();

        let warnings = scenario_health_warnings(&stats, &definition);

        let has_breaker_warning = warnings
            .iter()
            .any(|w| w.to_lowercase().contains("no breakers were tagged"));

        assert!(
            has_breaker_warning,
            "expected warning containing 'no breakers were tagged', got: {warnings:?}"
        );
    }

    // -------------------------------------------------------------------------
    // scenario_health_warnings — warns when scenario exits very early
    // -------------------------------------------------------------------------

    /// When `stats.max_frame == 5` (below the early-exit threshold of 10),
    /// `scenario_health_warnings` must return at least one warning containing
    /// "exited" or "very early".
    #[test]
    fn scenario_health_warnings_warns_when_scenario_exits_very_early() {
        let stats = ScenarioStats {
            actions_injected: 50,
            invariant_checks: 10,
            max_frame: 5,
            entered_playing: true,
            bolts_tagged: 1,
            breakers_tagged: 1,
        };
        let definition = make_chaos_definition();

        let warnings = scenario_health_warnings(&stats, &definition);

        let has_early_exit_warning = warnings.iter().any(|w| {
            let lower = w.to_lowercase();
            lower.contains("exited") || lower.contains("very early")
        });

        assert!(
            has_early_exit_warning,
            "expected warning containing 'exited' or 'very early' for max_frame=5, got: {warnings:?}"
        );
    }

    // -------------------------------------------------------------------------
    // scenario_health_warnings — no warnings for healthy scenario
    // -------------------------------------------------------------------------

    /// A healthy scenario (entered Playing, bolts tagged, breakers tagged,
    /// `max_frame`=100, `actions_injected`=50, Chaos input) must produce no warnings.
    #[test]
    fn scenario_health_warnings_no_warnings_for_healthy_scenario() {
        let stats = ScenarioStats {
            actions_injected: 50,
            invariant_checks: 100,
            max_frame: 100,
            entered_playing: true,
            bolts_tagged: 1,
            breakers_tagged: 1,
        };
        let definition = make_chaos_definition();

        let warnings = scenario_health_warnings(&stats, &definition);

        assert!(
            warnings.is_empty(),
            "expected no health warnings for a healthy scenario, got: {warnings:?}"
        );
    }

    // -------------------------------------------------------------------------
    // evaluate_pass — clean pass with no definition
    // -------------------------------------------------------------------------

    /// Empty violations, no logs, no definition: scenario passes.
    #[test]
    fn evaluate_pass_returns_true_with_no_violations_no_logs_no_definition() {
        let result = evaluate_pass(&[], &[], None);
        assert!(result, "expected pass when violations=[], logs=[], definition=None");
    }

    // -------------------------------------------------------------------------
    // evaluate_pass — violations without definition cause failure
    // -------------------------------------------------------------------------

    /// A [`ViolationEntry`] with no definition present causes failure.
    #[test]
    fn evaluate_pass_returns_false_when_violations_present_and_no_definition() {
        let violations = vec![make_violation(InvariantKind::BoltInBounds)];
        let result = evaluate_pass(&violations, &[], None);
        assert!(!result, "expected fail when violations=[BoltInBounds], definition=None");
    }

    // -------------------------------------------------------------------------
    // evaluate_pass — captured logs cause failure even with no violations
    // -------------------------------------------------------------------------

    /// Logs alone (no violations, no definition) cause the scenario to fail.
    #[test]
    fn evaluate_pass_returns_false_when_logs_present_and_no_violations_no_definition() {
        let logs = vec![make_log_entry()];
        let result = evaluate_pass(&[], &logs, None);
        assert!(!result, "expected fail when logs=[one entry], violations=[], definition=None");
    }

    // -------------------------------------------------------------------------
    // evaluate_pass — expected violations match exactly
    // -------------------------------------------------------------------------

    /// When `expected_violations` matches fired violations exactly, scenario passes.
    #[test]
    fn evaluate_pass_returns_true_when_expected_violations_match_exactly() {
        let violations = vec![make_violation(InvariantKind::BoltInBounds)];
        let mut definition = make_chaos_definition();
        definition.expected_violations = Some(vec![InvariantKind::BoltInBounds]);
        let result = evaluate_pass(&violations, &[], Some(&definition));
        assert!(
            result,
            "expected pass when violations=[BoltInBounds] and expected=[BoltInBounds]"
        );
    }

    // -------------------------------------------------------------------------
    // evaluate_pass — expected violation not fired causes failure
    // -------------------------------------------------------------------------

    /// When an expected invariant never fires, the scenario fails.
    #[test]
    fn evaluate_pass_returns_false_when_expected_violation_not_fired() {
        let mut definition = make_chaos_definition();
        definition.expected_violations = Some(vec![InvariantKind::BoltInBounds]);
        let result = evaluate_pass(&[], &[], Some(&definition));
        assert!(
            !result,
            "expected fail when expected=[BoltInBounds] but violations=[]"
        );
    }

    // -------------------------------------------------------------------------
    // evaluate_pass — unexpected violation causes failure
    // -------------------------------------------------------------------------

    /// When a fired violation is not in the expected list, the scenario fails.
    #[test]
    fn evaluate_pass_returns_false_when_unexpected_violation_fires() {
        let violations = vec![make_violation(InvariantKind::NoNaN)];
        let mut definition = make_chaos_definition();
        definition.expected_violations = Some(vec![InvariantKind::BoltInBounds]);
        let result = evaluate_pass(&violations, &[], Some(&definition));
        assert!(
            !result,
            "expected fail when violations=[NoNaN] but expected=[BoltInBounds]"
        );
    }

    // -------------------------------------------------------------------------
    // evaluate_pass — empty expected list with no violations passes
    // -------------------------------------------------------------------------

    /// Some([]) with no violations and no logs is treated as a pass.
    #[test]
    fn evaluate_pass_returns_true_when_expected_violations_empty_and_none_fired() {
        let mut definition = make_chaos_definition();
        definition.expected_violations = Some(vec![]);
        let result = evaluate_pass(&[], &[], Some(&definition));
        assert!(
            result,
            "expected pass when expected=Some([]) and violations=[], logs=[]"
        );
    }

    // -------------------------------------------------------------------------
    // evaluate_pass — logs cause failure even when expected violations match
    // -------------------------------------------------------------------------

    /// Even when violations match expected exactly, captured logs still cause failure.
    #[test]
    fn evaluate_pass_returns_false_when_logs_present_even_though_expected_violations_match() {
        let violations = vec![make_violation(InvariantKind::BoltInBounds)];
        let logs = vec![make_log_entry()];
        let mut definition = make_chaos_definition();
        definition.expected_violations = Some(vec![InvariantKind::BoltInBounds]);
        let result = evaluate_pass(&violations, &logs, Some(&definition));
        assert!(
            !result,
            "expected fail when logs=[one entry] even though violations match expected"
        );
    }
}
