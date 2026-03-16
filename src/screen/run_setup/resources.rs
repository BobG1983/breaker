//! Run setup screen resources.

use bevy::prelude::*;

/// Resource tracking the currently highlighted breaker card index.
#[derive(Resource, Debug)]
pub struct RunSetupSelection {
    /// Index into the sorted archetype names list.
    pub index: usize,
}
