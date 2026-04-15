//! Once node evaluator — single-fire effect application.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::evaluate_once;
