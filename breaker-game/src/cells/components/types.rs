//! Cells domain components.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Spatial2D;

use crate::shared::CleanupOnNodeExit;

/// Marker component identifying a cell entity.
#[derive(Component, Debug, Default)]
#[require(Spatial2D, CleanupOnNodeExit)]
pub(crate) struct Cell;

/// Marker for cells that count toward node completion.
#[derive(Component, Debug)]
pub struct RequiredToClear;

/// Tracks which cell type definition alias spawned this cell.
/// Used by hot-reload to update live cells when their type definition changes.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CellTypeAlias(pub char);

/// Visual parameters for cell damage color feedback.
#[derive(Component, Debug, Clone)]
pub(crate) struct CellDamageVisuals {
    /// HDR intensity multiplier at full health.
    pub hdr_base: f32,
    /// Minimum green channel value.
    pub green_min: f32,
    /// Blue channel range added based on health fraction.
    pub blue_range: f32,
    /// Base blue channel value.
    pub blue_base: f32,
}

/// Full width of a cell in world units.
#[derive(Component, Debug)]
pub(crate) struct CellWidth {
    #[cfg(any(test, feature = "dev"))]
    pub value: f32,
}

impl CellWidth {
    /// Creates a new `CellWidth`.
    #[cfg(any(test, feature = "dev"))]
    pub(crate) const fn new(value: f32) -> Self {
        Self { value }
    }

    /// Creates a new `CellWidth` (value unused outside test/dev).
    #[cfg(not(any(test, feature = "dev")))]
    pub(crate) const fn new(_value: f32) -> Self {
        Self {}
    }

    /// Returns half the cell width.
    #[cfg(test)]
    pub(crate) fn half_width(&self) -> f32 {
        self.value / 2.0
    }
}

/// Full height of a cell in world units.
#[derive(Component, Debug)]
pub(crate) struct CellHeight {
    #[cfg(any(test, feature = "dev"))]
    pub value: f32,
}

impl CellHeight {
    /// Creates a new `CellHeight`.
    #[cfg(any(test, feature = "dev"))]
    pub(crate) const fn new(value: f32) -> Self {
        Self { value }
    }

    /// Creates a new `CellHeight` (value unused outside test/dev).
    #[cfg(not(any(test, feature = "dev")))]
    pub(crate) const fn new(_value: f32) -> Self {
        Self {}
    }

    /// Returns half the cell height.
    #[cfg(test)]
    pub(crate) fn half_height(&self) -> f32 {
        self.value / 2.0
    }
}

/// Health of a cell — hit points remaining before destruction.
#[derive(Component, Debug, Clone)]
pub(crate) struct CellHealth {
    /// Current hit points.
    pub current: f32,
    /// Maximum hit points (used for visual damage feedback).
    pub max: f32,
}

impl CellHealth {
    /// Creates a new cell health with the given max HP.
    pub(crate) const fn new(hp: f32) -> Self {
        Self {
            current: hp,
            max: hp,
        }
    }

    /// Returns true if the cell has been destroyed (HP at or below 0).
    pub(crate) const fn is_destroyed(&self) -> bool {
        self.current <= 0.0
    }

    /// Applies the given damage amount. Returns true if the cell was destroyed.
    pub(crate) fn take_damage(&mut self, amount: f32) -> bool {
        self.current -= amount;
        self.is_destroyed()
    }

    /// Returns the health fraction (0.0 to 1.0) for visual feedback.
    pub(crate) fn fraction(&self) -> f32 {
        if self.max == 0.0 {
            return 0.0;
        }
        (self.current / self.max).clamp(0.0, 1.0)
    }
}

/// Marker component — cell is locked and immune to damage.
///
/// Removed by `check_lock_release` when all adjacent cells are destroyed.
#[derive(Component, Debug)]
pub(crate) struct Locked;

/// Tracks which adjacent cells must be destroyed to unlock this cell.
#[derive(Component, Debug)]
pub(crate) struct LockAdjacents(pub Vec<Entity>);

/// Cell regenerates HP at this rate per second.
#[derive(Component, Debug)]
pub(crate) struct CellRegen {
    /// HP regenerated per second.
    pub rate: f32,
}

/// Marker component identifying a shield cell (spawns orbiting children).
#[derive(Component, Debug)]
pub(crate) struct ShieldParent;

/// Marker component identifying an orbit cell (child of a shield).
#[derive(Component, Debug)]
pub(crate) struct OrbitCell;

/// Current angular position of an orbit cell around its parent shield.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct OrbitAngle(pub f32);

/// Configuration for an orbit cell's circular motion.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct OrbitConfig {
    /// Distance from shield center to orbit cell center.
    pub radius: f32,
    /// Angular speed in radians per second.
    pub speed: f32,
}
