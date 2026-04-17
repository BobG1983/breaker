//! Hazard selection screen resources.

use bevy::prelude::*;
use rantzsoft_defaults::GameConfig;

/// Hazard select configuration resource.
#[derive(Resource, Debug, Clone, PartialEq, GameConfig)]
#[game_config(
    defaults = "HazardSelectDefaults",
    path = "config/defaults.hazardselect.ron",
    ext = "hazardselect.ron"
)]
pub(crate) struct HazardSelectConfig {
    /// Time in seconds for the selection countdown.
    pub timer_secs:                 f32,
    /// Font size for card title text.
    pub card_title_font_size:       f32,
    /// Font size for card description text.
    pub card_description_font_size: f32,
    /// Font size for the countdown timer display.
    pub timer_font_size:            f32,
    /// RGB color for the selected card border.
    pub selected_color_rgb:         [f32; 3],
    /// RGB color for unselected card borders.
    pub normal_color_rgb:           [f32; 3],
    /// RGB color for the timer text.
    pub timer_color_rgb:            [f32; 3],
}

impl Default for HazardSelectConfig {
    fn default() -> Self {
        Self {
            timer_secs:                 10.0,
            card_title_font_size:       36.0,
            card_description_font_size: 20.0,
            timer_font_size:            48.0,
            selected_color_rgb:         [0.4, 0.8, 1.0],
            normal_color_rgb:           [0.3, 0.3, 0.4],
            timer_color_rgb:            [1.0, 0.8, 0.2],
        }
    }
}

/// Screen-local countdown timer for the hazard selection screen.
#[derive(Resource, Debug)]
pub(crate) struct HazardSelectTimer {
    /// Remaining time in seconds.
    pub remaining: f32,
}

/// Tracks which hazard card is currently highlighted.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct HazardSelectSelection {
    /// Zero-based index of the currently selected hazard card.
    pub card_index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Element-wise approximate equality for RGB triples — avoids clippy's
    /// `float_cmp` lint on exact `assert_eq!` for `f32` arrays.
    fn assert_rgb_eq(actual: [f32; 3], expected: [f32; 3]) {
        for (a, e) in actual.iter().zip(expected.iter()) {
            assert!(
                (a - e).abs() < 1e-6,
                "RGB component mismatch: actual {actual:?}, expected {expected:?}"
            );
        }
    }

    // ── Domain H.1: default config matches canonical values ──────────────

    #[test]
    fn default_config_has_canonical_timer_and_colors() {
        let config = HazardSelectConfig::default();
        assert!(
            (config.timer_secs - 10.0).abs() < f32::EPSILON,
            "timer_secs default should be 10.0, got {}",
            config.timer_secs
        );
        assert!(
            (config.card_title_font_size - 36.0).abs() < f32::EPSILON,
            "card_title_font_size default should be 36.0"
        );
        assert!(
            (config.card_description_font_size - 20.0).abs() < f32::EPSILON,
            "card_description_font_size default should be 20.0"
        );
        assert!(
            (config.timer_font_size - 48.0).abs() < f32::EPSILON,
            "timer_font_size default should be 48.0"
        );
        assert_rgb_eq(config.selected_color_rgb, [0.4, 0.8, 1.0]);
        assert_rgb_eq(config.normal_color_rgb, [0.3, 0.3, 0.4]);
        assert_rgb_eq(config.timer_color_rgb, [1.0, 0.8, 0.2]);
    }

    #[test]
    fn default_config_has_positive_timer() {
        let config = HazardSelectConfig::default();
        assert!(config.timer_secs > 0.0);
    }

    // ── Domain H.2: defaults.hazardselect.ron parses into HazardSelectDefaults ──

    #[test]
    fn hazard_select_defaults_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/config/defaults.hazardselect.ron"
        ));
        let result: HazardSelectDefaults =
            ron::de::from_str(ron_str).expect("hazardselect RON should parse");
        assert!(result.timer_secs > 0.0);
    }
}
