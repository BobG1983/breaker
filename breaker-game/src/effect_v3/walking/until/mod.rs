//! Until node evaluator — event-scoped effect application.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::{UntilApplied, evaluate_until};
