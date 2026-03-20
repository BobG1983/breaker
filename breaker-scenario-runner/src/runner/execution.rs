//! Subprocess batching, in-process serial/parallel execution, and stress-run
//! aggregation for the scenario runner.

use std::{
    fmt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use super::{
    app::run_scenario,
    discovery::{collect_scenario_paths, load_stress_config, scenario_name},
};
use crate::{log_capture::LogBuffer, types::StressConfig};

/// Parallelism level for subprocess-based execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parallelism {
    /// Run at most N subprocesses concurrently.
    Count(usize),
    /// Run all subprocesses at once (unlimited concurrency).
    All,
}

impl Parallelism {
    /// Default parallelism when no `--parallel` flag is given.
    pub const DEFAULT: Self = Self::Count(32);

    /// Resolves to a concrete batch size given the total number of runs.
    #[must_use]
    pub fn resolve(&self, total: usize) -> usize {
        match self {
            Self::Count(n) => (*n).max(1),
            Self::All => total.max(1),
        }
    }
}

impl fmt::Display for Parallelism {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Count(n) => write!(f, "{n}"),
            Self::All => write!(f, "all"),
        }
    }
}

/// Parses a `--parallel` value: either `"all"` or a positive integer.
///
/// # Errors
///
/// Returns a descriptive error string if the value is not `"all"` or a positive integer.
pub fn parse_parallelism(value: &str) -> Result<Parallelism, String> {
    if value.eq_ignore_ascii_case("all") {
        return Ok(Parallelism::All);
    }
    match value.parse::<usize>() {
        Ok(0) => Err("--parallel must be a positive number or \"all\"".to_owned()),
        Ok(n) => Ok(Parallelism::Count(n)),
        Err(_) => Err(format!(
            "invalid value for --parallel: \"{value}\" (expected a positive number or \"all\")"
        )),
    }
}

/// Builds the run list for a single iteration of execution.
///
/// Each entry is `(display_name, scenario_path)`. For a single scenario,
/// returns one entry; for `--all`, returns one entry per scenario file.
#[must_use]
pub fn build_run_list(scenario: Option<&str>, all: bool) -> Vec<(String, PathBuf)> {
    let scenario_paths = collect_scenario_paths(scenario, all);
    scenario_paths
        .into_iter()
        .map(|p| {
            let name = scenario_name(&p);
            (name, p)
        })
        .collect()
}

/// Runs a single scenario in-process. Returns process exit code (0 = pass, 1 = fail).
#[must_use]
pub fn run_with_args(scenario: Option<&str>, headless: bool, verbose: bool) -> i32 {
    let scenario_paths = collect_scenario_paths(scenario, false);

    if scenario_paths.is_empty() {
        eprintln!("No scenarios found. Use -s <name> or --all.");
        return 1;
    }

    let mut shared_log_buffer: Option<LogBuffer> = None;
    let path = &scenario_paths[0];
    let passed = run_scenario(path, headless, verbose, &mut shared_log_buffer);

    i32::from(!passed)
}

/// Runs a single scenario in-process given its already-resolved path.
///
/// Unlike [`run_with_args`], this does not re-resolve the scenario name to a
/// path — use it when the caller has already located the `.scenario.ron` file.
#[must_use]
pub fn run_single_scenario(path: &Path, headless: bool, verbose: bool) -> i32 {
    let mut shared_log_buffer: Option<LogBuffer> = None;
    let passed = run_scenario(path, headless, verbose, &mut shared_log_buffer);
    i32::from(!passed)
}

/// Runs scenarios in-process sequentially. Returns per-scenario pass/fail results.
///
/// Shares a single `LogBuffer` across all runs (the global tracing subscriber is
/// installed once). Each scenario's result is printed inline. The caller is
/// responsible for aggregating results and printing a summary.
#[must_use]
pub fn run_all_serial(
    runs: &[(String, PathBuf)],
    headless: bool,
    verbose: bool,
) -> Vec<(String, bool)> {
    let mut shared_log_buffer: Option<LogBuffer> = None;
    let mut results: Vec<(String, bool)> = Vec::with_capacity(runs.len());

    for (display_name, path) in runs {
        let passed = run_scenario(path, headless, verbose, &mut shared_log_buffer);
        results.push((display_name.clone(), passed));
    }

    results
}

/// Returns the path to the `scenarios/` directory relative to this crate's manifest.
#[must_use]
pub fn scenarios_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scenarios")
}

/// Prints the cross-scenario summary and returns the exit code.
#[must_use]
pub fn print_summary(results: &[(String, bool)]) -> i32 {
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

// -------------------------------------------------------------------------
// Subprocess batched execution
// -------------------------------------------------------------------------

/// Work item for [`spawn_batched`]: one subprocess to launch.
struct SubprocessSpec {
    /// Name shown in output and stored in results.
    display_name: String,
    /// CLI arguments specific to this work item (e.g. `["-s", "name"]`).
    /// Shared flags (`--visual`, `-v`) are added by [`spawn_batched`].
    extra_args: Vec<String>,
}

/// Result of a single subprocess run.
struct ChildResult {
    name: String,
    passed: bool,
    stdout: String,
    stderr: String,
}

/// Spawns subprocesses in batches and collects results.
///
/// Each [`SubprocessSpec`] becomes one child process. Results are returned in
/// the same order as the input specs — spawn and wait errors are recorded as
/// failures inline (they do not abort the run).
///
/// Returns `Err` only if `current_exe()` fails (no subprocess can be spawned).
fn spawn_batched(
    specs: &[SubprocessSpec],
    visual: bool,
    verbose: bool,
    parallelism: usize,
) -> Result<Vec<ChildResult>, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Failed to determine current executable path: {e}"))?;

    let mut all_results: Vec<ChildResult> = Vec::with_capacity(specs.len());

    for batch in specs.chunks(parallelism) {
        let mut children: Vec<(&SubprocessSpec, Child)> = Vec::with_capacity(batch.len());

        for spec in batch {
            let mut cmd = Command::new(&exe);
            for arg in &spec.extra_args {
                cmd.arg(arg);
            }
            if visual {
                cmd.arg("--visual");
            }
            if verbose {
                cmd.arg("-v");
            }
            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

            match cmd.spawn() {
                Ok(child) => children.push((spec, child)),
                Err(e) => {
                    eprintln!(
                        "Failed to spawn subprocess for [{}]: {e}",
                        spec.display_name
                    );
                    all_results.push(ChildResult {
                        name: spec.display_name.clone(),
                        passed: false,
                        stdout: String::new(),
                        stderr: format!("spawn error: {e}"),
                    });
                }
            }
        }

        for (spec, child) in children {
            let output = match child.wait_with_output() {
                Ok(o) => o,
                Err(e) => {
                    eprintln!(
                        "Failed to wait on child process [{}]: {e}",
                        spec.display_name
                    );
                    all_results.push(ChildResult {
                        name: spec.display_name.clone(),
                        passed: false,
                        stdout: String::new(),
                        stderr: format!("wait error: {e}"),
                    });
                    continue;
                }
            };

            let stdout = String::from_utf8(output.stdout)
                .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());
            let stderr = String::from_utf8(output.stderr)
                .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());

            all_results.push(ChildResult {
                name: spec.display_name.clone(),
                passed: output.status.success(),
                stdout,
                stderr,
            });
        }
    }

    Ok(all_results)
}

/// Runs scenarios as parallel subprocesses. Returns per-scenario pass/fail results.
///
/// Each scenario gets its own child process. `parallelism` is the maximum
/// number of subprocesses to run concurrently (must be >= 1; use
/// [`Parallelism::resolve`] to compute from CLI input).
///
/// The run list is pre-built by the caller via [`build_run_list`].
///
/// If the current executable path cannot be determined, returns a single
/// failed entry so the caller can still produce a summary.
///
/// Spawn or wait failures for individual subprocesses are recorded as failed
/// results and do not abort the run.
#[must_use]
pub fn run_all_parallel(
    runs: &[(String, PathBuf)],
    visual: bool,
    verbose: bool,
    parallelism: usize,
) -> Vec<(String, bool)> {
    let specs: Vec<SubprocessSpec> = runs
        .iter()
        .map(|(display_name, path)| {
            let name = scenario_name(path);
            SubprocessSpec {
                display_name: display_name.clone(),
                extra_args: vec!["-s".into(), name],
            }
        })
        .collect();

    let all_results = match spawn_batched(&specs, visual, verbose, parallelism) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("{e}");
            return vec![("(subprocess error)".to_owned(), false)];
        }
    };

    // Print output in original scenario order (results collected per batch in spawn order).
    for result in &all_results {
        println!("[{}]", result.name);
        for line in result.stdout.lines() {
            println!("  {line}");
        }
        if !result.stderr.is_empty() {
            for line in result.stderr.lines() {
                eprintln!("  {line}");
            }
        }
        println!();
    }

    all_results
        .into_iter()
        .map(|r| (r.name, r.passed))
        .collect()
}

// -------------------------------------------------------------------------
// StressResult / StressFailure — stress-run aggregation
// -------------------------------------------------------------------------

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
pub(super) type NormalRun = (String, PathBuf);

/// A stress scenario run entry: `(name, path, stress_config)`.
pub(super) type StressRun = (String, PathBuf, StressConfig);

/// Partitions a run list into `(normal, stress)` scenarios by checking each
/// RON file for a `stress` field.
///
/// Returns `(normal_runs, stress_runs)` where `stress_runs` includes the
/// resolved [`StressConfig`] for each stress scenario.
#[must_use]
pub fn partition_stress_scenarios(runs: &[(String, PathBuf)]) -> (Vec<NormalRun>, Vec<StressRun>) {
    let mut normal = Vec::new();
    let mut stress = Vec::new();

    for (name, path) in runs {
        match load_stress_config(path) {
            Some(config) => stress.push((name.clone(), path.clone(), config)),
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
) -> StressResult {
    let runs = config.runs.max(1);
    let parallelism = config.parallelism.max(1);

    let specs: Vec<SubprocessSpec> = (0..runs)
        .map(|i| SubprocessSpec {
            display_name: format!("copy_{i}"),
            extra_args: vec!["-s".into(), name.into(), "--stress-copy".into()],
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
pub fn print_stress_result(result: &StressResult) {
    println!("[{}] stress: {}", result.name, result.summary_line());

    if !result.passed() {
        for failure in &result.failures {
            println!("  Copy {}:", failure.copy_index);
            for line in failure.stdout.lines() {
                println!("    {line}");
            }
            if !failure.stderr.is_empty() {
                for line in failure.stderr.lines() {
                    eprintln!("    {line}");
                }
            }
        }
    }
    println!();
}
