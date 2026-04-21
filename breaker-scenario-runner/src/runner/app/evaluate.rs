//! Post-run evaluation and scenario-loop helpers.
//!
//! - [`collect_and_evaluate`] consumes the eval snapshot and prints pass/fail.
//! - [`should_fail_fast`] decides mid-run whether to stop early on violations.
//! - [`is_timed_out`], [`drain_remaining_logs`], and [`guarded_update`] are the
//!   building blocks of the headless run loop in [`super::run::run_scenario`].

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;
use tracing::{info, warn};

use super::types::SharedEvalBuffer;
use crate::{
    invariants::{ScenarioFrame, ScenarioStats, ViolationLog},
    log_capture::{CapturedLogs, LogBuffer, LogEntry},
    runner::{
        execution::scenarios_dir,
        output::{format_verbose_failures, print_compact_failures, print_verbose_failures},
        run_log::RunLog,
    },
    types::{InputStrategy, ScenarioDefinition, ScriptedFrame, ScriptedParams},
    verdict::ScenarioVerdict,
};

/// Evaluates pass/fail from the shared eval buffer populated by
/// [`super::types::snapshot_eval_data`].
///
/// Returns `false` if the buffer is empty (no snapshot captured), any health
/// check fails, any invariant violation is unexpected, or any log was captured.
///
/// Poison recovery on the mutex lock is intentional: if the snapshot writer
/// panicked, we still evaluate whatever partial data was captured (or report
/// the missing-snapshot failure) rather than propagating the panic.
pub(crate) fn collect_and_evaluate(
    shared: &SharedEvalBuffer,
    scenario_name: &str,
    verbose: bool,
    run_log: Option<&RunLog>,
) -> bool {
    let mut verdict = ScenarioVerdict::default();

    let snapshot = shared
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();

    let (violations, logs, stats, pin_data) = if let Some(snap) = snapshot {
        verdict.evaluate(&snap.violations, &snap.logs, &snap.stats, &snap.definition);
        let pin = (snap.definition, snap.chaos_input_log);
        (snap.violations, snap.logs, snap.stats, Some(pin))
    } else {
        verdict.add_fail_reason("No evaluation data captured during run".into());
        (vec![], vec![], ScenarioStats::default(), None)
    };

    let stats_line = format!(
        "  [{scenario_name}] frames={} actions={} violations={} logs={} bolts={} breakers={} entered_playing={}",
        stats.max_frame,
        stats.actions_injected,
        violations.len(),
        logs.len(),
        stats.bolts_tagged,
        stats.breakers_tagged,
        stats.entered_playing
    );
    println!("{stats_line}");
    if let Some(log) = run_log {
        log.write_line(&stats_line);
    }

    if verdict.passed() {
        let pass_line = format!("PASS [{scenario_name}]");
        println!("{pass_line}");
        if let Some(log) = run_log {
            log.write_line(&pass_line);
        }
        info!(target: "breaker_scenario_runner", "scenario pass name={scenario_name}");
    } else {
        let reason_count = verdict.reasons.len();
        let fail_line = format!("FAIL [{scenario_name}]: {reason_count} failure(s)");
        println!("{fail_line}");
        if let Some(log) = run_log {
            log.write_line(&fail_line);
        }
        warn!(
            target: "breaker_scenario_runner",
            "scenario fail name={scenario_name} reasons={reason_count}",
        );

        // Always send verbose output to log file.
        if let Some(log) = run_log {
            let verbose_text = format_verbose_failures(scenario_name, &verdict, &violations, &logs);
            log.write_lines(verbose_text.lines());
        }

        // Print to stdout based on verbose flag.
        if verbose {
            print_verbose_failures(scenario_name, &verdict, &violations, &logs);
        } else {
            print_compact_failures(&verdict, &violations, &logs);
        }

        if let Some((definition, chaos_log)) = pin_data {
            try_write_chaos_regression(&scenarios_dir(), scenario_name, &definition, chaos_log);
        }
    }

    verdict.passed()
}

/// Attempts to write a replayable scripted scenario to
/// `<scenarios_dir>/regressions/` when the failed scenario used a chaos
/// strategy and the chaos driver emitted at least one action during the run.
///
/// Silently no-ops for non-chaos strategies or when no actions were recorded.
/// Errors (filesystem, serialization) print a warning but do not panic — a
/// failed write must never mask the underlying invariant violation.
fn try_write_chaos_regression(
    scenarios_root: &Path,
    scenario_name: &str,
    definition: &ScenarioDefinition,
    chaos_log: Option<Vec<ScriptedFrame>>,
) {
    let uses_chaos = matches!(
        definition.input,
        InputStrategy::Chaos(_) | InputStrategy::Hybrid(_)
    );
    if !uses_chaos {
        return;
    }
    let Some(log) = chaos_log else {
        return;
    };
    if log.is_empty() {
        return;
    }

    match write_chaos_regression(scenarios_root, scenario_name, definition, log) {
        Ok(path) => {
            if let Ok(rel) = path.strip_prefix(scenarios_root) {
                println!("REGRESSION PINNED: scenarios/{}", rel.display());
            } else {
                println!("REGRESSION PINNED: {}", path.display());
            }
        }
        Err(e) => {
            warn!(
                target: "breaker_scenario_runner",
                "failed to write chaos regression for {scenario_name}: {e}"
            );
        }
    }
}

/// Serializes a replacement [`ScenarioDefinition`] — scripted input cloned
/// from the chaos log, with the original seed dropped — and writes it to
/// `<scenarios_root>/regressions/<timestamp>-chaos-<scenario_name>.scenario.ron`.
///
/// Returns the absolute path of the written file. Creates the `regressions/`
/// directory if needed.
pub(crate) fn write_chaos_regression(
    scenarios_root: &Path,
    scenario_name: &str,
    original: &ScenarioDefinition,
    chaos_log: Vec<ScriptedFrame>,
) -> Result<PathBuf, String> {
    let mut replay = original.clone();
    // Both `Chaos` and `Hybrid` live-produce their actions — neither
    // stores scripted frames on the definition (see `HybridParams`: it
    // is `{ scripted_frames: u32, action_prob: f32 }` — a silent-warmup
    // frame count, not a frame list). The chaos log is therefore the
    // complete input stream for the replay.
    replay.input = InputStrategy::Scripted(ScriptedParams { actions: chaos_log });
    replay.seed = None;

    let pretty = ron::ser::PrettyConfig::new();
    let serialized = ron::ser::to_string_pretty(&replay, pretty)
        .map_err(|e| format!("serialize failed: {e}"))?;

    let regressions_dir = scenarios_root.join("regressions");
    fs::create_dir_all(&regressions_dir)
        .map_err(|e| format!("create_dir_all {} failed: {e}", regressions_dir.display()))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    // Sanitize `scenario_name` before using it as a filename component to
    // prevent path traversal via `..`, embedded separators, or dotfile tricks.
    // Replace anything that isn't `[A-Za-z0-9_-]` with `_`.
    let safe_name: String = scenario_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let filename = format!("{timestamp}-chaos-{safe_name}.scenario.ron");
    let path = regressions_dir.join(filename);

    fs::write(&path, serialized).map_err(|e| format!("write {} failed: {e}", path.display()))?;
    Ok(path)
}

/// Returns `true` when the scenario should exit early due to `--fail-fast`.
///
/// Triggers when:
/// - `fail_fast` flag is `true`
/// - `violation_log` contains at least one violation whose invariant kind
///   is NOT in `definition.allowed_failures`
///
/// A self-test scenario with `allowed_failures: Some([BoltInBounds])` will
/// still fail-fast if an unexpected `NoNaN` violation occurs.
#[must_use]
pub(crate) fn should_fail_fast(
    violation_log: &ViolationLog,
    definition: &ScenarioDefinition,
    fail_fast: bool,
) -> bool {
    fail_fast
        && violation_log.0.iter().any(|v| {
            definition
                .allowed_failures
                .as_ref()
                .is_none_or(|af| !af.contains(&v.invariant))
        })
}

/// Returns `true` if `start` elapsed longer ago than `timeout`.
///
/// Used by the run loop to detect wall-clock timeouts without blocking.
#[must_use]
pub(crate) fn is_timed_out(start: Instant, timeout: Duration) -> bool {
    start.elapsed() > timeout
}

/// Drains any buffered log entries from [`LogBuffer`] into [`CapturedLogs`].
///
/// Called after the run loop exits to ensure entries captured after the last
/// `poll_log_buffer` tick are not silently discarded.
pub(crate) fn drain_remaining_logs(app: &mut App) {
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
pub(crate) fn guarded_update(app: &mut App) -> Result<(), String> {
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
