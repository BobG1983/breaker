//! Shockwave systems — damage application, despawn, tick.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::apply_shockwave_damage;
pub use system::{despawn_finished_shockwave, tick_shockwave};
