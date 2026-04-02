//! System to spawn the breaker entity.

mod system;

#[cfg(test)]
mod tests;

pub use system::{reset_breaker, spawn_or_reuse_breaker};
