//! System to spawn the pause menu overlay UI.

use bevy::prelude::*;

use crate::screen::pause_menu::{
    components::{PAUSE_MENU_ITEMS, PauseMenuItem, PauseMenuScreen},
    resources::PauseMenuSelection,
};

/// Spawns the pause menu overlay.
pub fn spawn_pause_menu(mut commands: Commands) {
    commands.insert_resource(PauseMenuSelection {
        selected: PauseMenuItem::Resume,
    });

    commands
        .spawn((
            PauseMenuScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(32.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            // Render above gameplay UI
            GlobalZIndex(10),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                },
            ));

            // Menu items
            for item in &PAUSE_MENU_ITEMS {
                let label = match item {
                    PauseMenuItem::Resume => "Resume",
                    PauseMenuItem::Quit => "Quit to Menu",
                };

                let color = if *item == PauseMenuItem::Resume {
                    Color::srgb(0.4, 0.8, 1.0)
                } else {
                    Color::srgb(0.6, 0.6, 0.7)
                };

                parent.spawn((
                    *item,
                    Text::new(label),
                    TextFont {
                        font_size: 36.0,
                        ..default()
                    },
                    TextColor(color),
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, spawn_pause_menu);
        app
    }

    #[test]
    fn spawn_creates_pause_screen_entity() {
        let mut app = test_app();
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<PauseMenuScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn spawn_creates_menu_items() {
        let mut app = test_app();
        app.update();

        let items: Vec<PauseMenuItem> = app
            .world_mut()
            .query::<&PauseMenuItem>()
            .iter(app.world())
            .copied()
            .collect();
        assert_eq!(items.len(), 2);
        assert!(items.contains(&PauseMenuItem::Resume));
        assert!(items.contains(&PauseMenuItem::Quit));
    }

    #[test]
    fn spawn_inserts_selection_resource() {
        let mut app = test_app();
        app.update();

        let selection = app.world().resource::<PauseMenuSelection>();
        assert_eq!(selection.selected, PauseMenuItem::Resume);
    }
}
