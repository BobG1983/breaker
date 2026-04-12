//! Spawns the main menu UI.

use bevy::prelude::*;

use crate::{shared::color_from_rgb, state::menu::main::*};

/// Spawns the main menu UI.
pub(crate) fn spawn_main_menu(
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
                                padding: UiRect::axes(Val::Px(43.0), Val::Px(14.0)),
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

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::state::{
        cleanup::cleanup_entities,
        types::{AppState, GameState, MenuState},
    };

    fn test_config() -> MainMenuConfig {
        MainMenuConfig {
            title_font_size:     96.0,
            menu_font_size:      36.0,
            title_color_rgb:     [2.0, 4.0, 5.0],
            selected_color_rgb:  [0.4, 3.0, 4.0],
            normal_color_rgb:    [0.6, 0.6, 0.7],
            disabled_color_rgb:  [0.25, 0.25, 0.3],
            title_bottom_margin: 48.0,
            menu_item_gap:       12.0,
            title_font_path:     "fonts/Orbitron-Bold.ttf".to_owned(),
            menu_font_path:      "fonts/Rajdhani-Medium.ttf".to_owned(),
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin, AssetPlugin::default()))
            .init_asset::<Font>()
            .insert_resource(test_config())
            .add_systems(Update, spawn_main_menu);
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
    fn cleanup_removes_entities() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin, AssetPlugin::default()))
            .init_asset::<Font>()
            .insert_resource(test_config())
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<MenuState>()
            .add_systems(OnEnter(MenuState::Main), spawn_main_menu)
            .add_systems(OnExit(MenuState::Main), cleanup_entities::<MainMenuScreen>);

        // Navigate to MenuState::Main
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Menu);
        // MenuState defaults to Loading — need one update to enter Menu, then set Main
        app.update();
        app.world_mut()
            .resource_mut::<NextState<MenuState>>()
            .set(MenuState::Main);
        app.update();

        // Verify entities exist and selection resource was inserted
        let screen_count = app
            .world_mut()
            .query_filtered::<Entity, With<MainMenuScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(screen_count, 1);
        assert!(app.world().get_resource::<MainMenuSelection>().is_some());

        // Exit MenuState::Main by navigating to a different MenuState
        app.world_mut()
            .resource_mut::<NextState<MenuState>>()
            .set(MenuState::Teardown);
        app.update();

        // Verify entities cleaned up; selection resource persists (reset on re-entry)
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
    }
}
