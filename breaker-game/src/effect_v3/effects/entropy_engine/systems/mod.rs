//! Entropy engine systems — tick entropy counter, reset on threshold.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::{reset_entropy_counter, tick_entropy_engine};
