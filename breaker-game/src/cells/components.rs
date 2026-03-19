//! Cells domain components.

use bevy::prelude::*;

/// Marker component identifying a cell entity.
#[derive(Component, Debug)]
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
pub(crate) struct CellWidth(pub f32);

impl CellWidth {
    /// Returns half the cell width.
    pub(crate) fn half_width(&self) -> f32 {
        self.0 / 2.0
    }
}

/// Full height of a cell in world units.
#[derive(Component, Debug)]
pub(crate) struct CellHeight(pub f32);

impl CellHeight {
    /// Returns half the cell height.
    pub(crate) fn half_height(&self) -> f32 {
        self.0 / 2.0
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
    fn cell_health_new_sets_current_and_max_to_given_hp() {
        let health = CellHealth::new(1.0);
        assert!((health.current - 1.0).abs() < f32::EPSILON);
        assert!((health.max - 1.0).abs() < f32::EPSILON);
        assert!(!health.is_destroyed());
    }

    #[test]
    fn cell_health_not_destroyed_at_positive_hp() {
        let health = CellHealth::new(10.0);
        assert!(
            !health.is_destroyed(),
            "10.0 HP cell should not be destroyed"
        );
    }

    #[test]
    fn cell_health_is_destroyed_at_exactly_zero() {
        let health = CellHealth {
            current: 0.0,
            max: 10.0,
        };
        assert!(
            health.is_destroyed(),
            "cell at exactly 0.0 HP should be destroyed"
        );
    }

    #[test]
    fn cell_health_is_destroyed_when_negative_current() {
        // Edge case: overkill drives current below 0.0 — must still be destroyed.
        let health = CellHealth {
            current: -5.0,
            max: 10.0,
        };
        assert!(
            health.is_destroyed(),
            "cell with negative current HP (-5.0) should be destroyed (current <= 0.0)"
        );
    }

    #[test]
    fn take_damage_10_destroys_10hp_cell() {
        let mut health = CellHealth::new(10.0);
        let destroyed = health.take_damage(10.0);
        assert!(
            destroyed,
            "take_damage(10.0) on 10.0-HP cell should return true"
        );
        assert!(
            health.current <= 0.0,
            "HP after lethal damage should be <= 0.0, got {}",
            health.current
        );
        assert!(health.is_destroyed());
    }

    #[test]
    fn take_damage_15_on_10hp_cell_overkill() {
        // Overkill: damage exceeds remaining HP. Result must be <= 0 and destroyed=true.
        let mut health = CellHealth::new(10.0);
        let destroyed = health.take_damage(15.0);
        assert!(
            destroyed,
            "take_damage(15.0) on 10.0-HP cell should destroy it"
        );
        assert!(
            health.current <= 0.0,
            "overkill should leave HP <= 0.0, got {}",
            health.current
        );
    }

    #[test]
    fn take_damage_10_on_30hp_cell_survives_with_20hp() {
        let mut health = CellHealth::new(30.0);
        let destroyed = health.take_damage(10.0);
        assert!(
            !destroyed,
            "take_damage(10.0) on 30.0-HP cell should not destroy it"
        );
        assert!(
            (health.current - 20.0).abs() < f32::EPSILON,
            "30.0 HP - 10.0 damage = 20.0 HP remaining, got {}",
            health.current
        );
    }

    #[test]
    fn take_damage_zero_on_already_dead_cell_returns_true() {
        // A cell already at 0 HP: take_damage(0.0) should still report destroyed.
        let mut health = CellHealth::new(10.0);
        health.take_damage(10.0); // kill it first
        let destroyed = health.take_damage(0.0);
        assert!(
            destroyed,
            "take_damage(0.0) on already-dead cell should return true (is_destroyed)"
        );
    }

    #[test]
    fn take_damage_5_on_dead_cell_stays_destroyed() {
        // Dead cells should not go more negative than needed — and must remain destroyed.
        let mut health = CellHealth::new(10.0);
        health.take_damage(10.0); // kill it
        let destroyed = health.take_damage(5.0);
        assert!(destroyed, "take_damage on dead cell should return true");
        assert!(
            health.current <= 0.0,
            "dead cell should have HP <= 0.0 after further damage, got {}",
            health.current
        );
    }

    #[test]
    fn cell_health_fraction_at_full_hp() {
        let health = CellHealth::new(4.0);
        assert!(
            (health.fraction() - 1.0).abs() < f32::EPSILON,
            "fraction at full HP should be 1.0"
        );
    }

    #[test]
    fn cell_health_fraction_after_one_damage() {
        let mut health = CellHealth::new(4.0);
        health.take_damage(1.0);
        assert!(
            (health.fraction() - 0.75).abs() < 1e-5,
            "fraction after 1 damage on 4-HP cell should be 0.75, got {}",
            health.fraction()
        );
    }

    #[test]
    fn cell_health_fraction_with_zero_max_returns_zero() {
        // Edge case: max == 0.0 guard prevents divide-by-zero.
        let health = CellHealth {
            current: 0.0,
            max: 0.0,
        };
        assert!(
            (health.fraction() - 0.0).abs() < f32::EPSILON,
            "fraction with max=0.0 should return 0.0"
        );
    }

    #[test]
    fn cell_health_fraction_with_negative_current_clamped() {
        // Negative current (overkill) — fraction should not go below 0.0.
        let health = CellHealth {
            current: -5.0,
            max: 10.0,
        };
        assert!(
            health.fraction() <= 0.0,
            "fraction with negative current should be <= 0.0, got {}",
            health.fraction()
        );
    }
}
