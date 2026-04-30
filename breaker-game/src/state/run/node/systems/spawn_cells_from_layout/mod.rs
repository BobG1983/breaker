//! System to spawn cells from the active node layout.

pub(crate) mod geometry;
mod lock_resolution;
pub(crate) mod spawning;

pub(crate) use geometry::*;
pub(crate) use spawning::*;

#[cfg(test)]
mod tests;
