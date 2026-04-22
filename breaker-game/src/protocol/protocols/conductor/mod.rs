//! Conductor protocol — Perfect-bump-driven primary-bolt swap.
//!
//! Design doc: `docs/design/protocols/conductor.md`.
//!
//! Owns the `ConductorConfig` resource (per-run tuning), the single runtime
//! system `conductor_swap_on_perfect_bump`, and the `activate` / `register`
//! dispatch entry points. No per-node state, no sentinel, no cleanup system.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, register};
