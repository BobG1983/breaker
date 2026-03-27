/// Invariant checker for monotonic run stats progression.
pub(crate) mod checker;
#[cfg(test)]
mod tests;

pub use checker::*;
