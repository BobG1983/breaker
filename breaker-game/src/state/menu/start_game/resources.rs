//! Run setup screen resources.

use bevy::prelude::*;

/// Resource tracking the currently highlighted breaker card index.
#[derive(Resource, Debug)]
pub(crate) struct RunSetupSelection {
    /// Index into the sorted breaker names list.
    pub index: usize,
}

/// Resource tracking the seed text entry field on the run setup screen.
#[derive(Resource, Debug, Default)]
pub(crate) struct SeedEntry {
    /// The typed seed digits (empty = random).
    pub value: String,
    /// Whether the seed field is focused for input.
    pub focused: bool,
}
