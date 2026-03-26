//! Convenience re-exports for consumers of `rantzsoft_physics2d`.
//!
//! ```rust,ignore
//! use rantzsoft_physics2d::prelude::*;
//! ```

pub use crate::{
    aabb::Aabb2D,
    ccd::{CCD_EPSILON, MAX_BOUNCES, RayHit, SweepHit, ray_vs_aabb},
    collision_layers::CollisionLayers,
    constraint::DistanceConstraint,
    plugin::{PhysicsSystems, RantzPhysics2dPlugin},
    quadtree::Quadtree,
    resources::CollisionQuadtree,
};
