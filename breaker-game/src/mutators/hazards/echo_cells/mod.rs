//! `EchoCells` hazard — every destroyed cell leaves a ghost after
//! `delay_secs` seconds. Ghosts are low-HP `Cell` entities marked with
//! [`GhostCell`]; they carry no original cell rules. HP scales
//! geometrically per stack (`base_hp * multiplier^(stacks - 1)`).
//!
//! Ghost deaths do NOT spawn new ghosts — the `GhostCell` marker
//! filters them out in the tracker.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, wire};
