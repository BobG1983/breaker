//! App construction and multi-scenario execution.
//!
//! Builds either a visual or headless [`App`] for each scenario and runs it to
//! completion, then prints a structured summary and returns the exit code.

mod app;
mod discovery;
mod execution;
mod output;
pub mod output_dir;
#[cfg(test)]
mod tests;
pub mod tiling;

pub use app::{drain_remaining_logs, guarded_update, is_timed_out, sync_ui_scale};
pub use discovery::load_scenario;
pub use execution::*;
