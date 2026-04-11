//! Flash step runtime components.

use bevy::prelude::*;

/// Marker indicating the breaker has flash-step movement enabled.
#[derive(Component, Debug, Clone)]
pub struct FlashStepActive;
