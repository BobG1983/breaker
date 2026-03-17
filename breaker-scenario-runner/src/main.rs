//! Scenario runner — automated gameplay testing tool.
//!
//! Usage:
//!   `cargo dscenario -- -s aegis_chaos`
//!   `cargo dscenario -- --headless -s aegis_chaos`
//!   `cargo dscenario -- --all --headless`

use std::process;

use argh::FromArgs;

fn main() {
    let args: Args = argh::from_env();
    let exit_code = breaker_scenario_runner::runner::run_with_args(
        args.scenario.as_deref(),
        args.all,
        args.headless,
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

    /// run without a window (headless mode — no GPU required)
    #[argh(switch)]
    pub headless: bool,
}
