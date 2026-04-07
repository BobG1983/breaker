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
    // Handle --clean before full CLI parsing to avoid adding a 4th bool to Args.
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

    let args = Args::parse();

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
        let exit_code = run_with_args(args.scenario.as_deref(), headless, args.verbose, None);
        process::exit(exit_code);
    }

    // Fast path: single scenario, no loop → check for stress config first.
    if args.scenario.is_some() && !args.all && loop_count == 1 && !args.execution.serial {
        // Build the run list to resolve the path, then check for stress config.
        let runs = build_run_list(args.scenario.as_deref(), false);
        if runs.is_empty() {
            eprintln!("No scenarios found. Use -s <name> or --all.");
            process::exit(1);
        }

        // Create RunLog for single-scenario fast path.
        let run_log = create_run_log_if_needed(true);

        let (normal, stress) = partition_stress_scenarios(&runs);
        if let Some((name, _path, config)) = stress.into_iter().next() {
            let result =
                run_stress_scenario(&name, &config, args.visual, args.verbose, run_log.as_ref());
            print_stress_result(&result, run_log.as_ref());
            print_log_path_and_shutdown(run_log);
            process::exit(i32::from(!result.passed()));
        }

        // No stress config — run in-process with the already-resolved path.
        if !normal.is_empty() {
            let exit_code =
                run_single_scenario(&normal[0].1, headless, args.verbose, run_log.as_ref());
            print_log_path_and_shutdown(run_log);
            process::exit(exit_code);
        }
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

    let run_log = create_run_log_if_needed(args.all || args.scenario.is_some());

    let worst_exit = run_loop(
        &normal_runs,
        &stress_runs,
        loop_count,
        headless,
        &args,
        parallelism,
        run_log.as_ref(),
    );

    // Print coverage report when running --all.
    if args.all {
        print_coverage_for_runs(&runs);
    }

    print_log_path_and_shutdown(run_log);

    process::exit(worst_exit);
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
    headless: bool,
    args: &Args,
    parallelism: Parallelism,
    run_log: Option<&RunLog>,
) -> i32 {
    let mut worst_exit = 0;
    for iteration in 1..=loop_count {
        if loop_count > 1 {
            println!("\n=== Loop {iteration}/{loop_count} ===");
        }

        let mut all_results: Vec<(String, bool)> = Vec::new();

        if !normal_runs.is_empty() {
            let results = if args.execution.serial {
                run_all_serial(normal_runs, headless, args.verbose, run_log)
            } else {
                let batch_size = parallelism.resolve(normal_runs.len());
                run_all_parallel(normal_runs, args.visual, args.verbose, batch_size, run_log)
            };
            all_results.extend(results);
        }

        for (name, _path, config) in stress_runs {
            let result = run_stress_scenario(name, config, args.visual, args.verbose, run_log);
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
    let _ = print_coverage_report(&report);
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
    execution: ExecutionMode,

    /// Repeat the entire run N times
    #[arg(short = 'l', long = "loop", value_parser = parse_loop_count)]
    loops: Option<usize>,
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
}
