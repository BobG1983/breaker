//! Main menu UI — spawn, visual updates, and cleanup.

use bevy::prelude::*;

use crate::{
    screen::{
        components::{MENU_ITEMS, MainMenuScreen, MenuItem},
        resources::{MainMenuConfig, MainMenuSelection},
    },
    shared::color_from_rgb,
};

/// Spawns the main menu UI.
pub fn spawn_main_menu(
    mut commands: Commands,
    config: Res<MainMenuConfig>,
    asset_server: Res<AssetServer>,
) {
    let title_font: Handle<Font> = asset_server.load(&config.title_font_path);
    let menu_font: Handle<Font> = asset_server.load(&config.menu_font_path);

    commands.insert_resource(MainMenuSelection {
        selected: MenuItem::Play,
    });

    commands
        .spawn((
            MainMenuScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("BREAKER"),
                TextFont {
                    font: title_font,
                    font_size: config.title_font_size,
                    ..default()
                },
                TextColor(color_from_rgb(config.title_color_rgb)),
                Node {
                    margin: UiRect::bottom(Val::Px(config.title_bottom_margin)),
                    ..default()
                },
            ));

            // Menu items container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(config.menu_item_gap),
                    ..default()
                })
                .with_children(|menu| {
                    for item in &MENU_ITEMS {
                        let label = match item {
                            MenuItem::Play => "Play",
                            MenuItem::Settings => "Settings",
                            MenuItem::Quit => "Quit",
                        };

                        let color = if *item == MenuItem::Settings {
                            color_from_rgb(config.disabled_color_rgb)
                        } else if *item == MenuItem::Play {
                            color_from_rgb(config.selected_color_rgb)
                        } else {
                            color_from_rgb(config.normal_color_rgb)
                        };

                        menu.spawn((
                            *item,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(24.0), Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            Text::new(label),
                            TextFont {
                                font: menu_font.clone(),
                                font_size: config.menu_font_size,
                                ..default()
                            },
                            TextColor(color),
                        ));
                    }
                });
        });
}

/// Updates menu item colors based on the current selection.
pub fn update_menu_colors(
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

/// Removes the selection resource after main menu entity cleanup.
pub fn cleanup_main_menu(mut commands: Commands) {
    commands.remove_resource::<MainMenuSelection>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;

    fn test_config() -> MainMenuConfig {
        MainMenuConfig {
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

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin, AssetPlugin::default()));
        app.init_asset::<Font>();
        app.insert_resource(test_config());
        app.add_systems(Update, spawn_main_menu);
        app
    }

    #[test]
    fn spawn_creates_menu_entities() {
        let mut app = test_app();
        app.update();

        let screen_count = app
            .world_mut()
            .query_filtered::<Entity, With<MainMenuScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(screen_count, 1);

        let item_count = app
            .world_mut()
            .query::<&MenuItem>()
            .iter(app.world())
            .count();
        assert_eq!(item_count, 3);

        // Verify all three variants exist
        let items: Vec<MenuItem> = app
            .world_mut()
            .query::<&MenuItem>()
            .iter(app.world())
            .copied()
            .collect();
        assert!(items.contains(&MenuItem::Play));
        assert!(items.contains(&MenuItem::Settings));
        assert!(items.contains(&MenuItem::Quit));
    }

    #[test]
    fn spawn_inserts_selection_resource() {
        let mut app = test_app();
        app.update();

        let selection = app.world().resource::<MainMenuSelection>();
        assert_eq!(selection.selected, MenuItem::Play);
    }

    #[test]
    fn cleanup_removes_entities_and_resource() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin, AssetPlugin::default()));
        app.init_asset::<Font>();
        app.insert_resource(test_config());
        app.init_state::<crate::shared::GameState>();
        app.add_systems(OnEnter(crate::shared::GameState::MainMenu), spawn_main_menu);
        app.add_systems(
            OnExit(crate::shared::GameState::MainMenu),
            (
                super::super::cleanup_entities::<MainMenuScreen>,
                cleanup_main_menu,
            ),
        );

        // Enter MainMenu state
        app.world_mut()
            .resource_mut::<NextState<crate::shared::GameState>>()
            .set(crate::shared::GameState::MainMenu);
        app.update();

        // Verify entities exist
        let screen_count = app
            .world_mut()
            .query_filtered::<Entity, With<MainMenuScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(screen_count, 1);
        assert!(app.world().get_resource::<MainMenuSelection>().is_some());

        // Exit MainMenu state
        app.world_mut()
            .resource_mut::<NextState<crate::shared::GameState>>()
            .set(crate::shared::GameState::Loading);
        app.update();

        // Verify cleanup ran
        let screen_count = app
            .world_mut()
            .query_filtered::<Entity, With<MainMenuScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(screen_count, 0);

        let item_count = app
            .world_mut()
            .query::<&MenuItem>()
            .iter(app.world())
            .count();
        assert_eq!(item_count, 0);

        assert!(app.world().get_resource::<MainMenuSelection>().is_none());
    }
}
