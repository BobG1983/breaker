//! Scenario verdict consolidation.
//!
//! [`ScenarioVerdict`] collects pass/fail state across violation checks, log
//! capture, and health warnings. Defaults to `Fail` so any gap in the
//! evaluation pipeline produces a safe failure.

pub(crate) mod evaluation;

#[cfg(test)]
mod tests;

pub use evaluation::{ScenarioVerdict, VerdictStatus};
