//! Breaker definition — RON-deserialized breaker content type.

pub(crate) mod types;

#[cfg(test)]
mod tests;

pub use types::{BreakerDefinition, DEFAULT_COLOR_RGB};
