//! Scenario runner — automated gameplay testing tool.
//!
//! Runs headless by default (no GPU required). Pass `--visual` to open a window
//! with full graphics at 10x speed for debugging a single scenario.
//!
//! Usage:
//!   `cargo scenario -- -s aegis_chaos`
//!   `cargo scenario -- --all`
//!   `cargo scenario -- --visual -s aegis_chaos`

use std::process;

use argh::FromArgs;

fn main() {
    let args: Args = argh::from_env();

    if args.visual && args.all {
        eprintln!("--visual cannot be combined with --all (the event loop can only run once)");
        eprintln!("Use --visual -s <scenario_name> to debug a single scenario.");
        process::exit(1);
    }

    if args.visual && args.scenario.is_none() {
        eprintln!("--visual requires -s <scenario_name>");
        process::exit(1);
    }

    let headless = !args.visual;
    let exit_code = breaker_scenario_runner::runner::run_with_args(
        args.scenario.as_deref(),
        args.all,
        headless,
    );
    process::exit(exit_code);
}

/// Automated gameplay scenario runner.
#[derive(FromArgs)]
pub struct Args {
    /// scenario name to run (stem of a `.scenario.ron` file in `scenarios/`)
    #[argh(option, short = 's')]
    pub scenario: Option<String>,

    /// run all scenarios in the `scenarios/` directory tree
    #[argh(switch)]
    pub all: bool,

    /// run with a window at normal speed for visual debugging (single scenario only)
    #[argh(switch)]
    pub visual: bool,
}
