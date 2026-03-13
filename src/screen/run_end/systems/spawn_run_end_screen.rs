//! System to spawn the run-end screen UI.

use bevy::prelude::*;

use crate::{
    run::resources::{RunOutcome, RunState},
    screen::run_end::RunEndScreen,
};

/// Spawns the run-end UI showing the run outcome.
pub fn spawn_run_end_screen(mut commands: Commands, run_state: Res<RunState>) {
    let (title, subtitle) = match run_state.outcome {
        RunOutcome::Won => ("RUN COMPLETE", "All nodes cleared!"),
        RunOutcome::Lost => ("TIME'S UP", "Better luck next time."),
        RunOutcome::InProgress => ("RUN ENDED", ""),
    };

    commands
        .spawn((
            RunEndScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            if !subtitle.is_empty() {
                parent.spawn((
                    Text::new(subtitle),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                ));
            }

            parent.spawn((
                Text::new("Press Enter to continue"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app(outcome: RunOutcome) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(RunState {
            node_index: 0,
            outcome,
        });
        app.add_systems(Update, spawn_run_end_screen);
        app
    }

    #[test]
    fn spawn_creates_run_end_screen_entity() {
        let mut app = test_app(RunOutcome::Won);
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<RunEndScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn won_shows_complete_text() {
        let mut app = test_app(RunOutcome::Won);
        app.update();

        let texts: Vec<String> = app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .map(|t| t.0.clone())
            .collect();
        assert!(
            texts.iter().any(|t| t.contains("RUN COMPLETE")),
            "expected 'RUN COMPLETE' in texts: {texts:?}"
        );
    }

    #[test]
    fn lost_shows_times_up_text() {
        let mut app = test_app(RunOutcome::Lost);
        app.update();

        let texts: Vec<String> = app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .map(|t| t.0.clone())
            .collect();
        assert!(
            texts.iter().any(|t| t.contains("TIME'S UP")),
            "expected \"TIME'S UP\" in texts: {texts:?}"
        );
    }
}
