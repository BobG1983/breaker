//! App construction and multi-scenario execution.
//!
//! Builds either a visual or headless [`App`] for each scenario and runs it to
//! completion, then prints a structured summary and returns the exit code.

mod app;
mod discovery;
mod execution;
mod output;
pub mod output_dir;
pub mod run_log;
pub mod streaming;
#[cfg(test)]
mod tests;
pub mod tiling;

pub use discovery::load_scenario;
pub use execution::*;
