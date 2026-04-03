//! Shared imports for `dispatch_initial_effects` tests.

pub(super) use bevy::prelude::*;

pub(super) use super::super::super::ext::*;
pub(super) use crate::{
    bolt::components::{Bolt, ExtraBolt, PrimaryBolt},
    breaker::{components::Breaker, definition::BreakerDefinition},
    cells::components::Cell,
    effect::{core::*, effects::damage_boost::ActiveDamageBoosts},
    walls::components::Wall,
};
