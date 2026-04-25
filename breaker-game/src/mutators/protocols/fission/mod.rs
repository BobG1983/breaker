//! Fission protocol — kill-counter bolt-split.
//!
//! Design doc: `docs/design/protocols/fission.md`.
//!
//! Owns `FissionConfig` (per-run tuning), `FissionCounter` (persistent-across-nodes
//! kill tracker), `FISSION_DIVERGENCE_ANGLE_RAD`, `activate`, `wire`, and the
//! `fission_on_cell_destroyed` + `fission_cleanup_run` systems.

pub mod system;

#[cfg(test)]
mod tests;

pub use system::FissionCounter;
pub(crate) use system::{activate, wire};
