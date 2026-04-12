//! Playfield configuration — dimensions, boundaries, and cell zone.

use bevy::prelude::*;
use rantzsoft_defaults::GameConfig;

use super::color_from_rgb;

/// Playfield configuration resource.
#[derive(Resource, Debug, Clone, PartialEq, GameConfig)]
#[game_config(
    defaults = "PlayfieldDefaults",
    path = "config/defaults.playfield.ron",
    ext = "playfield.ron"
)]
pub struct PlayfieldConfig {
    /// Width of the playfield in world units.
    pub width:                f32,
    /// Height of the playfield in world units.
    pub height:               f32,
    /// RGB values for the background clear color.
    pub background_color_rgb: [f32; 3],
    /// Thickness of boundary walls in world units.
    pub wall_thickness:       f32,
    /// Fraction of height reserved for the cell zone (0.0 to 1.0).
    pub zone_fraction:        f32,
}

impl Default for PlayfieldConfig {
    fn default() -> Self {
        Self {
            width:                800.0,
            height:               600.0,
            background_color_rgb: [0.02, 0.01, 0.04],
            wall_thickness:       180.0,
            zone_fraction:        0.667,
        }
    }
}

impl PlayfieldConfig {
    /// Left boundary x coordinate.
    #[must_use]
    pub fn left(&self) -> f32 {
        -self.width / 2.0
    }

    /// Right boundary x coordinate.
    #[must_use]
    pub fn right(&self) -> f32 {
        self.width / 2.0
    }

    /// Bottom boundary y coordinate.
    #[must_use]
    pub fn bottom(&self) -> f32 {
        -self.height / 2.0
    }

    /// Top boundary y coordinate.
    #[must_use]
    pub fn top(&self) -> f32 {
        self.height / 2.0
    }

    /// Half the wall thickness.
    #[must_use]
    pub fn wall_half_thickness(&self) -> f32 {
        self.wall_thickness / 2.0
    }

    /// Background clear color as a Bevy [`Color`].
    #[must_use]
    pub const fn background_color(&self) -> Color {
        color_from_rgb(self.background_color_rgb)
    }

    /// Height of the cell zone in world units.
    #[must_use]
    pub fn cell_zone_height(&self) -> f32 {
        self.height * self.zone_fraction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playfield_boundaries_are_symmetric() {
        let config = PlayfieldConfig::default();
        assert!((config.left() + config.right()).abs() < f32::EPSILON);
        assert!((config.bottom() + config.top()).abs() < f32::EPSILON);
    }

    #[test]
    fn playfield_dimensions_match_boundaries() {
        let config = PlayfieldConfig::default();
        assert!((config.right() - config.left() - config.width).abs() < f32::EPSILON);
        assert!((config.top() - config.bottom() - config.height).abs() < f32::EPSILON);
    }

    #[test]
    fn playfield_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.playfield.ron");
        let result: PlayfieldDefaults =
            ron::de::from_str(ron_str).expect("playfield RON should parse");
        assert!(result.width > 0.0);
        assert!((result.zone_fraction - 0.667).abs() < f32::EPSILON);
    }

    #[test]
    fn playfield_config_default_includes_zone_fraction() {
        let config = PlayfieldConfig::default();
        assert!(
            (config.zone_fraction - 0.667).abs() < f32::EPSILON,
            "expected zone_fraction ~0.667, got {}",
            config.zone_fraction,
        );
    }

    #[test]
    fn cell_zone_height_computes_fraction_of_height() {
        let config = PlayfieldConfig {
            height: 1080.0,
            zone_fraction: 0.667,
            ..Default::default()
        };
        let expected = 1080.0 * 0.667;
        assert!(
            (config.cell_zone_height() - expected).abs() < 0.01,
            "expected cell_zone_height ~{expected}, got {}",
            config.cell_zone_height(),
        );
    }

    #[test]
    fn cell_zone_height_with_zero_fraction_returns_zero() {
        let config = PlayfieldConfig {
            height: 1080.0,
            zone_fraction: 0.0,
            ..Default::default()
        };
        assert!(
            config.cell_zone_height().abs() < f32::EPSILON,
            "expected cell_zone_height 0.0, got {}",
            config.cell_zone_height(),
        );
    }
}
