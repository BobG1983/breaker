//! System to spawn the hazard selection screen UI.

use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use crate::{
    mutators::hazards::{definition::HazardDefinition, resources::HazardOffers},
    shared::color_from_rgb,
    state::run::hazard_select::{
        HazardSelectConfig,
        components::{HazardCard, HazardSelectScreen, HazardTimerText},
        resources::{HazardSelectSelection, HazardSelectTimer},
    },
};

/// Spawns the hazard selection UI with cards from the pre-generated offers
/// and a countdown timer.
///
/// Reads `HazardOffers` inserted by `generate_hazard_offerings` (which runs
/// earlier in the `OnEnter(HazardSelectState::Selecting)` chain).
pub(crate) fn spawn_hazard_select(
    mut commands: Commands,
    config: Res<HazardSelectConfig>,
    offers: Res<HazardOffers>,
) {
    commands.insert_resource(HazardSelectTimer {
        remaining: config.timer_secs,
    });
    commands.insert_resource(HazardSelectSelection::default());

    commands
        .spawn((
            HazardSelectScreen,
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
            spawn_card_row(parent, &config, &offers.0);
            spawn_prompt(parent);
        });
}

fn spawn_timer_display(parent: &mut ChildSpawnerCommands<'_>, config: &HazardSelectConfig) {
    parent.spawn((
        HazardTimerText,
        Text::new(format!("{:.0}", config.timer_secs)),
        TextFont {
            font_size: config.timer_font_size,
            ..default()
        },
        TextColor(color_from_rgb(config.timer_color_rgb)),
    ));
}

fn spawn_title(parent: &mut ChildSpawnerCommands<'_>) {
    parent.spawn((
        Text::new("CHOOSE A HAZARD"),
        TextFont {
            font_size: 48.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn spawn_card_row(
    parent: &mut ChildSpawnerCommands<'_>,
    config: &HazardSelectConfig,
    offers: &[HazardDefinition],
) {
    let selected_color = color_from_rgb(config.selected_color_rgb);
    let normal_color = color_from_rgb(config.normal_color_rgb);

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(24.0),
            ..default()
        })
        .with_children(|row| {
            for (i, def) in offers.iter().enumerate() {
                let border_color = if i == 0 { selected_color } else { normal_color };

                row.spawn((
                    HazardCard { index: i },
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
                        Text::new(def.name.clone()),
                        TextFont {
                            font_size: config.card_title_font_size,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    card.spawn((
                        Text::new(def.description.clone()),
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
        Text::new("< > to select, Enter to confirm"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
    ));
}

#[cfg(test)]
mod tests;
