use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use bevy::{log::LogPlugin, prelude::*, time::TimeUpdateStrategy};
use breaker::game::Game;
use tracing::{info, warn};

use super::{
    discovery::{load_scenario, scenario_name},
    output::{print_compact_failures, print_verbose_failures},
};
use crate::{
    invariants::{ScenarioFrame, ScenarioStats, ViolationEntry, ViolationLog},
    lifecycle::{ScenarioConfig, ScenarioLifecycle},
    log_capture::{CapturedLogs, LogBuffer, LogEntry, poll_log_buffer, scenario_log_layer_factory},
    types::ScenarioDefinition,
    verdict::ScenarioVerdict,
};

/// Bevy's default fixed timestep frequency (Hz).
const FIXED_TIMESTEP_HZ: f64 = 64.0;

/// Speed multiplier for visual mode — each rendered frame advances virtual
/// time by this many fixed timesteps.
const VISUAL_SPEED_MULTIPLIER: f64 = 10.0;

/// Cloned snapshot of evaluation data, captured by a `Last` system so results
/// survive `App::run()` (which replaces self with `App::empty()`).
pub(super) struct EvalSnapshot {
    pub(super) violations: Vec<ViolationEntry>,
    pub(super) logs: Vec<LogEntry>,
    pub(super) stats: ScenarioStats,
    pub(super) definition: ScenarioDefinition,
}

/// Shared buffer inserted as a resource so the snapshot system can write to it
/// and the caller can read it after `app.run()` returns.
#[derive(Resource, Clone)]
pub(super) struct SharedEvalBuffer(pub(super) Arc<Mutex<Option<EvalSnapshot>>>);

/// Snapshots evaluation data every frame into the shared buffer.
///
/// Runs in `Last` so it captures the final state even on the exit frame.
pub(super) fn snapshot_eval_data(
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
            logs: cl.0.clone(),
            stats: stats.clone(),
            definition: config.definition.clone(),
        });
    }
}

/// Non-system version of [`snapshot_eval_data`] for direct world access.
///
/// Called after `drain_remaining_logs` in headless mode to capture the final
/// state including any logs drained after the last `FixedUpdate` tick.
fn snapshot_eval_data_from_world(world: &World, shared: &SharedEvalBuffer) {
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
            logs: cl.0.clone(),
            stats: stats.clone(),
            definition: config.definition.clone(),
        });
    }
}

/// Scenario runner log plugin — captures `WARN`-and-above logs via [`scenario_log_layer_factory`].
fn scenario_log_plugin() -> LogPlugin {
    LogPlugin {
        level: bevy::log::Level::WARN,
        filter: "warn,bevy_egui=error".to_owned(),
        custom_layer: scenario_log_layer_factory,
        ..default()
    }
}

/// Builds a Bevy app configured for scenario running.
///
/// In headless mode, uses [`MinimalPlugins`] with only the specific Bevy
/// plugins the game needs (states, assets, input). This avoids pulling in the
/// full render pipeline, winit event loop, and GPU initialization — none of
/// which are needed when running scenarios at CPU speed on CI. Asset types
/// for headless spawn systems (`Mesh`, `ColorMaterial`, `Font`) are registered by
/// [`Game::headless()`].
///
/// In visual mode, uses [`DefaultPlugins`] for full windowed rendering.
///
/// On the first run (headless or visual), installs `LogPlugin` with a custom
/// tracing layer. On subsequent runs, skips `LogPlugin` (headless: omits it;
/// visual: disables it from `DefaultPlugins`) to avoid the "global logger
/// already set" error — the shared `LogBuffer` is inserted by `run_scenario`
/// instead.
fn build_app(headless: bool, first_run: bool) -> App {
    let mut app = App::new();

    // Point to the game crate's assets directory so scenarios
    // load real RON config files rather than code defaults.
    let game_asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../breaker-game/assets").to_owned();

    if headless {
        // Minimal plugin set — no render pipeline, no window, no GPU.
        // Asset types needed by game spawn systems (Mesh, ColorMaterial, Font)
        // are registered by HeadlessAssetsPlugin inside Game::headless().
        app.add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            bevy::asset::AssetPlugin {
                file_path: game_asset_path,
                ..default()
            },
            bevy::input::InputPlugin,
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
        )))
        .add_plugins(Game::headless());
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

        app.add_plugins(defaults)
            // Visual mode runs at 10x speed to avoid 5+ minute waits for
            // 20,000-frame scenarios. Each Update tick advances virtual time
            // by 10 fixed timesteps (10/64 s).
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                VISUAL_SPEED_MULTIPLIER / FIXED_TIMESTEP_HZ,
            )))
            .add_plugins(Game::default());
    }

    app
}

/// Builds and runs one scenario app. Returns `true` if passed, `false` if failed.
///
/// The `shared_log_buffer` persists across scenarios so the global tracing
/// subscriber (installed once) always writes to the same buffer that each app's
/// `poll_log_buffer` system reads from.
pub(super) fn run_scenario(
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

    let eval_buffer = SharedEvalBuffer(Arc::new(Mutex::new(None)));

    app.insert_resource(ScenarioConfig { definition })
        .add_plugins(ScenarioLifecycle)
        .init_resource::<CapturedLogs>()
        .insert_resource(eval_buffer.clone())
        .add_systems(FixedUpdate, poll_log_buffer)
        .add_systems(Last, snapshot_eval_data);

    if headless {
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

    collect_and_evaluate(&eval_buffer, &sname, verbose)
}

/// Evaluates pass/fail from the shared eval buffer populated by [`snapshot_eval_data`].
///
/// Returns `false` if the buffer is empty (no snapshot captured), any health
/// check fails, any invariant violation is unexpected, or any log was captured.
///
/// Poison recovery on the mutex lock is intentional: if the snapshot writer
/// panicked, we still evaluate whatever partial data was captured (or report
/// the missing-snapshot failure) rather than propagating the panic.
fn collect_and_evaluate(shared: &SharedEvalBuffer, scenario_name: &str, verbose: bool) -> bool {
    let mut verdict = ScenarioVerdict::default();

    let snapshot = shared
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();

    let (violations, logs, stats) = if let Some(snap) = snapshot {
        verdict.evaluate(&snap.violations, &snap.logs, &snap.stats, &snap.definition);
        (snap.violations, snap.logs, snap.stats)
    } else {
        verdict.add_fail_reason("No evaluation data captured during run".into());
        (vec![], vec![], ScenarioStats::default())
    };

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
    use crate::types::{InputStrategy, ScriptedParams};

    #[test]
    fn collect_and_evaluate_fails_when_no_snapshot() {
        let buffer = SharedEvalBuffer(Arc::new(Mutex::new(None)));
        let passed = collect_and_evaluate(&buffer, "test_scenario", false);
        assert!(!passed, "should fail when no snapshot was captured");
    }

    #[test]
    fn collect_and_evaluate_passes_with_clean_snapshot() {
        let definition = ScenarioDefinition {
            breaker: "test".into(),
            layout: "test".into(),
            input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
            max_frames: 100,
            invariants: vec![],
            expected_violations: None,
            debug_setup: None,
            invariant_params: default(),
            allow_early_end: true,
            stress: None,
            seed: None,
            initial_overclocks: None,
        };
        let stats = ScenarioStats {
            actions_injected: 0,
            invariant_checks: 10,
            max_frame: 50,
            entered_playing: true,
            bolts_tagged: 1,
            breakers_tagged: 1,
        };
        let snapshot = EvalSnapshot {
            violations: vec![],
            logs: vec![],
            stats,
            definition,
        };
        let buffer = SharedEvalBuffer(Arc::new(Mutex::new(Some(snapshot))));
        let passed = collect_and_evaluate(&buffer, "test_scenario", false);
        assert!(
            passed,
            "should pass with clean snapshot and empty scripted actions"
        );
    }
}
