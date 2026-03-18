//! Scenario runner — automated gameplay testing tool.
//!
//! Runs headless by default (no GPU required). Pass `--visual` to open a window
//! with full graphics at normal speed for debugging.
//!
//! Usage:
//!   `cargo dscenario -- -s aegis_chaos`
//!   `cargo dscenario -- --all`
//!   `cargo dscenario -- --visual -s aegis_chaos`

use std::process;

use argh::FromArgs;

fn main() {
    let args: Args = argh::from_env();
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

    /// run with a window at normal speed (for visual debugging)
    #[argh(switch)]
    pub visual: bool,
}
