//! Bolt domain query type aliases.
//!
//! Query types with 4+ components live here rather than inline in system files.

use bevy::{ecs::query::Has, prelude::Entity};
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, Velocity2D};

use crate::{
    bolt::components::{
        BoltBaseSpeed, BoltRadius, BoltRespawnAngleSpread, BoltRespawnOffsetY, ExtraBolt,
        LastImpact, PiercingRemaining, SpawnedByEvolution,
    },
    effect::effects::{
        damage_boost::ActiveDamageBoosts, piercing::ActivePiercings, speed_boost::ActiveSpeedBoosts,
    },
    shared::EntityScale,
};

/// Bolt entity data needed by physics collision systems.
pub(crate) type CollisionQueryBolt = (
    Entity,
    &'static mut Position2D,
    &'static mut Velocity2D,
    &'static BoltBaseSpeed,
    &'static BoltRadius,
    Option<&'static mut PiercingRemaining>,
    Option<&'static ActivePiercings>,
    Option<&'static ActiveDamageBoosts>,
    Option<&'static EntityScale>,
    Option<&'static SpawnedByEvolution>,
    Option<&'static mut LastImpact>,
    Option<&'static ActiveSpeedBoosts>,
);

/// Bolt entity data needed by the reset system at node start.
pub(crate) type ResetBoltQuery = (
    Entity,
    &'static mut Position2D,
    &'static mut Velocity2D,
    Option<&'static mut PiercingRemaining>,
    Option<&'static ActivePiercings>,
    Option<&'static mut PreviousPosition>,
);

/// Bolt entity data needed by the bolt-lost detection system.
pub(crate) type LostQuery = (
    Entity,
    &'static Position2D,
    &'static Velocity2D,
    &'static BoltBaseSpeed,
    &'static BoltRadius,
    &'static BoltRespawnOffsetY,
    &'static BoltRespawnAngleSpread,
    Has<ExtraBolt>,
    Option<&'static EntityScale>,
);
