//! Bolt domain query filters.

use bevy::prelude::*;

use crate::bolt::components::{Bolt, BoltServing};

/// Query filter for active (non-serving) bolts.
///
/// Shared across bolt and physics systems that should skip serving bolts.
pub type ActiveBoltFilter = (With<Bolt>, Without<BoltServing>);
