//! Physics domain query type aliases — clippy `type_complexity` lint.

use bevy::prelude::*;

use crate::bolt::components::{BoltBaseSpeed, BoltRadius, BoltVelocity};

/// Bolt entity data needed by physics collision systems.
pub type BoltPhysicsQuery = (
    Entity,
    &'static mut Transform,
    &'static mut BoltVelocity,
    &'static BoltBaseSpeed,
    &'static BoltRadius,
);
