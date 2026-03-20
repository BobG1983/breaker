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

use std::process;

use breaker_scenario_runner::runner::{
    Parallelism, build_run_list, parse_parallelism, partition_stress_scenarios,
    print_stress_result, print_summary, run_all_parallel, run_all_serial, run_single_scenario,
    run_stress_scenario, run_with_args,
};
use clap::Parser;

fn main() {
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
        let exit_code = run_with_args(args.scenario.as_deref(), headless, args.verbose);
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

        let (normal, stress) = partition_stress_scenarios(&runs);
        if let Some((name, _path, config)) = stress.into_iter().next() {
            let result = run_stress_scenario(&name, &config, args.visual, args.verbose);
            print_stress_result(&result);
            process::exit(i32::from(!result.passed()));
        }

        // No stress config — run in-process with the already-resolved path.
        if !normal.is_empty() {
            let exit_code = run_single_scenario(&normal[0].1, headless, args.verbose);
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

    let mut worst_exit = 0;
    for iteration in 1..=loop_count {
        if loop_count > 1 {
            println!("\n=== Loop {iteration}/{loop_count} ===");
        }

        let mut all_results: Vec<(String, bool)> = Vec::new();

        // Run normal scenarios.
        if !normal_runs.is_empty() {
            let results = if args.execution.serial {
                run_all_serial(&normal_runs, headless, args.verbose)
            } else {
                let batch_size = parallelism.resolve(normal_runs.len());
                run_all_parallel(&normal_runs, args.visual, args.verbose, batch_size)
            };
            all_results.extend(results);
        }

        // Run stress scenarios.
        for (name, _path, config) in &stress_runs {
            let result = run_stress_scenario(name, config, args.visual, args.verbose);
            print_stress_result(&result);
            all_results.push((name.clone(), result.passed()));
        }

        // Print combined summary for this iteration.
        let exit_code = print_summary(&all_results);
        if exit_code > worst_exit {
            worst_exit = exit_code;
        }
    }

    process::exit(worst_exit);
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
