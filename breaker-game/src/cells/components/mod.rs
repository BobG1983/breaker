//! Cells domain components.

mod types;

#[cfg(test)]
mod tests;

pub use types::*;

// Behavior components re-exported for cross-module access
pub use crate::cells::behaviors::locked::components::{LockCell, Locked, Locks, Unlocked};
pub use crate::cells::behaviors::regen::components::{NoRegen, Regen, RegenCell, RegenRate};
