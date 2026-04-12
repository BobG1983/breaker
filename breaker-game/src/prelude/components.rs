//! Re-exports of cross-domain components and spatial/physics types.

pub(crate) use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
pub(crate) use rantzsoft_spatial2d::components::{Position2D, PreviousScale, Scale2D, Velocity2D};
pub(crate) use rantzsoft_stateflow::CleanupOnExit;

pub(crate) use crate::{
    bolt::components::{Bolt, BoltServing},
    breaker::components::Breaker,
    cells::{
        behaviors::{
            locked::components::{LockCell, Locked, Locks, Unlocked},
            regen::components::{NoRegen, Regen, RegenCell, RegenRate},
        },
        components::Cell,
    },
    effect_v3::{
        effects::{
            anchor::{AnchorActive, AnchorPlanted},
            flash_step::FlashStepActive,
        },
        storage::{BoundEffects, StagedEffects},
    },
    shared::{birthing::Birthing, components::NodeScalingFactor},
    walls::components::Wall,
};
