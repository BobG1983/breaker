//! Cells systems — one file per system function.

pub(crate) mod apply_damage_to_cells;
mod cell_wall_collision;
pub(crate) mod update_cell_damage_visuals;

pub(crate) use apply_damage_to_cells::apply_damage_to_cells;
pub(crate) use cell_wall_collision::cell_wall_collision;
pub(crate) use update_cell_damage_visuals::update_cell_damage_visuals;
