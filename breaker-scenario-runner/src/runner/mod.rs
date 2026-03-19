//! App construction and multi-scenario execution.
//!
//! Builds either a visual or headless [`App`] for each scenario and runs it to
//! completion, then prints a structured summary and returns the exit code.

mod app;
mod discovery;
mod execution;
mod output;
#[cfg(test)]
mod tests;

pub use app::{drain_remaining_logs, guarded_update, is_timed_out};
pub use execution::{
    Parallelism, StressFailure, StressResult, build_run_list, parse_parallelism,
    partition_stress_scenarios, print_stress_result, run_all_parallel, run_all_serial,
    run_single_scenario, run_stress_scenario, run_with_args, scenarios_dir,
};
