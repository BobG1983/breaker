//! Physics domain query type aliases — clippy `type_complexity` lint.

use bevy::prelude::*;

use crate::{
    bolt::components::{BoltBaseSpeed, BoltRadius, BoltVelocity},
    breaker::components::{
        BreakerHeight, BreakerTilt, BreakerWidth, MaxReflectionAngle, MinAngleFromHorizontal,
    },
    cells::components::{CellHeight, CellWidth},
};

/// Bolt entity data needed by physics collision systems.
pub(crate) type CollisionQueryBolt = (
    Entity,
    &'static mut Transform,
    &'static mut BoltVelocity,
    &'static BoltBaseSpeed,
    &'static BoltRadius,
);

/// Breaker entity data needed by bolt-breaker collision.
pub(crate) type CollisionQueryBreaker = (
    &'static Transform,
    &'static BreakerTilt,
    &'static BreakerWidth,
    &'static BreakerHeight,
    &'static MaxReflectionAngle,
    &'static MinAngleFromHorizontal,
);

/// Cell entity data needed by bolt-cell collision.
pub(crate) type CollisionQueryCell = (
    Entity,
    &'static Transform,
    &'static CellWidth,
    &'static CellHeight,
);
