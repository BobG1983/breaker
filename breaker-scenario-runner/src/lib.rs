//! Scenario runner library — automated gameplay testing.
//!
//! Provides the [`ScenarioLifecycle`] plugin and the [`runner`] entry point.

pub mod input;
pub mod invariants;
pub mod lifecycle;
pub mod log_capture;
pub mod runner;
pub mod types;

pub use lifecycle::{ScenarioConfig, ScenarioLifecycle};
