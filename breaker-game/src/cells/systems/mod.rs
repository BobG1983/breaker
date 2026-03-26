//! Cells systems — one file per system function.

pub(super) mod check_lock_release;
pub(crate) mod cleanup_destroyed_cells;
mod handle_cell_hit;
pub(super) mod rotate_shield_cells;
pub(super) mod sync_orbit_cell_positions;
pub(super) mod tick_cell_regen;

pub(crate) use cleanup_destroyed_cells::cleanup_destroyed_cells;
pub(crate) use handle_cell_hit::handle_cell_hit;
