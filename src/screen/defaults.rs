//! RON-loaded default configuration types.
//!
//! Each `*Defaults` type is an asset loaded from a `.defaults.ron` file.
//! On load, each is converted into the corresponding `*Config` resource
//! via [`From`] implementations.

use bevy::prelude::*;
use serde::Deserialize;

use crate::bolt::BoltConfig;
use crate::breaker::BreakerConfig;
use crate::cells::CellConfig;
use crate::physics::PhysicsConfig;
use crate::shared::PlayfieldConfig;

use super::resources::MainMenuConfig;

/// Playfield defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct PlayfieldDefaults {
    /// Width of the playfield in world units.
    pub width: f32,
    /// Height of the playfield in world units.
    pub height: f32,
    /// RGB values for the background clear color.
    pub background_color_rgb: [f32; 3],
}

impl Default for PlayfieldDefaults {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            background_color_rgb: [0.02, 0.01, 0.04],
        }
    }
}

impl From<PlayfieldDefaults> for PlayfieldConfig {
    fn from(d: PlayfieldDefaults) -> Self {
        Self {
            width: d.width,
            height: d.height,
            background_color_rgb: d.background_color_rgb,
        }
    }
}

/// Bolt defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct BoltDefaults {
    /// Base speed in world units per second.
    pub base_speed: f32,
    /// Minimum speed cap.
    pub min_speed: f32,
    /// Maximum speed cap.
    pub max_speed: f32,
    /// Minimum angle from horizontal in radians.
    pub min_angle_from_horizontal: f32,
    /// Bolt radius in world units.
    pub radius: f32,
    /// Vertical offset above the breaker where the bolt spawns.
    pub spawn_offset_y: f32,
    /// Initial launch angle from vertical in radians.
    pub initial_angle: f32,
    /// Vertical offset above the breaker for bolt respawn after loss.
    pub respawn_offset_y: f32,
    /// RGB values for the bolt HDR color.
    pub color_rgb: [f32; 3],
}

impl Default for BoltDefaults {
    fn default() -> Self {
        Self {
            base_speed: 400.0,
            min_speed: 200.0,
            max_speed: 800.0,
            min_angle_from_horizontal: 0.17,
            radius: 8.0,
            spawn_offset_y: 30.0,
            initial_angle: 0.26,
            respawn_offset_y: 30.0,
            color_rgb: [6.0, 5.0, 0.5],
        }
    }
}

impl From<BoltDefaults> for BoltConfig {
    fn from(d: BoltDefaults) -> Self {
        Self {
            base_speed: d.base_speed,
            min_speed: d.min_speed,
            max_speed: d.max_speed,
            min_angle_from_horizontal: d.min_angle_from_horizontal,
            radius: d.radius,
            spawn_offset_y: d.spawn_offset_y,
            initial_angle: d.initial_angle,
            respawn_offset_y: d.respawn_offset_y,
            color_rgb: d.color_rgb,
        }
    }
}

/// Breaker defaults loaded from RON.
#[allow(clippy::struct_excessive_bools)]
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct BreakerDefaults {
    /// Half-width of the breaker in world units.
    pub half_width: f32,
    /// Half-height of the breaker in world units.
    pub half_height: f32,
    /// Maximum horizontal speed in world units per second.
    pub max_speed: f32,
    /// Horizontal acceleration in world units per second squared.
    pub acceleration: f32,
    /// Horizontal deceleration (friction) in world units per second squared.
    pub deceleration: f32,
    /// Dash speed multiplier relative to max speed.
    pub dash_speed_multiplier: f32,
    /// Duration of the dash in seconds.
    pub dash_duration: f32,
    /// Brake deceleration multiplier relative to normal deceleration.
    pub brake_decel_multiplier: f32,
    /// Duration of the settle phase in seconds.
    pub settle_duration: f32,
    /// Maximum tilt angle during dash in radians.
    pub dash_tilt_angle: f32,
    /// Maximum tilt angle during brake in radians.
    pub brake_tilt_angle: f32,
    /// Y position of the breaker.
    pub y_position: f32,
    /// Duration of the bump active window in seconds.
    pub bump_duration: f32,
    /// Cooldown between bumps in seconds.
    pub bump_cooldown: f32,
    /// Perfect bump timing window (seconds).
    pub perfect_bump_window: f32,
    /// Early bump window (seconds).
    pub early_bump_window: f32,
    /// Velocity multiplier for perfect bump.
    pub perfect_bump_multiplier: f32,
    /// Velocity multiplier for early/late bump.
    pub weak_bump_multiplier: f32,
    /// Velocity multiplier for no bump.
    pub no_bump_multiplier: f32,
    /// RGB values for the breaker HDR color.
    pub color_rgb: [f32; 3],
}

impl Default for BreakerDefaults {
    fn default() -> Self {
        Self {
            half_width: 60.0,
            half_height: 10.0,
            max_speed: 500.0,
            acceleration: 3000.0,
            deceleration: 2500.0,
            dash_speed_multiplier: 2.0,
            dash_duration: 0.15,
            brake_decel_multiplier: 4.0,
            settle_duration: 0.12,
            dash_tilt_angle: 0.26,
            brake_tilt_angle: 0.44,
            y_position: -250.0,
            bump_duration: 0.3,
            bump_cooldown: 0.3,
            perfect_bump_window: 0.05,
            early_bump_window: 0.15,
            perfect_bump_multiplier: 1.5,
            weak_bump_multiplier: 0.8,
            no_bump_multiplier: 1.0,
            color_rgb: [0.2, 2.0, 3.0],
        }
    }
}

impl From<BreakerDefaults> for BreakerConfig {
    fn from(d: BreakerDefaults) -> Self {
        Self {
            half_width: d.half_width,
            half_height: d.half_height,
            max_speed: d.max_speed,
            acceleration: d.acceleration,
            deceleration: d.deceleration,
            dash_speed_multiplier: d.dash_speed_multiplier,
            dash_duration: d.dash_duration,
            brake_decel_multiplier: d.brake_decel_multiplier,
            settle_duration: d.settle_duration,
            dash_tilt_angle: d.dash_tilt_angle,
            brake_tilt_angle: d.brake_tilt_angle,
            y_position: d.y_position,
            bump_duration: d.bump_duration,
            bump_cooldown: d.bump_cooldown,
            perfect_bump_window: d.perfect_bump_window,
            early_bump_window: d.early_bump_window,
            perfect_bump_multiplier: d.perfect_bump_multiplier,
            weak_bump_multiplier: d.weak_bump_multiplier,
            no_bump_multiplier: d.no_bump_multiplier,
            color_rgb: d.color_rgb,
        }
    }
}

/// Cell defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct CellDefaults {
    /// Half-width of a cell in world units.
    pub half_width: f32,
    /// Half-height of a cell in world units.
    pub half_height: f32,
    /// Horizontal padding between cells.
    pub padding_x: f32,
    /// Vertical padding between cells.
    pub padding_y: f32,
    /// Number of columns in the grid.
    pub grid_cols: u32,
    /// Number of rows in the grid.
    pub grid_rows: u32,
    /// Y offset from playfield top for grid start.
    pub grid_top_offset: f32,
    /// HP for standard cells.
    pub standard_hp: u32,
    /// HP for tough cells.
    pub tough_hp: u32,
    /// RGB values for standard cell HDR color.
    pub standard_color_rgb: [f32; 3],
    /// RGB values for tough cell HDR color.
    pub tough_color_rgb: [f32; 3],
    /// Row index (0-indexed from top) that contains tough cells.
    pub tough_row_index: u32,
    /// HDR intensity multiplier for damaged cells at full health.
    pub damage_hdr_base: f32,
    /// Minimum green channel value for damage color feedback.
    pub damage_green_min: f32,
    /// Green channel range added based on health fraction.
    pub damage_green_range: f32,
    /// Base blue channel value for damage color feedback.
    pub damage_blue_base: f32,
}

impl Default for CellDefaults {
    fn default() -> Self {
        Self {
            half_width: 35.0,
            half_height: 12.0,
            padding_x: 4.0,
            padding_y: 4.0,
            grid_cols: 10,
            grid_rows: 5,
            grid_top_offset: 50.0,
            standard_hp: 1,
            tough_hp: 3,
            standard_color_rgb: [4.0, 0.2, 0.5],
            tough_color_rgb: [2.5, 0.2, 4.0],
            tough_row_index: 0,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_green_range: 0.4,
            damage_blue_base: 0.2,
        }
    }
}

impl From<CellDefaults> for CellConfig {
    fn from(d: CellDefaults) -> Self {
        Self {
            half_width: d.half_width,
            half_height: d.half_height,
            padding_x: d.padding_x,
            padding_y: d.padding_y,
            grid_cols: d.grid_cols,
            grid_rows: d.grid_rows,
            grid_top_offset: d.grid_top_offset,
            standard_hp: d.standard_hp,
            tough_hp: d.tough_hp,
            standard_color_rgb: d.standard_color_rgb,
            tough_color_rgb: d.tough_color_rgb,
            tough_row_index: d.tough_row_index,
            damage_hdr_base: d.damage_hdr_base,
            damage_green_min: d.damage_green_min,
            damage_green_range: d.damage_green_range,
            damage_blue_base: d.damage_blue_base,
        }
    }
}

/// Physics defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct PhysicsDefaults {
    /// Maximum reflection angle from vertical in radians.
    pub max_reflection_angle: f32,
}

impl Default for PhysicsDefaults {
    fn default() -> Self {
        Self {
            max_reflection_angle: 1.31,
        }
    }
}

impl From<PhysicsDefaults> for PhysicsConfig {
    fn from(d: PhysicsDefaults) -> Self {
        Self {
            max_reflection_angle: d.max_reflection_angle,
        }
    }
}

/// Main menu defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
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

impl From<MainMenuDefaults> for MainMenuConfig {
    fn from(d: MainMenuDefaults) -> Self {
        Self {
            title_font_size: d.title_font_size,
            menu_font_size: d.menu_font_size,
            title_color_rgb: d.title_color_rgb,
            selected_color_rgb: d.selected_color_rgb,
            normal_color_rgb: d.normal_color_rgb,
            disabled_color_rgb: d.disabled_color_rgb,
            title_bottom_margin: d.title_bottom_margin,
            menu_item_gap: d.menu_item_gap,
            title_font_path: d.title_font_path,
            menu_font_path: d.menu_font_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bolt_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.bolt.ron");
        let result: BoltDefaults = ron::de::from_str(ron_str).expect("bolt RON should parse");
        assert!(result.base_speed > 0.0);
    }

    #[test]
    fn breaker_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.breaker.ron");
        let result: BreakerDefaults = ron::de::from_str(ron_str).expect("breaker RON should parse");
        assert!(result.half_width > 0.0);
    }

    #[test]
    fn cell_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.cells.ron");
        let result: CellDefaults = ron::de::from_str(ron_str).expect("cells RON should parse");
        assert!(result.grid_cols > 0);
    }

    #[test]
    fn physics_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.physics.ron");
        let result: PhysicsDefaults = ron::de::from_str(ron_str).expect("physics RON should parse");
        assert!(result.max_reflection_angle > 0.0);
    }

    #[test]
    fn main_menu_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.mainmenu.ron");
        let result: MainMenuDefaults =
            ron::de::from_str(ron_str).expect("mainmenu RON should parse");
        assert!(result.title_font_size > 0.0);
    }

    #[test]
    fn playfield_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.playfield.ron");
        let result: PlayfieldDefaults =
            ron::de::from_str(ron_str).expect("playfield RON should parse");
        assert!(result.width > 0.0);
    }
}
