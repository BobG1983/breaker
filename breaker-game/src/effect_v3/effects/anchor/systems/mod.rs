//! Anchor systems — detect breaker movement, tick anchor.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::{detect_breaker_movement, tick_anchor};
