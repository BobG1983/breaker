//! Ramping damage chip effect — accumulates bonus damage per cell hit, resets on breaker bounce.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::*;
