//! Scenario runner library — automated gameplay testing.
//!
//! Provides the [`ScenarioLifecycle`] plugin and the CLI entry point [`run`].

pub mod input;
pub mod invariants;
pub mod lifecycle;
pub mod log_capture;
pub mod types;

pub use lifecycle::{ScenarioConfig, ScenarioLifecycle};

/// CLI entry point. Parses arguments and runs the requested scenario(s).
pub fn run() {
    // Stub — wired in Commit 9
    println!("Scenario runner — not yet wired. Use --help for usage.");
}
