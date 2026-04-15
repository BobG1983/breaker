//! When node evaluator — trigger-gated effect firing.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::evaluate_when;
