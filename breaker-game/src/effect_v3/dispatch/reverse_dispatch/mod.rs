//! Reverse dispatch — fire, reverse, and reverse-all-by-source.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::{fire_reversible_dispatch, reverse_all_by_source_dispatch, reverse_dispatch};
