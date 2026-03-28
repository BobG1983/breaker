//! Bolt-wall collision detection.
//!
//! Separated from bolt-cell collision to support distinct Impact/Impacted
//! trigger mappings per the collision architecture. Currently a stub:
//! wall collision physics remain in [`bolt_cell_collision`] which handles
//! the unified CCD sweep. This system exists for future migration and
//! message routing.

use bevy::prelude::*;

/// Stub system for bolt-wall collision detection.
///
/// Wall collision physics are currently handled by the combined CCD sweep
/// in [`bolt_cell_collision`]. This system is registered for forward
/// compatibility with the per-collision-type architecture.
pub(crate) fn bolt_wall_collision() {
    // Wall collision logic is currently handled by bolt_cell_collision's
    // unified CCD sweep. This stub exists so the system is registered
    // and can be referenced in system sets and ordering constraints.
}
