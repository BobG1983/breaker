//! Chain lightning configuration.
pub(crate) mod config_impl;

#[cfg(test)]
mod tests;

pub use config_impl::ChainLightningConfig;
