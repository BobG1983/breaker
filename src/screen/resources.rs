//! Screen domain resources.

use bevy::prelude::*;

/// Main menu visual configuration, seeded from [`super::defaults::MainMenuDefaults`].
#[derive(Resource, Clone, Debug)]
pub struct MainMenuConfig {
    /// Font size for the title text.
    pub title_font_size: f32,
    /// Font size for menu item text.
    pub menu_font_size: f32,
    /// HDR RGB color for the title.
    pub title_color_rgb: [f32; 3],
    /// HDR RGB color for the selected menu item.
    pub selected_color_rgb: [f32; 3],
    /// RGB color for unselected menu items.
    pub normal_color_rgb: [f32; 3],
    /// RGB color for disabled menu items.
    pub disabled_color_rgb: [f32; 3],
    /// Bottom margin below the title in pixels.
    pub title_bottom_margin: f32,
    /// Gap between menu items in pixels.
    pub menu_item_gap: f32,
    /// Asset path for the title font.
    pub title_font_path: String,
    /// Asset path for the menu font.
    pub menu_font_path: String,
}
