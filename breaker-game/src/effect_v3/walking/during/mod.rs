//! During node evaluator — condition-scoped effect activation.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::evaluate_during;
