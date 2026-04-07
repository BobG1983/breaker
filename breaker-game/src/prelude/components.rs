//! Re-exports of cross-domain components.

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
    shared::components::NodeScalingFactor,
    walls::components::Wall,
};
