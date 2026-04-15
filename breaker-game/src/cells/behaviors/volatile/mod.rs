//! Volatile cell behavior — detonates on death, dealing explosion damage to
//! nearby cells within a configured radius.

pub(crate) mod components;
pub(crate) mod stamp;

#[cfg(test)]
mod tests;
