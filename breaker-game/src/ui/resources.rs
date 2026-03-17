//! UI domain resources — timer display configuration.

use bevy::prelude::*;
use breaker_derive::GameConfig;
use serde::Deserialize;

use crate::shared::color_from_rgb;

/// Timer UI defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
#[game_config(name = "TimerUiConfig")]
pub struct TimerUiDefaults {
    /// Font size for the timer display.
    pub font_size: f32,
    /// Asset path for the timer font.
    pub font_path: String,
    /// RGB color when plenty of time remains.
    pub normal_color_rgb: [f32; 3],
    /// RGB color when time is running low.
    pub warning_color_rgb: [f32; 3],
    /// RGB color when time is critically low.
    pub urgent_color_rgb: [f32; 3],
    /// Fraction of total time below which warning color activates.
    pub warning_threshold: f32,
    /// Fraction of total time below which urgent color activates.
    pub urgent_threshold: f32,
}

impl Default for TimerUiDefaults {
    fn default() -> Self {
        Self {
            font_size: 48.0,
            font_path: "fonts/Rajdhani-Medium.ttf".to_owned(),
            normal_color_rgb: [0.8, 0.8, 0.9],
            warning_color_rgb: [4.0, 3.0, 0.2],
            urgent_color_rgb: [5.0, 0.3, 0.2],
            warning_threshold: 0.33,
            urgent_threshold: 0.15,
        }
    }
}

impl TimerUiConfig {
    /// Returns the appropriate color for the given time fraction.
    #[must_use]
    pub fn color_for_fraction(&self, fraction: f32) -> Color {
        if fraction <= self.urgent_threshold {
            color_from_rgb(self.urgent_color_rgb)
        } else if fraction <= self.warning_threshold {
            color_from_rgb(self.warning_color_rgb)
        } else {
            color_from_rgb(self.normal_color_rgb)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_ui_defaults_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/config/defaults.timerui.ron"
        ));
        let result: TimerUiDefaults = ron::de::from_str(ron_str).expect("timerui RON should parse");
        assert!(result.font_size > 0.0);
        assert!(result.warning_threshold > result.urgent_threshold);
    }

    #[test]
    fn color_for_fraction_returns_normal_above_warning() {
        let config = TimerUiConfig::default();
        let color = config.color_for_fraction(0.8);
        assert_eq!(color, color_from_rgb(config.normal_color_rgb));
    }

    #[test]
    fn color_for_fraction_returns_warning_between_thresholds() {
        let config = TimerUiConfig::default();
        let color = config.color_for_fraction(0.3);
        assert_eq!(color, color_from_rgb(config.warning_color_rgb));
    }

    #[test]
    fn color_for_fraction_returns_urgent_below_urgent_threshold() {
        let config = TimerUiConfig::default();
        let color = config.color_for_fraction(0.1);
        assert_eq!(color, color_from_rgb(config.urgent_color_rgb));
    }
}
