//! Updates the upgrade selection screen display — timer text and card highlights.

use bevy::prelude::*;

use crate::screen::upgrade_select::{
    UpgradeSelectConfig,
    components::{UpgradeCard, UpgradeTimerText},
    resources::{UpgradeSelectSelection, UpgradeSelectTimer},
};

/// Updates the timer display text and card border colors based on selection.
pub fn update_upgrade_display(
    config: Res<UpgradeSelectConfig>,
    timer: Res<UpgradeSelectTimer>,
    selection: Res<UpgradeSelectSelection>,
    mut timer_text: Query<&mut Text, With<UpgradeTimerText>>,
    mut cards: Query<(&UpgradeCard, &mut BorderColor)>,
) {
    // Update timer text
    for mut text in &mut timer_text {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let display_secs = timer.remaining.ceil().max(0.0) as u32;
        **text = format!("{display_secs}");
    }

    // Update card border colors
    let selected_color = Color::srgb(
        config.selected_color_rgb[0],
        config.selected_color_rgb[1],
        config.selected_color_rgb[2],
    );
    let normal_color = Color::srgb(
        config.normal_color_rgb[0],
        config.normal_color_rgb[1],
        config.normal_color_rgb[2],
    );

    for (card, mut border) in &mut cards {
        *border = if card.index == selection.index {
            BorderColor::all(selected_color)
        } else {
            BorderColor::all(normal_color)
        };
    }
}
