//! Cells-domain `apply_damage_to_cells` — replaces the generic
//! `apply_damage::<Cell>` in the cells domain. Owns the Cell damage-application
//! path including the Diffusion hazard's BFS redistribution branch.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::apply_damage_to_cells;
