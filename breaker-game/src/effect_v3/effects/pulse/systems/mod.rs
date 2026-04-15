//! Pulse systems — emitter tick, ring expansion, damage, despawn.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{
    apply_pulse_damage, despawn_finished_pulse_ring, tick_pulse, tick_pulse_ring,
};
