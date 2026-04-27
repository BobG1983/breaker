//! Effect timer tick system.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::tick_effect_timers;
