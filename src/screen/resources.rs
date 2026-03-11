//! Screen domain resources.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use brickbreaker_derive::GameConfig;
use serde::Deserialize;

use super::components::MenuItem;
use crate::{
    bolt::BoltDefaults, breaker::BreakerDefaults, cells::CellDefaults, physics::PhysicsDefaults,
    shared::PlayfieldDefaults,
};

/// Asset collection for all defaults — automatically loaded during
/// [`GameState::Loading`] by `bevy_asset_loader`.
#[derive(AssetCollection, Resource)]
pub struct DefaultsCollection {
    /// Handle for playfield defaults.
    #[asset(path = "config/defaults.playfield.ron")]
    pub playfield: Handle<PlayfieldDefaults>,
    /// Handle for bolt defaults.
    #[asset(path = "config/defaults.bolt.ron")]
    pub bolt: Handle<BoltDefaults>,
    /// Handle for breaker defaults.
    #[asset(path = "config/defaults.breaker.ron")]
    pub breaker: Handle<BreakerDefaults>,
    /// Handle for cells defaults.
    #[asset(path = "config/defaults.cells.ron")]
    pub cells: Handle<CellDefaults>,
    /// Handle for physics defaults.
    #[asset(path = "config/defaults.physics.ron")]
    pub physics: Handle<PhysicsDefaults>,
    /// Handle for main menu defaults.
    #[asset(path = "config/defaults.mainmenu.ron")]
    pub mainmenu: Handle<MainMenuDefaults>,
}

/// Tracks the currently selected menu item.
#[derive(Resource, Debug)]
pub struct MainMenuSelection {
    /// The currently highlighted menu item.
    pub selected: MenuItem,
}

/// Main menu defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
#[game_config(name = "MainMenuConfig")]
pub struct MainMenuDefaults {
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

impl Default for MainMenuDefaults {
    fn default() -> Self {
        Self {
            title_font_size: 96.0,
            menu_font_size: 36.0,
            title_color_rgb: [2.0, 4.0, 5.0],
            selected_color_rgb: [0.4, 3.0, 4.0],
            normal_color_rgb: [0.6, 0.6, 0.7],
            disabled_color_rgb: [0.25, 0.25, 0.3],
            title_bottom_margin: 48.0,
            menu_item_gap: 12.0,
            title_font_path: "fonts/Orbitron-Bold.ttf".to_owned(),
            menu_font_path: "fonts/Rajdhani-Medium.ttf".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_menu_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.mainmenu.ron");
        let result: MainMenuDefaults =
            ron::de::from_str(ron_str).expect("mainmenu RON should parse");
        assert!(result.title_font_size > 0.0);
    }
}
