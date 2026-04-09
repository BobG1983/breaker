//! Scenario runner — automated gameplay testing tool.
//!
//! Runs headless by default (no GPU required). Pass `--visual` to open a window
//! with full graphics at 10x speed for debugging.
//!
//! Usage:
//!   `cargo scenario -- -s aegis_chaos`
//!   `cargo scenario -- --all`
//!   `cargo scenario -- --visual -s aegis_chaos`
//!   `cargo scenario -- --all --visual`
//!   `cargo scenario -- --all -p 4`
//!   `cargo scenario -- --all --serial`
//!   `cargo scenario -- --all --loop 3`
//!   `cargo scenario -- --coverage`

use std::{
    path::{Path, PathBuf},
    process,
};

use breaker_scenario_runner::{
    coverage::{check_coverage, print_coverage_report},
    runner::{
        self, Parallelism, build_run_list, parse_parallelism, partition_stress_scenarios,
        print_stress_result, print_summary, run_all_parallel, run_all_serial,
        run_log::{RunLog, create_run_log, format_log_path_message},
        run_single_scenario, run_stress_scenario, run_with_args,
    },
};
use clap::Parser;

fn main() {
    // Handle --clean and --coverage before full CLI parsing to keep Args under 3 bools.
    if std::env::args().any(|a| a == "--clean") {
        if let Err(e) =
            runner::output_dir::clean_output_dir(Path::new(runner::output_dir::BASE_DIR))
        {
            eprintln!("Failed to clean output directory: {e}");
            process::exit(1);
        }
        println!("Cleaned {}", runner::output_dir::BASE_DIR);
        process::exit(0);
    }
    if std::env::args().any(|a| a == "--coverage") {
        let runs = build_run_list(None, true);
        if runs.is_empty() {
            eprintln!("No scenarios found.");
            process::exit(1);
        }
        print_coverage_for_runs(&runs);
        process::exit(0);
    }

    let args = Args::parse();
    let fail_fast = resolve_fail_fast(
        args.fail_fast_mode.fail_fast,
        args.fail_fast_mode.no_fail_fast,
        args.all,
    );

    if args.visual && !args.all && args.scenario.is_none() {
        eprintln!("--visual requires -s <scenario_name> or --all");
        process::exit(1);
    }

    let parallelism = args
        .execution
        .parallel
        .as_deref()
        .map_or(Parallelism::DEFAULT, |value| {
            parse_parallelism(value).unwrap_or_else(|e| {
                eprintln!("{e}");
                process::exit(1);
            })
        });

    let loop_count = args.loops.unwrap_or(1);
    let headless = !args.visual;

    // Stress-copy subprocess: always run single in-process, ignore stress fields.
    if args.execution.stress_copy {
        let exit_code = run_with_args(
            args.scenario.as_deref(),
            headless,
            args.verbose,
            None,
            fail_fast,
        );
        process::exit(exit_code);
    }

    // Fast path: single scenario, no loop → check for stress config first.
    if args.scenario.is_some() && !args.all && loop_count == 1 && !args.execution.serial {
        run_single_fast_path(&args, headless, fail_fast);
    }

    let runs = build_run_list(args.scenario.as_deref(), args.all);
    if runs.is_empty() {
        eprintln!("No scenarios found. Use -s <name> or --all.");
        process::exit(1);
    }

    // Visual + serial with multiple total runs is unsupported (Winit event loop runs once).
    let total_runs = runs.len() * loop_count;
    if args.visual && args.execution.serial && total_runs > 1 {
        eprintln!(
            "--visual with --serial is not supported for multiple runs (Winit event loop can only run once)"
        );
        process::exit(1);
    }

    // Partition into normal and stress scenarios.
    let (normal_runs, stress_runs) = partition_stress_scenarios(&runs);

    if args.execution.serial && !stress_runs.is_empty() {
        let stress_names: Vec<&str> = stress_runs.iter().map(|(n, ..)| n.as_str()).collect();
        eprintln!(
            "note: --serial applies to normal scenarios only; {} stress scenario(s) will still use parallel subprocesses: {}",
            stress_runs.len(),
            stress_names.join(", ")
        );
    }

    // Print coverage report before running scenarios.
    if args.all {
        print_coverage_for_runs(&runs);
    }

    let run_log = create_run_log_if_needed(args.all || args.scenario.is_some());

    let worst_exit = run_loop(
        &normal_runs,
        &stress_runs,
        loop_count,
        &args,
        parallelism,
        run_log.as_ref(),
        fail_fast,
    );

    print_log_path_and_shutdown(run_log);

    process::exit(worst_exit);
}

/// Fast path for a single scenario (no `--all`, no `--loop`, no `--serial`).
///
/// Resolves the scenario path, checks for stress config, and runs in-process.
/// Calls `process::exit` — does not return.
fn run_single_fast_path(args: &Args, headless: bool, fail_fast: bool) {
    let runs = build_run_list(args.scenario.as_deref(), false);
    if runs.is_empty() {
        eprintln!("No scenarios found. Use -s <name> or --all.");
        process::exit(1);
    }

    let run_log = None;

    let (normal, stress) = partition_stress_scenarios(&runs);
    if let Some((name, _path, config)) = stress.into_iter().next() {
        let result = run_stress_scenario(
            &name,
            &config,
            args.visual,
            args.verbose,
            run_log.as_ref(),
            fail_fast,
        );
        print_stress_result(&result, run_log.as_ref());
        print_log_path_and_shutdown(run_log);
        process::exit(i32::from(!result.passed()));
    }

    if !normal.is_empty() {
        let exit_code = run_single_scenario(
            &normal[0].1,
            headless,
            args.verbose,
            run_log.as_ref(),
            fail_fast,
        );
        print_log_path_and_shutdown(run_log);
        process::exit(exit_code);
    }
}

/// Runs normal + stress scenarios for `loop_count` iterations. Returns worst exit code.
fn run_loop(
    normal_runs: &[(String, PathBuf)],
    stress_runs: &[(
        String,
        PathBuf,
        breaker_scenario_runner::types::StressConfig,
    )],
    loop_count: usize,
    args: &Args,
    parallelism: Parallelism,
    run_log: Option<&RunLog>,
    fail_fast: bool,
) -> i32 {
    let headless = !args.visual;
    let mut worst_exit = 0;
    for iteration in 1..=loop_count {
        if loop_count > 1 {
            println!("\n=== Loop {iteration}/{loop_count} ===");
        }

        let mut all_results: Vec<(String, bool)> = Vec::new();

        if !normal_runs.is_empty() {
            let results = if args.execution.serial {
                run_all_serial(normal_runs, headless, args.verbose, run_log, fail_fast)
            } else {
                let batch_size = parallelism.resolve(normal_runs.len());
                run_all_parallel(
                    normal_runs,
                    args.visual,
                    args.verbose,
                    batch_size,
                    run_log,
                    fail_fast,
                )
            };
            all_results.extend(results);
        }

        for (name, _path, config) in stress_runs {
            let result =
                run_stress_scenario(name, config, args.visual, args.verbose, run_log, fail_fast);
            print_stress_result(&result, run_log);
            all_results.push((name.clone(), result.passed()));
        }

        let exit_code = print_summary(&all_results);
        if exit_code > worst_exit {
            worst_exit = exit_code;
        }
    }
    worst_exit
}

/// Creates a `RunLog` if needed. Returns `None` on failure (warning printed to stderr).
fn create_run_log_if_needed(should_create: bool) -> Option<RunLog> {
    if !should_create {
        return None;
    }
    match create_run_log(Path::new(runner::output_dir::BASE_DIR)) {
        Ok(log) => Some(log),
        Err(e) => {
            eprintln!("warning: failed to create run log: {e}");
            None
        }
    }
}

/// Flushes, prints log path, and shuts down the `RunLog` if present.
fn print_log_path_and_shutdown(run_log: Option<RunLog>) {
    if let Some(log) = run_log {
        log.flush();
        println!("{}", format_log_path_message(log.path()));
        log.shutdown();
    }
}

/// Uses pre-parsed scenario definitions to identify self-test scenarios,
/// discover layout files, and print the coverage report.
fn print_coverage_for_runs(
    runs: &[(
        String,
        PathBuf,
        breaker_scenario_runner::types::ScenarioDefinition,
    )],
) {
    use breaker_scenario_runner::runner::scenarios_dir;

    let scenarios: Vec<(String, breaker_scenario_runner::types::ScenarioDefinition)> = runs
        .iter()
        .map(|(name, _path, def)| (name.clone(), def.clone()))
        .collect();

    let self_tests_dir = scenarios_dir().join("self_tests");
    let self_test_names: Vec<String> = runs
        .iter()
        .filter(|(_, path, _)| path.starts_with(&self_tests_dir))
        .map(|(name, ..)| name.clone())
        .collect();

    let layout_names = discover_layout_names();

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);
    println!();
    let _ = print_coverage_report(&report, true);
}

/// Discovers layout names from `.node.ron` files in the game assets directory.
fn discover_layout_names() -> Vec<String> {
    let nodes_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../breaker-game/assets/nodes");
    let Ok(entries) = std::fs::read_dir(&nodes_dir) else {
        return vec![];
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?;
            if name.ends_with(".node.ron") {
                let stem = name.strip_suffix(".node.ron")?;
                Some(stem.to_owned())
            } else {
                None
            }
        })
        .collect()
}

/// Automated gameplay scenario runner.
#[derive(Parser)]
#[command(about = "Automated gameplay scenario runner")]
struct Args {
    /// Scenario name to run (stem of a `.scenario.ron` file in `scenarios/`)
    #[arg(short = 's', long)]
    scenario: Option<String>,

    /// Run all scenarios in the `scenarios/` directory tree
    #[arg(long)]
    all: bool,

    /// Run with a window for visual debugging
    #[arg(long)]
    visual: bool,

    /// Print all violations and logs verbatim (default: grouped compact output)
    #[arg(short = 'v', long)]
    verbose: bool,

    #[command(flatten)]
    fail_fast_mode: FailFastMode,

    #[command(flatten)]
    execution: ExecutionMode,

    /// Repeat the entire run N times
    #[arg(short = 'l', long = "loop", value_parser = parse_loop_count)]
    loops: Option<usize>,
}

/// Fail-fast mode: `--fail-fast` or `--no-fail-fast` (clap enforces mutual exclusion via `overrides_with`).
#[derive(clap::Args)]
struct FailFastMode {
    /// Stop scenario on first invariant violation (default: on for `--all`, off for `-s`)
    #[arg(long, overrides_with = "no_fail_fast")]
    fail_fast: bool,

    /// Run scenarios to completion even with violations (overrides default)
    #[arg(long, overrides_with = "fail_fast")]
    no_fail_fast: bool,
}

/// Execution mode: `--parallel` or `--serial` (clap enforces mutual exclusion).
#[derive(clap::Args)]
struct ExecutionMode {
    /// Max parallel subprocesses: a number or "all" (default: 32)
    #[arg(short = 'p', long, conflicts_with = "serial")]
    parallel: Option<String>,

    /// Run in-process sequentially, no subprocesses
    #[arg(long, conflicts_with = "parallel")]
    serial: bool,

    /// Internal: marks this process as a stress-copy subprocess.
    /// Skips stress expansion to prevent infinite recursion.
    #[arg(long, hide = true)]
    stress_copy: bool,
}

/// Resolves the effective `fail_fast` value from CLI flags and run mode.
///
/// Resolution order:
/// 1. `--fail-fast` explicitly present → `true`
/// 2. `--no-fail-fast` explicitly present → `false`
/// 3. Neither present → default to `all` (on for `--all`, off for `-s`)
#[must_use]
const fn resolve_fail_fast(fail_fast: bool, no_fail_fast: bool, all: bool) -> bool {
    if fail_fast {
        true
    } else if no_fail_fast {
        false
    } else {
        all
    }
}

fn parse_loop_count(s: &str) -> Result<usize, String> {
    let n: usize = s
        .parse()
        .map_err(|_| format!("invalid loop count: \"{s}\""))?;
    if n == 0 {
        return Err("--loop must be a positive number".to_owned());
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_loop_count_accepts_positive_number() {
        assert_eq!(parse_loop_count("5"), Ok(5));
    }

    #[test]
    fn parse_loop_count_rejects_zero() {
        let result = parse_loop_count("0");
        assert!(result.is_err(), "expected error for 0, got: {result:?}");
    }

    #[test]
    fn parse_loop_count_rejects_non_numeric() {
        let result = parse_loop_count("abc");
        assert!(result.is_err(), "expected error for 'abc', got: {result:?}");
    }

    #[test]
    fn stress_copy_flag_parses() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo", "--stress-copy"]);
        assert!(args.execution.stress_copy, "stress_copy must be true");
        assert_eq!(args.scenario.as_deref(), Some("foo"));
    }

    #[test]
    fn stress_copy_flag_defaults_to_false() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo"]);
        assert!(
            !args.execution.stress_copy,
            "stress_copy must default to false"
        );
    }

    // -----------------------------------------------------------------
    // --fail-fast / --no-fail-fast flags: parsing and resolution
    // -----------------------------------------------------------------

    // Behavior 1: --fail-fast flag sets fail_fast field to true
    #[test]
    fn fail_fast_flag_sets_fail_fast_field_to_true() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo", "--fail-fast"]);
        assert!(
            args.fail_fast_mode.fail_fast,
            "fail_fast must be true when --fail-fast is provided"
        );
        assert!(
            !args.fail_fast_mode.no_fail_fast,
            "no_fail_fast must be false when only --fail-fast is provided"
        );
        assert_eq!(args.scenario.as_deref(), Some("foo"));
    }

    // Behavior 1 edge: --fail-fast combined with --all
    #[test]
    fn fail_fast_flag_with_all_sets_both_fields() {
        let args = Args::parse_from(["breaker_scenario_runner", "--all", "--fail-fast"]);
        assert!(args.fail_fast_mode.fail_fast, "fail_fast must be true");
        assert!(args.all, "all must be true");
    }

    // Behavior 2: --no-fail-fast flag sets no_fail_fast field to true
    #[test]
    fn no_fail_fast_flag_sets_no_fail_fast_field_to_true() {
        let args = Args::parse_from(["breaker_scenario_runner", "--all", "--no-fail-fast"]);
        assert!(
            args.fail_fast_mode.no_fail_fast,
            "no_fail_fast must be true when --no-fail-fast is provided"
        );
        assert!(
            !args.fail_fast_mode.fail_fast,
            "fail_fast must be false when only --no-fail-fast is provided"
        );
        assert!(args.all, "all must be true");
    }

    // Behavior 2 edge: --no-fail-fast combined with -s
    #[test]
    fn no_fail_fast_flag_with_scenario_sets_no_fail_fast_field() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo", "--no-fail-fast"]);
        assert!(
            args.fail_fast_mode.no_fail_fast,
            "no_fail_fast must be true"
        );
        assert_eq!(args.scenario.as_deref(), Some("foo"));
    }

    // Behavior 3: neither flag + --all resolves to true
    #[test]
    fn neither_flag_with_all_resolves_to_true() {
        let args = Args::parse_from(["breaker_scenario_runner", "--all"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            resolved,
            "resolved fail_fast must be true when neither flag given with --all"
        );
    }

    // Behavior 3 edge: --all with --serial and no fail-fast flags
    #[test]
    fn neither_flag_with_all_and_serial_resolves_to_true() {
        let args = Args::parse_from(["breaker_scenario_runner", "--all", "--serial"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            resolved,
            "resolved fail_fast must be true when --all --serial given without fail-fast flags"
        );
    }

    // Behavior 4: neither flag + -s resolves to false
    #[test]
    fn neither_flag_with_scenario_resolves_to_false() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            !resolved,
            "resolved fail_fast must be false when neither flag given with -s"
        );
    }

    // Behavior 4 edge: -s with --verbose and no fail-fast flags
    #[test]
    fn neither_flag_with_scenario_and_verbose_resolves_to_false() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo", "-v"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            !resolved,
            "resolved fail_fast must be false when -s -v given without fail-fast flags"
        );
    }

    // Behavior 5: --fail-fast with -s resolves to true (explicit override)
    #[test]
    fn explicit_fail_fast_with_scenario_resolves_to_true() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo", "--fail-fast"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            resolved,
            "resolved fail_fast must be true when --fail-fast explicitly given with -s"
        );
    }

    // Behavior 5 edge: reversed order
    #[test]
    fn explicit_fail_fast_before_scenario_resolves_to_true() {
        let args = Args::parse_from(["breaker_scenario_runner", "--fail-fast", "-s", "foo"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            resolved,
            "resolved fail_fast must be true regardless of flag order"
        );
    }

    // Behavior 6: --no-fail-fast with --all resolves to false (explicit override)
    #[test]
    fn explicit_no_fail_fast_with_all_resolves_to_false() {
        let args = Args::parse_from(["breaker_scenario_runner", "--all", "--no-fail-fast"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            !resolved,
            "resolved fail_fast must be false when --no-fail-fast explicitly given with --all"
        );
    }

    // Behavior 6 edge: reversed order
    #[test]
    fn explicit_no_fail_fast_before_all_resolves_to_false() {
        let args = Args::parse_from(["breaker_scenario_runner", "--no-fail-fast", "--all"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            !resolved,
            "resolved fail_fast must be false regardless of flag order"
        );
    }

    // Behavior 7: --fail-fast=true syntax is rejected by clap
    #[test]
    fn fail_fast_equals_value_syntax_is_rejected() {
        let result =
            Args::try_parse_from(["breaker_scenario_runner", "-s", "foo", "--fail-fast=true"]);
        assert!(
            result.is_err(),
            "clap should reject --fail-fast=true for boolean flags"
        );
    }

    // Behavior 7 edge: --fail-fast=false is also rejected
    #[test]
    fn fail_fast_equals_false_syntax_is_rejected() {
        let result =
            Args::try_parse_from(["breaker_scenario_runner", "-s", "foo", "--fail-fast=false"]);
        assert!(
            result.is_err(),
            "clap should reject --fail-fast=false for boolean flags"
        );
    }

    // Behavior 7 edge: --no-fail-fast=true is also rejected
    #[test]
    fn no_fail_fast_equals_true_syntax_is_rejected() {
        let result = Args::try_parse_from([
            "breaker_scenario_runner",
            "-s",
            "foo",
            "--no-fail-fast=true",
        ]);
        assert!(
            result.is_err(),
            "clap should reject --no-fail-fast=true for boolean flags"
        );
    }

    // Behavior 8: both flags given -- last one wins via clap overrides_with
    #[test]
    fn both_flags_given_last_wins_no_fail_fast_last() {
        let args = Args::parse_from([
            "breaker_scenario_runner",
            "--all",
            "--fail-fast",
            "--no-fail-fast",
        ]);
        assert!(
            !args.fail_fast_mode.fail_fast,
            "fail_fast must be false when --no-fail-fast is last (overrides_with)"
        );
        assert!(
            args.fail_fast_mode.no_fail_fast,
            "no_fail_fast must be true when --no-fail-fast is last"
        );
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            !resolved,
            "resolved fail_fast must be false when --no-fail-fast wins"
        );
    }

    // Behavior 8 edge: reversed order -- --fail-fast last
    #[test]
    fn both_flags_given_last_wins_fail_fast_last() {
        let args = Args::parse_from([
            "breaker_scenario_runner",
            "--all",
            "--no-fail-fast",
            "--fail-fast",
        ]);
        assert!(
            args.fail_fast_mode.fail_fast,
            "fail_fast must be true when --fail-fast is last (overrides_with)"
        );
        assert!(
            !args.fail_fast_mode.no_fail_fast,
            "no_fail_fast must be false when --fail-fast is last"
        );
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            resolved,
            "resolved fail_fast must be true when --fail-fast wins"
        );
    }

    // Behavior 9: --fail-fast combines with other flags without conflict
    #[test]
    fn fail_fast_combines_with_all_verbose_serial() {
        let args = Args::parse_from([
            "breaker_scenario_runner",
            "--all",
            "--fail-fast",
            "-v",
            "--serial",
        ]);
        assert!(args.fail_fast_mode.fail_fast, "fail_fast must be true");
        assert!(args.all, "all must be true");
        assert!(args.verbose, "verbose must be true");
        assert!(args.execution.serial, "serial must be true");
    }

    // Behavior 9 edge: --fail-fast with --stress-copy
    #[test]
    fn fail_fast_combines_with_stress_copy() {
        let args = Args::parse_from([
            "breaker_scenario_runner",
            "-s",
            "foo",
            "--fail-fast",
            "--stress-copy",
        ]);
        assert!(args.fail_fast_mode.fail_fast, "fail_fast must be true");
        assert!(args.execution.stress_copy, "stress_copy must be true");
    }
}
