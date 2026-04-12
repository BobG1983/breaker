//! Entropy engine systems — reset counter on node start.

use bevy::prelude::*;

use super::components::EntropyCounter;

/// Resets all `EntropyCounter` components to zero at the start of each node.
pub fn reset_entropy_counter(mut query: Query<&mut EntropyCounter>) {
    for mut counter in &mut query {
        counter.count = 0;
    }
}
