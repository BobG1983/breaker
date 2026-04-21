//! Diffusion hazard — stateless damage-sharing redistribution.
//!
//! Per the design doc at `docs/todos/detail/mod-system-design/hazards/diffusion.md`,
//! redistribution lives INSIDE the cells-domain system `apply_damage_to_cells`.
//! The hazard domain owns only the per-run `DiffusionConfig` resource populated
//! by [`activate`] and a no-op [`register`] — there is no hazard-domain runtime
//! system for the BFS / HP math.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{DiffusionConfig, activate, register};
