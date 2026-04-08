//! Re-exports of cross-domain components and spatial/physics types.

pub(crate) use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
pub(crate) use rantzsoft_spatial2d::components::{Position2D, PreviousScale, Scale2D, Velocity2D};
pub(crate) use rantzsoft_stateflow::CleanupOnExit;

pub(crate) use crate::{
    bolt::components::{Bolt, BoltServing},
    breaker::components::Breaker,
    cells::components::Cell,
    effect::{
        AnchorActive, AnchorPlanted, BoundEffects, EffectNode, RootEffect, StagedEffects,
        effects::{
            damage_boost::ActiveDamageBoosts, flash_step::FlashStepActive,
            size_boost::ActiveSizeBoosts, speed_boost::ActiveSpeedBoosts,
            vulnerable::ActiveVulnerability,
        },
    },
    shared::{birthing::Birthing, components::NodeScalingFactor},
    walls::components::Wall,
};
