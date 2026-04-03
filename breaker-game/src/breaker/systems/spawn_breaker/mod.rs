//! System to spawn the breaker entity.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::spawn_or_reuse_breaker;
