//! Bolt domain query type aliases.
//!
//! Query types with 4+ components live here rather than inline in system files.

use bevy::{ecs::query::Has, prelude::*};

use crate::{
    bolt::components::{
        BoltBaseSpeed, BoltRadius, BoltRespawnAngleSpread, BoltRespawnOffsetY, BoltVelocity,
        ExtraBolt,
    },
    chips::components::{Piercing, PiercingRemaining},
    interpolate::components::PhysicsTranslation,
    shared::EntityScale,
};

/// Bolt entity data needed by the reset system at node start.
pub(crate) type ResetBoltQuery = (
    Entity,
    &'static mut Transform,
    &'static mut BoltVelocity,
    Option<&'static mut PiercingRemaining>,
    Option<&'static Piercing>,
    Option<&'static mut PhysicsTranslation>,
);

/// Bolt entity data needed by the bolt-lost detection system.
pub(crate) type LostQuery = (
    Entity,
    &'static Transform,
    &'static BoltVelocity,
    &'static BoltBaseSpeed,
    &'static BoltRadius,
    &'static BoltRespawnOffsetY,
    &'static BoltRespawnAngleSpread,
    Has<ExtraBolt>,
    Option<&'static EntityScale>,
);
