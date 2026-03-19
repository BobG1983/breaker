//! System to update the timer display text and color each frame.

use bevy::prelude::*;

use crate::{
    run::node::NodeTimer,
    ui::{components::NodeTimerDisplay, resources::TimerUiConfig},
};

/// Updates the timer display text and color based on remaining time.
pub(crate) fn update_timer_display(
    timer: Res<NodeTimer>,
    config: Res<TimerUiConfig>,
    mut query: Query<(&mut Text, &mut TextColor), With<NodeTimerDisplay>>,
) {
    for (mut text, mut color) in &mut query {
        let display_secs = timer.remaining.ceil().max(0.0);
        text.0 = format!("{display_secs:.0}");

        let fraction = if timer.total > 0.0 {
            timer.remaining / timer.total
        } else {
            0.0
        };
        color.0 = config.color_for_fraction(fraction);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app(remaining: f32, total: f32) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(NodeTimer { remaining, total })
            .insert_resource(TimerUiConfig::default())
            .add_systems(Update, update_timer_display);
        app
    }

    fn spawn_display(app: &mut App) -> Entity {
        app.world_mut()
            .spawn((NodeTimerDisplay, Text::new("99"), TextColor(Color::WHITE)))
            .id()
    }

    #[test]
    fn updates_text_to_remaining_ceiling() {
        let mut app = test_app(23.4, 60.0);
        let entity = spawn_display(&mut app);
        app.update();

        let text = app.world().entity(entity).get::<Text>().unwrap();
        assert_eq!(text.0, "24");
    }

    #[test]
    fn normal_color_at_high_fraction() {
        let mut app = test_app(50.0, 60.0);
        let config = TimerUiConfig::default();
        let entity = spawn_display(&mut app);
        app.update();

        let color = app.world().entity(entity).get::<TextColor>().unwrap();
        assert_eq!(color.0, config.color_for_fraction(50.0 / 60.0));
    }

    #[test]
    fn warning_color_at_mid_fraction() {
        let mut app = test_app(15.0, 60.0);
        let config = TimerUiConfig::default();
        let entity = spawn_display(&mut app);
        app.update();

        let color = app.world().entity(entity).get::<TextColor>().unwrap();
        // 15/60 = 0.25 — between urgent (0.15) and warning (0.33)
        assert_eq!(color.0, config.color_for_fraction(15.0 / 60.0));
    }

    #[test]
    fn urgent_color_at_low_fraction() {
        let mut app = test_app(5.0, 60.0);
        let config = TimerUiConfig::default();
        let entity = spawn_display(&mut app);
        app.update();

        let color = app.world().entity(entity).get::<TextColor>().unwrap();
        // 5/60 = 0.083 — below urgent (0.15)
        assert_eq!(color.0, config.color_for_fraction(5.0 / 60.0));
    }
}
