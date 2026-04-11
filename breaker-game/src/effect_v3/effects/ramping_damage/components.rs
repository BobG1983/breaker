//! Ramping damage runtime components.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

/// Accumulated bonus damage from consecutive hits without missing.
#[derive(Component, Debug, Clone)]
pub struct RampingDamageAccumulator(pub OrderedFloat<f32>);
