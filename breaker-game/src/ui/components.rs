//! UI domain components.

use bevy::prelude::*;

/// Marker component on the node timer display text entity.
#[derive(Component)]
pub(crate) struct NodeTimerDisplay;

/// Marker component on the side panel chrome root entity.
#[derive(Component, Debug)]
pub(crate) struct SidePanels;

/// Marker component on the right status panel.
#[derive(Component, Debug)]
pub(crate) struct StatusPanel;
