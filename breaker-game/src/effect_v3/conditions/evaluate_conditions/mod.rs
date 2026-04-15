//! Evaluate conditions — per-frame condition polling for During nodes.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::{DuringActive, evaluate_condition, evaluate_conditions};
