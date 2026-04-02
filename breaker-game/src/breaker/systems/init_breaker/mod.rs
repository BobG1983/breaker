//! Breaker initialization systems — config overrides and component stamping.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::init_breaker;
