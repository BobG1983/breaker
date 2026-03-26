//! Breaker initialization systems — config overrides and component stamping.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{apply_breaker_config_overrides, apply_stat_overrides, init_breaker};
