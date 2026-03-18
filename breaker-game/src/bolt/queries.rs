//! Bolt domain query type aliases.
//!
//! Query types with 5+ components live here rather than inline in system files.

use bevy::{ecs::query::Has, prelude::*};

use crate::bolt::components::{
    BoltBaseSpeed, BoltRadius, BoltRespawnAngleSpread, BoltRespawnOffsetY, BoltVelocity, ExtraBolt,
};

/// Bolt entity data needed by the bolt-lost detection system.
pub type BoltLostQuery = (
    Entity,
    &'static Transform,
    &'static BoltVelocity,
    &'static BoltBaseSpeed,
    &'static BoltRadius,
    &'static BoltRespawnOffsetY,
    &'static BoltRespawnAngleSpread,
    Has<ExtraBolt>,
);
