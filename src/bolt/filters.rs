//! Bolt domain query filters.

use bevy::prelude::*;

use crate::bolt::components::{Bolt, BoltServing};

/// Query filter for active (non-serving) bolts.
///
/// Shared across bolt and physics systems that should skip serving bolts.
pub type ActiveBoltFilter = (With<Bolt>, Without<BoltServing>);

/// Query filter for serving bolts (hovering, awaiting launch).
///
/// Used by `hover_bolt` and `launch_bolt`.
pub type ServingBoltFilter = (With<Bolt>, With<BoltServing>);
