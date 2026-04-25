//! Evaluate conditions — per-frame condition polling for During nodes.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{DuringActive, evaluate_conditions};
