//! Timed transition animations between nodes.
//!
//! Provides visual transitions (flash, sweep) when entering/leaving a node.
//! Spawns a full-screen overlay entity with a [`TransitionTimer`] that drives
//! the animation, then transitions to the next [`GameState`] on completion.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{
    TransitionConfig, animate_transition, cleanup_transition, spawn_transition_in,
    spawn_transition_out,
};
