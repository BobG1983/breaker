//! System to spawn the upgrade selection screen UI.

use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use crate::screen::upgrade_select::{
    UpgradeSelectConfig,
    components::{UpgradeCard, UpgradeSelectScreen, UpgradeTimerText},
    resources::{UpgradeSelectSelection, UpgradeSelectTimer},
};

/// Spawns the upgrade selection UI with 3 placeholder cards and a countdown timer.
pub fn spawn_upgrade_select(mut commands: Commands, config: Res<UpgradeSelectConfig>) {
    commands.insert_resource(UpgradeSelectTimer {
        remaining: config.timer_secs,
    });
    commands.insert_resource(UpgradeSelectSelection { index: 0 });

    commands
        .spawn((
            UpgradeSelectScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(32.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            spawn_timer_display(parent, &config);
            spawn_title(parent);
            spawn_card_row(parent, &config);
            spawn_prompt(parent);
        });
}

fn spawn_timer_display(parent: &mut ChildSpawnerCommands<'_>, config: &UpgradeSelectConfig) {
    parent.spawn((
        UpgradeTimerText,
        Text::new(format!("{:.0}", config.timer_secs)),
        TextFont {
            font_size: config.timer_font_size,
            ..default()
        },
        TextColor(Color::srgb(
            config.timer_color_rgb[0],
            config.timer_color_rgb[1],
            config.timer_color_rgb[2],
        )),
    ));
}

fn spawn_title(parent: &mut ChildSpawnerCommands<'_>) {
    parent.spawn((
        // TODO(phase-7): replace with Amp/Augment/Overclock category
        Text::new("CHOOSE A POWER-UP"),
        TextFont {
            font_size: 48.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn spawn_card_row(parent: &mut ChildSpawnerCommands<'_>, config: &UpgradeSelectConfig) {
    // TODO(phase-7): replace with real Amp names from data
    let card_titles = ["AMP A", "AMP B", "AMP C"];

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

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(24.0),
            ..default()
        })
        .with_children(|row| {
            for (i, title) in card_titles.iter().enumerate() {
                let border_color = if i == 0 { selected_color } else { normal_color };

                row.spawn((
                    UpgradeCard { index: i },
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(280.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(12.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(border_color),
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.9)),
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new(*title),
                        TextFont {
                            font_size: config.card_title_font_size,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    card.spawn((
                        Text::new("Placeholder Amp effect"),
                        TextFont {
                            font_size: config.card_description_font_size,
                            ..default()
                        },
                        TextColor(Color::srgba(0.6, 0.6, 0.7, 1.0)),
                    ));
                });
            }
        });
}

fn spawn_prompt(parent: &mut ChildSpawnerCommands<'_>) {
    parent.spawn((
        Text::new("Left/Right to select, Enter to confirm"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(UpgradeSelectConfig::default());
        app.add_systems(Update, spawn_upgrade_select);
        app
    }

    #[test]
    fn spawn_creates_screen_entity() {
        let mut app = test_app();
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<UpgradeSelectScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn spawn_creates_three_cards() {
        let mut app = test_app();
        app.update();

        let count = app
            .world_mut()
            .query::<&UpgradeCard>()
            .iter(app.world())
            .count();
        assert_eq!(count, 3);
    }

    #[test]
    fn spawn_inserts_timer_resource() {
        let mut app = test_app();
        app.update();

        let timer = app.world().resource::<UpgradeSelectTimer>();
        assert!((timer.remaining - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn spawn_inserts_selection_resource() {
        let mut app = test_app();
        app.update();

        let selection = app.world().resource::<UpgradeSelectSelection>();
        assert_eq!(selection.index, 0);
    }

    #[test]
    fn spawn_creates_timer_text() {
        let mut app = test_app();
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<UpgradeTimerText>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }
}
