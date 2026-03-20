//! Physics domain query type aliases — complex query tuples live here rather than inline in system files.

use bevy::prelude::*;

use crate::{
    bolt::components::{BoltBaseSpeed, BoltRadius, BoltVelocity},
    breaker::components::{
        BreakerHeight, BreakerTilt, BreakerWidth, MaxReflectionAngle, MinAngleFromHorizontal,
    },
    cells::components::{CellHealth, CellHeight, CellWidth},
    chips::components::{DamageBoost, Piercing, PiercingRemaining, TiltControlBoost, WidthBoost},
    shared::EntityScale,
};

/// Bolt entity data needed by physics collision systems.
pub(crate) type CollisionQueryBolt = (
    Entity,
    &'static mut Transform,
    &'static mut BoltVelocity,
    &'static BoltBaseSpeed,
    &'static BoltRadius,
    Option<&'static mut PiercingRemaining>,
    Option<&'static Piercing>,
    Option<&'static DamageBoost>,
    Option<&'static EntityScale>,
);

/// Breaker entity data needed by bolt-breaker collision.
pub(crate) type CollisionQueryBreaker = (
    &'static Transform,
    &'static BreakerTilt,
    &'static BreakerWidth,
    &'static BreakerHeight,
    &'static MaxReflectionAngle,
    &'static MinAngleFromHorizontal,
    Option<&'static TiltControlBoost>,
    Option<&'static WidthBoost>,
    Option<&'static EntityScale>,
);

/// Cell entity data needed by bolt-cell collision.
pub(crate) type CollisionQueryCell = (
    Entity,
    &'static Transform,
    &'static CellWidth,
    &'static CellHeight,
    Option<&'static CellHealth>,
);
