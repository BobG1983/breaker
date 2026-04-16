//! Magnetic cell behavior — cells emit an inverse-square attraction field
//! that pulls bolts toward their center within a configurable radius.

pub(crate) mod components;
pub(crate) mod systems;

#[cfg(test)]
mod tests;
