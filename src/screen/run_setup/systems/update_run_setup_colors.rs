//! Updates breaker card colors based on the current selection.

use bevy::prelude::*;

use crate::screen::run_setup::{components::BreakerCard, resources::RunSetupSelection};

const SELECTED_COLOR: Color = Color::srgb(0.4, 0.8, 1.0);
const NORMAL_COLOR: Color = Color::srgb(0.6, 0.6, 0.7);

/// Updates breaker card text colors based on the current selection index.
pub fn update_run_setup_colors(
    selection: Res<RunSetupSelection>,
    cards: Query<(Entity, &BreakerCard)>,
    children_query: Query<&Children>,
    mut text_colors: Query<&mut TextColor>,
) {
    // Sort cards by archetype name to match selection index
    let mut sorted_cards: Vec<(Entity, &BreakerCard)> = cards.iter().collect();
    sorted_cards.sort_by(|a, b| a.1.archetype_name.cmp(&b.1.archetype_name));

    for (i, (card_entity, _)) in sorted_cards.iter().enumerate() {
        let color = if i == selection.index {
            SELECTED_COLOR
        } else {
            NORMAL_COLOR
        };

        // Update text colors on card's children
        if let Ok(children) = children_query.get(*card_entity) {
            for child in children.iter() {
                if let Ok(mut text_color) = text_colors.get_mut(child) {
                    *text_color = TextColor(color);
                }
            }
        }
    }
}
