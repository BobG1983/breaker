//! Loading screen components.

use bevy::prelude::*;

/// Marker component for loading screen entities.
#[derive(Component)]
pub(super) struct LoadingScreen;

/// Marker for the loading progress bar inner fill.
#[derive(Component)]
pub(super) struct LoadingBarFill;

/// Marker for the loading progress text.
#[derive(Component)]
pub(super) struct LoadingProgressText;
