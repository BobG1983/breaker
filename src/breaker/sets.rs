//! Breaker domain system sets for cross-domain ordering.

use bevy::prelude::*;

/// System sets exported by the breaker domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BreakerSystems {
    /// The `move_breaker` system — updates breaker position from input.
    Move,
}
