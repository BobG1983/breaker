//! Bolt registry — maps bolt names to definitions.

pub(crate) mod data;

#[cfg(test)]
mod tests;

pub use data::BoltRegistry;
