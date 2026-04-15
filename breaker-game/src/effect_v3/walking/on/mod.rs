//! On node evaluator — participant-targeted effect dispatch.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::evaluate_on;
