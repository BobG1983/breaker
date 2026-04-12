//! Main menu resources.

use bevy::prelude::*;
use rantzsoft_defaults::GameConfig;

use super::components::MenuItem;

/// Tracks the currently selected menu item.
#[derive(Resource, Debug)]
pub(crate) struct MainMenuSelection {
    /// The currently highlighted menu item.
    pub selected: MenuItem,
}

/// Main menu configuration resource.
#[derive(Resource, Debug, Clone, PartialEq, GameConfig)]
#[game_config(
    defaults = "MainMenuDefaults",
    path = "config/defaults.mainmenu.ron",
    ext = "mainmenu.ron"
)]
pub(crate) struct MainMenuConfig {
    /// Font size for the title text.
    pub title_font_size:     f32,
    /// Font size for menu item text.
    pub menu_font_size:      f32,
    /// HDR RGB color for the title.
    pub title_color_rgb:     [f32; 3],
    /// HDR RGB color for the selected menu item.
    pub selected_color_rgb:  [f32; 3],
    /// RGB color for unselected menu items.
    pub normal_color_rgb:    [f32; 3],
    /// RGB color for disabled menu items.
    pub disabled_color_rgb:  [f32; 3],
    /// Bottom margin below the title in pixels.
    pub title_bottom_margin: f32,
    /// Gap between menu items in pixels.
    pub menu_item_gap:       f32,
    /// Asset path for the title font.
    pub title_font_path:     String,
    /// Asset path for the menu font.
    pub menu_font_path:      String,
}

impl Default for MainMenuConfig {
    fn default() -> Self {
        Self {
            title_font_size:     96.0,
            menu_font_size:      36.0,
            title_color_rgb:     [2.0, 4.0, 5.0],
            selected_color_rgb:  [0.4, 3.0, 4.0],
            normal_color_rgb:    [0.6, 0.6, 0.7],
            disabled_color_rgb:  [0.25, 0.25, 0.3],
            title_bottom_margin: 48.0,
            menu_item_gap:       12.0,
            title_font_path:     "fonts/Orbitron-Bold.ttf".to_owned(),
            menu_font_path:      "fonts/Rajdhani-Medium.ttf".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_menu_defaults_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/config/defaults.mainmenu.ron"
        ));
        let result: MainMenuDefaults =
            ron::de::from_str(ron_str).expect("mainmenu RON should parse");
        assert!(result.title_font_size > 0.0);
    }
}
