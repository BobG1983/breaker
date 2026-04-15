//! Tether beam systems — tick beam damage, cleanup dead beams.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{cleanup_tether_beams, tick_tether_beam};
