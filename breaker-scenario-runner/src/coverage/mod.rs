//! Coverage parity checking for invariant self-tests and layout usage.
//!
//! Pure analysis module — no Bevy, no ECS. Checks that every
//! [`InvariantKind`](crate::types::InvariantKind) variant has at least one
//! self-test scenario and that every layout RON file is referenced by at
//! least one scenario.

/// Production systems and helpers for invariant and layout coverage checks.
pub mod system;

#[cfg(test)]
mod tests;

pub use system::{CoverageReport, check_coverage, format_coverage_report, print_coverage_report};
