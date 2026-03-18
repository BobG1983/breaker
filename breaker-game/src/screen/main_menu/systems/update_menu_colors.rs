//! Updates menu item colors based on the current selection.

use bevy::prelude::*;

use crate::{
    screen::main_menu::{MainMenuConfig, MainMenuSelection, MenuItem},
    shared::color_from_rgb,
};

/// Updates menu item colors based on the current selection.
pub(crate) fn update_menu_colors(
    config: Res<MainMenuConfig>,
    selection: Res<MainMenuSelection>,
    mut query: Query<(&MenuItem, &mut TextColor)>,
) {
    for (item, mut text_color) in &mut query {
        let color = if *item == MenuItem::Settings {
            color_from_rgb(config.disabled_color_rgb)
        } else if *item == selection.selected {
            color_from_rgb(config.selected_color_rgb)
        } else {
            color_from_rgb(config.normal_color_rgb)
        };
        *text_color = TextColor(color);
    }
}
