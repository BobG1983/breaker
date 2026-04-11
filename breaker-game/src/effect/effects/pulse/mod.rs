// Re-export marker type for scenario-runner invariant checking.
pub use effect::PulseRing;
pub(crate) use effect::*;

mod effect;

#[cfg(test)]
mod tests;
