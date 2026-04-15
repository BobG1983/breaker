//! Circuit breaker systems — tick circuit breaker effect.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::tick_circuit_breaker;
