//! Circuit breaker effect — accumulates bumps then fires effects.

pub mod components;
pub mod config;

pub use components::CircuitBreakerCounter;
pub use config::CircuitBreakerConfig;
