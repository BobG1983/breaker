//! System to spawn cells from the active node layout.

pub(crate) mod system;

pub(crate) use system::*;

#[cfg(test)]
mod tests;
