//! Cells systems — one file per system function.

mod cell_wall_collision;
pub(crate) mod update_cell_damage_visuals;

pub(crate) use cell_wall_collision::cell_wall_collision;
pub(crate) use update_cell_damage_visuals::update_cell_damage_visuals;
