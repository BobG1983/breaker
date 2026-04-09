//! System to spawn cells from the active node layout.

mod lock_resolution;
pub(crate) mod system;

pub(crate) use system::*;

#[cfg(test)]
mod tests;
