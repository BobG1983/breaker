//! Updates the hazard selection screen display — timer text and card border highlights.
//!
//! Without this system the timer displayed on-screen is frozen at the
//! spawn-time value even though `HazardSelectTimer.remaining` ticks down
//! every frame (see `tick_hazard_timer`). Parallel to the chip-select
//! screen's `update_chip_display`.

use bevy::prelude::*;

use crate::{
    shared::color_from_rgb,
    state::run::hazard_select::{
        HazardSelectConfig,
        components::{HazardCard, HazardTimerText},
        resources::{HazardSelectSelection, HazardSelectTimer},
    },
};

/// Updates the hazard-select timer text and card border colors each frame
/// while `HazardSelectState::Selecting`.
pub(crate) fn update_hazard_display(
    config: Res<HazardSelectConfig>,
    timer: Res<HazardSelectTimer>,
    selection: Res<HazardSelectSelection>,
    mut timer_text: Query<&mut Text, With<HazardTimerText>>,
    mut cards: Query<(&HazardCard, &mut BorderColor)>,
) {
    for mut text in &mut timer_text {
        let display_secs = timer.remaining.ceil().max(0.0);
        **text = format!("{display_secs:.0}");
    }

    let selected_color = color_from_rgb(config.selected_color_rgb);
    let normal_color = color_from_rgb(config.normal_color_rgb);

    for (card, mut border) in &mut cards {
        *border = if card.index == selection.card_index {
            BorderColor::all(selected_color)
        } else {
            BorderColor::all(normal_color)
        };
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::{
        prelude::*,
        state::run::hazard_select::components::{HazardCard, HazardSelectScreen, HazardTimerText},
    };

    fn test_app(timer_remaining: f32, selection_card: usize) -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .insert_resource(HazardSelectConfig::default())
            .insert_resource(HazardSelectTimer {
                remaining: timer_remaining,
            })
            .insert_resource(HazardSelectSelection {
                card_index: selection_card,
            })
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                16,
            )))
            .with_system(Update, update_hazard_display)
            .build()
    }

    #[test]
    fn timer_text_updates_from_resource_value() {
        let mut app = test_app(7.5, 0);
        let text_entity = app
            .world_mut()
            .spawn((HazardTimerText, Text::new("???")))
            .id();

        app.update();

        let text = app
            .world()
            .get::<Text>(text_entity)
            .expect("Text component should exist on HazardTimerText entity");
        assert_eq!(
            **text, "8",
            "timer should display ceil of 7.5 = 8 (mirrors chip-select formatting)"
        );
    }

    #[test]
    fn timer_text_clamps_to_zero_when_negative() {
        let mut app = test_app(-0.3, 0);
        let text_entity = app
            .world_mut()
            .spawn((HazardTimerText, Text::new("???")))
            .id();

        app.update();

        let text = app.world().get::<Text>(text_entity).unwrap();
        assert_eq!(
            **text, "0",
            "negative remaining should display as 0, never negative"
        );
    }

    #[test]
    fn timer_text_does_not_update_when_component_absent() {
        // No HazardTimerText entity — the system must early-out harmlessly.
        let mut app = test_app(5.0, 0);
        app.update();
        // Pass condition: no panic.
    }

    #[test]
    fn selected_card_gets_selected_border_color() {
        let mut app = test_app(5.0, 1);
        let card_0 = app
            .world_mut()
            .spawn((HazardCard { index: 0 }, BorderColor::all(Color::BLACK)))
            .id();
        let card_1 = app
            .world_mut()
            .spawn((HazardCard { index: 1 }, BorderColor::all(Color::BLACK)))
            .id();
        let card_2 = app
            .world_mut()
            .spawn((HazardCard { index: 2 }, BorderColor::all(Color::BLACK)))
            .id();

        app.update();

        let config = app.world().resource::<HazardSelectConfig>().clone();
        let selected = color_from_rgb(config.selected_color_rgb);
        let normal = color_from_rgb(config.normal_color_rgb);

        assert_eq!(
            *app.world().get::<BorderColor>(card_0).unwrap(),
            BorderColor::all(normal)
        );
        assert_eq!(
            *app.world().get::<BorderColor>(card_1).unwrap(),
            BorderColor::all(selected)
        );
        assert_eq!(
            *app.world().get::<BorderColor>(card_2).unwrap(),
            BorderColor::all(normal)
        );
    }

    /// Regression pin: the system runs every Update, so re-ticking with a
    /// mutated timer value updates the text — it is not a one-shot.
    #[test]
    fn timer_text_retracks_resource_across_updates() {
        let mut app = test_app(10.0, 0);
        let text_entity = app
            .world_mut()
            .spawn((HazardTimerText, Text::new("???")))
            .id();
        let _screen = app.world_mut().spawn(HazardSelectScreen).id();

        app.update();
        assert_eq!(**app.world().get::<Text>(text_entity).unwrap(), "10");

        app.world_mut()
            .resource_mut::<HazardSelectTimer>()
            .remaining = 3.2;
        app.update();
        assert_eq!(
            **app.world().get::<Text>(text_entity).unwrap(),
            "4",
            "timer text must track resource updates, not stay at initial value"
        );
    }
}
