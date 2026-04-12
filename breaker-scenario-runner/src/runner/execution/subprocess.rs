//! Subprocess batching and parallel execution of scenarios.

use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use super::super::{discovery::scenario_name, run_log::RunLog};

/// Work item for subprocess execution: one subprocess to launch.
pub(in crate::runner) struct SubprocessSpec {
    /// Name shown in output and stored in results.
    pub(in crate::runner) display_name: String,
    /// CLI arguments specific to this work item (e.g. `["-s", "name"]`).
    /// Shared flags (`--visual`, `-v`) are added by [`spawn_batched`].
    pub(in crate::runner) extra_args:   Vec<String>,
}

/// Result of a single subprocess run.
pub(in crate::runner) struct ChildResult {
    pub(in crate::runner) name:   String,
    pub(in crate::runner) passed: bool,
    pub(in crate::runner) stdout: String,
    pub(in crate::runner) stderr: String,
}

/// Spawns subprocesses in batches and collects results.
///
/// Each [`SubprocessSpec`] becomes one child process. Results are returned in
/// the same order as the input specs — spawn and wait errors are recorded as
/// failures inline (they do not abort the run).
///
/// Returns `Err` only if `current_exe()` fails (no subprocess can be spawned).
pub(in crate::runner) fn spawn_batched(
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
                        name:   spec.display_name.clone(),
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
                        name:   spec.display_name.clone(),
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
