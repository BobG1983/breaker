//! Spawns the loading screen UI.

use bevy::prelude::*;

use crate::state::app::loading::components::{LoadingBarFill, LoadingProgressText, LoadingScreen};

/// Width of the loading bar background in pixels.
const LOADING_BAR_WIDTH: f32 = 720.0;

/// Height of the loading bar in pixels.
const LOADING_BAR_HEIGHT: f32 = 43.0;

/// Spawns the loading screen UI.
pub(crate) fn spawn_loading_screen(mut commands: Commands) {
    commands
        .spawn((
            LoadingScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(29.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Progress text
            parent.spawn((
                LoadingProgressText,
                Text::new("Loading..."),
                TextFont {
                    font_size: 43.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Bar background
            parent
                .spawn(Node {
                    width: Val::Px(LOADING_BAR_WIDTH),
                    height: Val::Px(LOADING_BAR_HEIGHT),
                    ..default()
                })
                .insert(BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.15)))
                .with_children(|bar_bg| {
                    // Bar fill
                    bar_bg.spawn((
                        LoadingBarFill,
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.8, 1.0)),
                    ));
                });
        });
}
