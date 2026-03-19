use std::{
    fmt,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use super::{
    app::run_scenario,
    discovery::{collect_scenario_paths, scenario_name},
};
use crate::log_capture::LogBuffer;

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

/// Runs scenarios in-process sequentially. Returns process exit code (0 = all pass, 1 = any fail).
///
/// Shares a single `LogBuffer` across all runs (the global tracing subscriber is
/// installed once). Each scenario's result is printed inline and a summary follows.
#[must_use]
pub fn run_all_serial(runs: &[(String, PathBuf)], headless: bool, verbose: bool) -> i32 {
    let mut shared_log_buffer: Option<LogBuffer> = None;
    let mut results: Vec<(String, bool)> = Vec::with_capacity(runs.len());

    for (display_name, path) in runs {
        let passed = run_scenario(path, headless, verbose, &mut shared_log_buffer);
        results.push((display_name.clone(), passed));
    }

    print_summary(&results)
}

/// Returns the path to the `scenarios/` directory relative to this crate's manifest.
#[must_use]
pub fn scenarios_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scenarios")
}

/// Prints the cross-scenario summary and returns the exit code.
pub(super) fn print_summary(results: &[(String, bool)]) -> i32 {
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

/// Runs scenarios as parallel subprocesses. Returns process exit code.
///
/// Each scenario gets its own child process. `parallelism` is the maximum
/// number of subprocesses to run concurrently (must be >= 1; use
/// [`Parallelism::resolve`] to compute from CLI input).
///
/// The run list is pre-built by the caller via [`build_run_list`].
///
/// # Errors
///
/// Returns exit code `1` if the current executable path cannot be determined.
/// Spawn or wait failures for individual subprocesses are recorded as failed
/// results and do not abort the run.
#[must_use]
pub fn run_all_parallel(
    runs: &[(String, PathBuf)],
    visual: bool,
    verbose: bool,
    parallelism: usize,
) -> i32 {
    let exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to determine current executable path: {e}");
            return 1;
        }
    };
    let mut all_results: Vec<ChildResult> = Vec::with_capacity(runs.len());

    for batch in runs.chunks(parallelism) {
        let mut children: Vec<(String, Child)> = Vec::with_capacity(batch.len());

        for (display_name, path) in batch {
            let name = scenario_name(path);
            let mut cmd = Command::new(&exe);
            cmd.arg("-s").arg(&name);
            if visual {
                cmd.arg("--visual");
            }
            if verbose {
                cmd.arg("-v");
            }
            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

            match cmd.spawn() {
                Ok(child) => children.push((display_name.clone(), child)),
                Err(e) => {
                    eprintln!("Failed to spawn subprocess for [{display_name}]: {e}");
                    all_results.push(ChildResult {
                        name: display_name.clone(),
                        passed: false,
                        stdout: String::new(),
                        stderr: format!("spawn error: {e}"),
                    });
                }
            }
        }

        for (name, child) in children {
            let output = match child.wait_with_output() {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("Failed to wait on child process [{name}]: {e}");
                    all_results.push(ChildResult {
                        name,
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
                name,
                passed: output.status.success(),
                stdout,
                stderr,
            });
        }
    }

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

    let summary: Vec<(String, bool)> = all_results
        .into_iter()
        .map(|r| (r.name, r.passed))
        .collect();
    print_summary(&summary)
}

struct ChildResult {
    name: String,
    passed: bool,
    stdout: String,
    stderr: String,
}
