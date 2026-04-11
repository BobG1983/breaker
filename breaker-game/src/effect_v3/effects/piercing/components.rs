//! Piercing runtime components.

use bevy::prelude::*;

/// Number of cells the bolt can pierce through without bouncing.
#[derive(Component, Debug, Clone)]
pub struct PiercingRemaining(pub u32);
