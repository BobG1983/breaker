//! Cells domain components.

mod types;

#[cfg(test)]
mod tests;

pub use types::*;

// Behavior components re-exported for cross-module access
pub use crate::cells::behaviors::guarded::components::{
    GuardedCell, GuardianCell, GuardianGridStep, GuardianSlideSpeed, GuardianSlot, SlideTarget,
};
pub use crate::cells::behaviors::{
    locked::components::{LockCell, Locked, Locks, Unlocked},
    regen::components::{NoRegen, Regen, RegenCell, RegenRate},
};
