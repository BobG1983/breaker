pub(crate) use effect::{CircuitBreakerConfig, fire, register, reverse};

mod effect;

#[cfg(test)]
mod tests;
