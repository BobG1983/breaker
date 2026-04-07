//! In-process serial execution, run-list building, and shared utilities.

use std::{
    fmt,
    path::{Path, PathBuf},
};

use super::super::{
    app::run_scenario,
    discovery::{collect_scenario_paths, load_scenario, scenario_name},
    run_log::RunLog,
};
use crate::{log_capture::LogBuffer, types::ScenarioDefinition};

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
/// Each entry is `(display_name, scenario_path, definition)`. For a single
/// scenario, returns one entry; for `--all`, returns one entry per scenario
/// file. Entries whose RON fails to parse are silently filtered out
/// (`load_scenario` prints errors to stderr).
#[must_use]
pub fn build_run_list(
    scenario: Option<&str>,
    all: bool,
) -> Vec<(String, PathBuf, ScenarioDefinition)> {
    let scenario_paths = collect_scenario_paths(scenario, all);
    scenario_paths
        .into_iter()
        .filter_map(|p| {
            let name = scenario_name(&p);
            let def = load_scenario(&p)?;
            Some((name, p, def))
        })
        .collect()
}

/// Runs a single scenario in-process. Returns process exit code (0 = pass, 1 = fail).
#[must_use]
pub fn run_with_args(
    scenario: Option<&str>,
    headless: bool,
    verbose: bool,
    run_log: Option<&RunLog>,
    fail_fast: bool,
) -> i32 {
    let scenario_paths = collect_scenario_paths(scenario, false);

    if scenario_paths.is_empty() {
        eprintln!("No scenarios found. Use -s <name> or --all.");
        return 1;
    }

    let mut shared_log_buffer: Option<LogBuffer> = None;
    let path = &scenario_paths[0];
    let passed = run_scenario(
        path,
        headless,
        verbose,
        &mut shared_log_buffer,
        run_log,
        fail_fast,
    );

    i32::from(!passed)
}

/// Runs a single scenario in-process given its already-resolved path.
///
/// Unlike [`run_with_args`], this does not re-resolve the scenario name to a
/// path — use it when the caller has already located the `.scenario.ron` file.
#[must_use]
pub fn run_single_scenario(
    path: &Path,
    headless: bool,
    verbose: bool,
    run_log: Option<&RunLog>,
    fail_fast: bool,
) -> i32 {
    let mut shared_log_buffer: Option<LogBuffer> = None;
    let passed = run_scenario(
        path,
        headless,
        verbose,
        &mut shared_log_buffer,
        run_log,
        fail_fast,
    );
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
    run_log: Option<&RunLog>,
    fail_fast: bool,
) -> Vec<(String, bool)> {
    let mut shared_log_buffer: Option<LogBuffer> = None;
    let mut results: Vec<(String, bool)> = Vec::with_capacity(runs.len());

    for (display_name, path) in runs {
        let passed = run_scenario(
            path,
            headless,
            verbose,
            &mut shared_log_buffer,
            run_log,
            fail_fast,
        );
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
