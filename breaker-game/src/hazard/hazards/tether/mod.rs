//! Tether hazard — pairs of cells share damage.
//!
//! Design doc: `docs/todos/detail/mod-system-design/hazards/tether.md`.
//!
//! Owns the `TetherConfig` resource, `TetherLink` component, the
//! `TetherRedirectBuffer` resource that the cells-domain
//! `apply_damage_to_cells` pushes redirect messages onto, and the
//! `establish_tether_links` / `cleanup_broken_tether_links` /
//! `emit_tether_redirects` systems registered via [`register`].

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{
    TETHER_SENTINEL, TetherConfig, TetherLink, TetherRedirectBuffer, activate, register,
};
