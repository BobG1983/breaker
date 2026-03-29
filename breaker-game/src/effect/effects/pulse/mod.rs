pub(crate) use effect::*;

// Re-export marker type for scenario-runner invariant checking.
pub use effect::PulseRing;

mod effect;

#[cfg(test)]
mod tests;
