//! Chips domain query type aliases — clippy `type_complexity` lint.

use bevy::prelude::*;

use crate::chips::components::*;

/// Bolt entity data for applying Amp chip effects.
pub(crate) type EffectQueryBolt = (
    Entity,
    Option<&'static mut Piercing>,
    Option<&'static mut DamageBoost>,
    Option<&'static mut BoltSpeedBoost>,
    Option<&'static mut ChainHit>,
    Option<&'static mut BoltSizeBoost>,
);

/// Breaker entity data for applying Augment chip effects.
pub(crate) type EffectQueryBreaker = (
    Entity,
    Option<&'static mut WidthBoost>,
    Option<&'static mut BreakerSpeedBoost>,
    Option<&'static mut BumpForceBoost>,
    Option<&'static mut TiltControlBoost>,
);
