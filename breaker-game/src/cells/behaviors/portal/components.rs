//! Portal behavior components.

use bevy::prelude::*;

/// Permanent marker identifying a cell as a portal-type cell.
///
/// Portal cells are invulnerable to damage and can only be cleared
/// by entering them (bolt collision → `PortalEntered` → `PortalCompleted`).
#[derive(Component, Debug)]
pub struct PortalCell;

/// Configuration for portal cells.
///
/// `tier_offset` controls the sub-node difficulty offset when the bolt
/// enters the portal (wired in node refactor).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PortalConfig {
    pub(crate) tier_offset: i32,
}
