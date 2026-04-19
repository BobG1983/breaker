//! Cells-domain adjacency constant.
//!
//! Owns the world-space radius (squared) within which two cells count as
//! adjacent for hazard BFS propagation (Diffusion / Sympathy / etc.) and for
//! Cascade-style neighbour heals. Tuned to ~1.25× a 50-unit cell width so
//! horizontally + vertically adjacent grid neighbours register without pulling
//! in diagonals across a wide gap.

/// World-space adjacency distance (squared). Cell centres within this squared
/// distance of each other count as neighbours for cells-domain adjacency
/// queries. Value: `70.0 * 70.0 = 4900.0`.
pub(crate) const ADJACENCY_RADIUS_SQ: f32 = 70.0 * 70.0;
