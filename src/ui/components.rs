//! UI domain components.

use bevy::prelude::*;

/// Marker component on the node timer display text entity.
#[derive(Component)]
pub struct NodeTimerDisplay;

/// Marker component on the side panel chrome root entity.
#[derive(Component, Debug)]
pub struct SidePanels;

/// Marker component on the right status panel.
#[derive(Component, Debug)]
pub struct StatusPanel;
