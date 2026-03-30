//! Breaker initialization systems — config overrides and component stamping.

mod system;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use system::apply_breaker_config_overrides;
#[cfg(any(test, feature = "dev"))]
pub(crate) use system::apply_stat_overrides;
pub(crate) use system::init_breaker;
