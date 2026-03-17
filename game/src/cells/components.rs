//! Cells domain components.

use bevy::prelude::*;

/// Marker component identifying a cell entity.
#[derive(Component, Debug)]
pub struct Cell;

/// Marker for cells that count toward node completion.
#[derive(Component, Debug)]
pub struct RequiredToClear;

/// Visual parameters for cell damage color feedback.
#[derive(Component, Debug, Clone)]
pub struct CellDamageVisuals {
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
pub struct CellWidth(pub f32);

impl CellWidth {
    /// Returns half the cell width.
    pub fn half_width(&self) -> f32 {
        self.0 / 2.0
    }
}

/// Full height of a cell in world units.
#[derive(Component, Debug)]
pub struct CellHeight(pub f32);

impl CellHeight {
    /// Returns half the cell height.
    pub fn half_height(&self) -> f32 {
        self.0 / 2.0
    }
}

/// Health of a cell — number of hits remaining before destruction.
#[derive(Component, Debug, Clone)]
pub struct CellHealth {
    /// Current hit points.
    pub current: u32,
    /// Maximum hit points (used for visual damage feedback).
    pub max: u32,
}

impl CellHealth {
    /// Creates a new cell health with the given max HP.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(hp: u32) -> Self {
        Self {
            current: hp,
            max: hp,
        }
    }

    /// Returns true if the cell has been destroyed (0 HP).
    #[allow(clippy::missing_const_for_fn)]
    pub fn is_destroyed(&self) -> bool {
        self.current == 0
    }

    /// Applies one hit of damage. Returns true if the cell was destroyed.
    pub fn take_hit(&mut self) -> bool {
        self.current = self.current.saturating_sub(1);
        self.is_destroyed()
    }

    /// Returns the health fraction (0.0 to 1.0) for visual feedback.
    pub fn fraction(&self) -> f32 {
        if self.max == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let frac = self.current as f32 / self.max as f32;
        frac
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_width_half_width() {
        let w = CellWidth(70.0);
        assert!((w.half_width() - 35.0).abs() < f32::EPSILON);
    }

    #[test]
    fn cell_height_half_height() {
        let h = CellHeight(24.0);
        assert!((h.half_height() - 12.0).abs() < f32::EPSILON);
    }

    #[test]
    fn cell_health_standard() {
        let health = CellHealth::new(1);
        assert_eq!(health.current, 1);
        assert_eq!(health.max, 1);
        assert!(!health.is_destroyed());
    }

    #[test]
    fn cell_health_take_hit_destroys_at_zero() {
        let mut health = CellHealth::new(1);
        let destroyed = health.take_hit();
        assert!(destroyed);
        assert!(health.is_destroyed());
    }

    #[test]
    fn cell_health_tough_cell() {
        let mut health = CellHealth::new(3);
        assert!(!health.take_hit());
        assert!(!health.take_hit());
        assert!(health.take_hit());
        assert!(health.is_destroyed());
    }

    #[test]
    fn cell_health_fraction() {
        let mut health = CellHealth::new(4);
        assert!((health.fraction() - 1.0).abs() < f32::EPSILON);
        health.take_hit();
        assert!((health.fraction() - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn cell_health_saturating_sub() {
        let mut health = CellHealth::new(0);
        health.take_hit(); // should not underflow
        assert_eq!(health.current, 0);
    }
}
