//! Updates the chip selection screen display — timer text and card highlights.

use bevy::prelude::*;

use crate::screen::chip_select::{
    ChipSelectConfig,
    components::{ChipCard, ChipTimerText},
    resources::{ChipSelectSelection, ChipSelectTimer},
};

/// Updates the timer display text and card border colors based on selection.
pub(crate) fn update_chip_display(
    config: Res<ChipSelectConfig>,
    timer: Res<ChipSelectTimer>,
    selection: Res<ChipSelectSelection>,
    mut timer_text: Query<&mut Text, With<ChipTimerText>>,
    mut cards: Query<(&ChipCard, &mut BorderColor)>,
) {
    // Update timer text
    for mut text in &mut timer_text {
        let display_secs = timer.remaining.ceil().max(0.0);
        **text = format!("{display_secs:.0}");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app(timer_remaining: f32, selection_index: usize) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(ChipSelectTimer {
                remaining: timer_remaining,
            })
            .insert_resource(ChipSelectSelection {
                index: selection_index,
            })
            .add_systems(Update, update_chip_display);
        app
    }

    #[test]
    fn timer_text_shows_ceiling_seconds() {
        let mut app = test_app(7.3, 0);
        let text_entity = app
            .world_mut()
            .spawn((ChipTimerText, Text::new("10"), TextColor(Color::WHITE)))
            .id();
        app.update();

        let text = app.world().get::<Text>(text_entity).unwrap();
        let s: &str = text;
        assert_eq!(s, "8", "ceil(7.3) = 8");
    }

    #[test]
    fn timer_text_clamps_at_zero() {
        let mut app = test_app(-1.0, 0);
        let text_entity = app
            .world_mut()
            .spawn((ChipTimerText, Text::new("10"), TextColor(Color::WHITE)))
            .id();
        app.update();

        let text = app.world().get::<Text>(text_entity).unwrap();
        let s: &str = text;
        assert_eq!(s, "0");
    }

    #[test]
    fn selected_card_gets_selected_border() {
        let config = ChipSelectConfig::default();
        let expected = Color::srgb(
            config.selected_color_rgb[0],
            config.selected_color_rgb[1],
            config.selected_color_rgb[2],
        );

        let mut app = test_app(10.0, 1);
        let card = app
            .world_mut()
            .spawn((ChipCard { index: 1 }, BorderColor::all(Color::BLACK)))
            .id();
        app.update();

        let border = app.world().get::<BorderColor>(card).unwrap();
        assert_eq!(*border, BorderColor::all(expected));
    }

    #[test]
    fn unselected_card_gets_normal_border() {
        let config = ChipSelectConfig::default();
        let expected = Color::srgb(
            config.normal_color_rgb[0],
            config.normal_color_rgb[1],
            config.normal_color_rgb[2],
        );

        let mut app = test_app(10.0, 0);
        let card = app
            .world_mut()
            .spawn((ChipCard { index: 1 }, BorderColor::all(Color::BLACK)))
            .id();
        app.update();

        let border = app.world().get::<BorderColor>(card).unwrap();
        assert_eq!(*border, BorderColor::all(expected));
    }
}
