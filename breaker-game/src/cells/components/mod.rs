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
    armored::components::{ArmorDirection, ArmorFacing, ArmorValue, ArmoredCell},
    locked::components::{LockCell, Locked, Locks, Unlocked},
    magnetic::components::{MagneticCell, MagneticField},
    phantom::components::{PhantomCell, PhantomConfig, PhantomPhase, PhantomTimer},
    regen::components::{NoRegen, Regen, RegenCell, RegenRate},
    sequence::components::{SequenceActive, SequenceCell, SequenceGroup, SequencePosition},
    survival::components::{
        BoltImmune, BumpVulnerable, SurvivalPattern, SurvivalTimer, SurvivalTurret,
    },
    volatile::components::VolatileCell,
};
