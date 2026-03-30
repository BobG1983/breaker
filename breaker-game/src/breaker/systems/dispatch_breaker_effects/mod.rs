//! Dispatch breaker-defined effects to target entities.

pub(crate) use system::dispatch_breaker_effects;

mod system;

#[cfg(test)]
mod tests;
