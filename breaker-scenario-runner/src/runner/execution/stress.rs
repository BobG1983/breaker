//! Stress scenario support: multi-copy runs and result aggregation.

use std::path::PathBuf;

use super::{
    super::run_log::RunLog,
    subprocess::{SubprocessSpec, spawn_batched},
};
use crate::types::{ScenarioDefinition, StressConfig};

/// A single failed stress copy's output.
pub struct StressFailure {
    /// Zero-based index of this copy within the stress run.
    pub copy_index: usize,
    /// Captured stdout from the child process.
    pub stdout: String,
    /// Captured stderr from the child process.
    pub stderr: String,
}

/// Result of running a stress scenario (multiple copies of the same scenario).
pub struct StressResult {
    /// Name of the scenario under stress.
    pub name: String,
    /// Total number of copies that were run.
    pub total: usize,
    /// Details for every copy that failed.
    pub failures: Vec<StressFailure>,
}

impl StressResult {
    /// Returns `true` if every copy of the stress run passed.
    #[must_use]
    pub const fn passed(&self) -> bool {
        self.failures.is_empty()
    }

    /// Number of copies that passed (derived from total - failures).
    #[must_use]
    pub const fn pass_count(&self) -> usize {
        self.total - self.failures.len()
    }

    /// Returns a human-readable summary line, e.g. `"32/32 passed"` or
    /// `"28/32 passed (4 failures)"`.
    #[must_use]
    pub fn summary_line(&self) -> String {
        let pass_count = self.pass_count();
        if self.failures.is_empty() {
            format!("{pass_count}/{} passed", self.total)
        } else {
            format!(
                "{pass_count}/{} passed ({} failures)",
                self.total,
                self.failures.len()
            )
        }
    }
}

/// A normal scenario run entry: `(name, path)`.
pub(in crate::runner) type NormalRun = (String, PathBuf);

/// A stress scenario run entry: `(name, path, stress_config)`.
pub(in crate::runner) type StressRun = (String, PathBuf, StressConfig);

/// Partitions a run list into `(normal, stress)` scenarios by reading the
/// `stress` field from each pre-parsed [`ScenarioDefinition`].
///
/// Returns `(normal_runs, stress_runs)` where `stress_runs` includes the
/// resolved [`StressConfig`] for each stress scenario.
#[must_use]
pub fn partition_stress_scenarios(
    runs: &[(String, PathBuf, ScenarioDefinition)],
) -> (Vec<NormalRun>, Vec<StressRun>) {
    let mut normal = Vec::new();
    let mut stress = Vec::new();

    for (name, path, def) in runs {
        match &def.stress {
            Some(config) => stress.push((name.clone(), path.clone(), config.clone())),
            None => normal.push((name.clone(), path.clone())),
        }
    }

    (normal, stress)
}

/// Runs a stress scenario by spawning `config.runs` copies as subprocesses,
/// batched by `config.parallelism`.
///
/// Each subprocess gets `--stress-copy` so it runs in single in-process mode
/// without recursively expanding stress config.
///
/// Returns a [`StressResult`] aggregating pass/fail across all copies.
#[must_use]
pub fn run_stress_scenario(
    name: &str,
    config: &StressConfig,
    visual: bool,
    verbose: bool,
    run_log: Option<&RunLog>,
    fail_fast: bool,
) -> StressResult {
    let runs = config.runs.max(1);
    let parallelism = config.parallelism.max(1);

    let specs: Vec<SubprocessSpec> = (0..runs)
        .map(|i| {
            let mut extra_args = vec!["-s".into(), name.into(), "--stress-copy".into()];
            if fail_fast {
                extra_args.push("--fail-fast".into());
            }
            SubprocessSpec {
                display_name: format!("copy_{i}"),
                extra_args,
            }
        })
        .collect();

    let all_results = match spawn_batched(&specs, visual, verbose, parallelism) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("{e}");
            return StressResult {
                name: name.to_owned(),
                total: runs,
                failures: vec![StressFailure {
                    copy_index: 0,
                    stdout: String::new(),
                    stderr: e,
                }],
            };
        }
    };

    // Send captured subprocess output to the unified log file.
    if let Some(log) = run_log {
        for result in &all_results {
            log.write_lines(result.stdout.lines());
            log.write_lines(result.stderr.lines());
        }
    }

    let failures: Vec<StressFailure> = all_results
        .into_iter()
        .filter(|r| !r.passed)
        .map(|r| {
            let copy_index = r
                .name
                .strip_prefix("copy_")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            StressFailure {
                copy_index,
                stdout: r.stdout,
                stderr: r.stderr,
            }
        })
        .collect();

    StressResult {
        name: name.to_owned(),
        total: runs,
        failures,
    }
}

/// Prints the result of a stress scenario run.
///
/// Failure stdout and stderr are always printed for failed copies.
pub fn print_stress_result(result: &StressResult, run_log: Option<&RunLog>) {
    let summary = format!("[{}] stress: {}", result.name, result.summary_line());
    println!("{summary}");
    if let Some(log) = run_log {
        log.write_line(&summary);
    }

    if !result.passed() {
        for failure in &result.failures {
            let copy_line = format!("  Copy {}:", failure.copy_index);
            println!("{copy_line}");
            if let Some(log) = run_log {
                log.write_line(&copy_line);
            }
            for line in failure.stdout.lines() {
                println!("    {line}");
                if let Some(log) = run_log {
                    log.write_line(&format!("    {line}"));
                }
            }
            if !failure.stderr.is_empty() {
                for line in failure.stderr.lines() {
                    eprintln!("    {line}");
                    if let Some(log) = run_log {
                        log.write_line(&format!("    {line}"));
                    }
                }
            }
        }
    }
    println!();
}
