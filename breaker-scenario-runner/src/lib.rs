//! Scenario runner library — automated gameplay testing.
//!
//! Provides the [`ScenarioLifecycle`] plugin and the [`runner`] entry point.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "test assertions use unwrap/expect/panic"
    )
)]

pub mod coverage;
pub mod input;
pub mod invariants;
pub mod lifecycle;
pub mod log_capture;
pub mod runner;
pub mod types;
pub mod verdict;

pub use lifecycle::{ScenarioConfig, ScenarioLifecycle};
