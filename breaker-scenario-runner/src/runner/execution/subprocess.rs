//! Subprocess parallel execution of scenarios.

use std::path::PathBuf;

use super::super::{discovery::scenario_name, run_log::RunLog};

/// Work item for subprocess execution: one subprocess to launch.
pub(in crate::runner) struct SubprocessSpec {
    /// Name shown in output and stored in results.
    pub(in crate::runner) display_name: String,
    /// CLI arguments specific to this work item (e.g. `["-s", "name"]`).
    /// Shared flags (`--visual`, `-v`) are added by
    /// [`super::super::streaming::spawn_streaming`].
    pub(in crate::runner) extra_args:   Vec<String>,
}

/// Result of a single subprocess run.
pub(in crate::runner) struct ChildResult {
    pub(in crate::runner) name:   String,
    pub(in crate::runner) passed: bool,
    pub(in crate::runner) stdout: String,
    pub(in crate::runner) stderr: String,
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
    run_log: Option<&RunLog>,
    fail_fast: bool,
) -> Vec<(String, bool)> {
    let specs: Vec<SubprocessSpec> = runs
        .iter()
        .map(|(display_name, path)| {
            let name = scenario_name(path);
            let mut extra_args = vec!["-s".into(), name];
            if fail_fast {
                extra_args.push("--fail-fast".into());
            }
            SubprocessSpec {
                display_name: display_name.clone(),
                extra_args,
            }
        })
        .collect();

    let all_results =
        match super::super::streaming::spawn_streaming(&specs, visual, verbose, parallelism) {
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

        // Send captured subprocess output to the unified log file.
        if let Some(log) = run_log {
            log.write_lines(result.stdout.lines());
            log.write_lines(result.stderr.lines());
        }
    }

    all_results
        .into_iter()
        .map(|r| (r.name, r.passed))
        .collect()
}
