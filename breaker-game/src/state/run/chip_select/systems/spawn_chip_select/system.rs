use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use crate::{
    mutators::protocols::resources::ProtocolOffer,
    shared::color_from_rgb,
    state::run::chip_select::{
        ChipOffering, ChipSelectConfig,
        components::{ChipCard, ChipSelectScreen, ChipTimerText, ProtocolCard},
        resources::{ChipOffers, ChipSelectSelection, ChipSelectTimer},
    },
};

/// Spawns the chip selection UI with cards from the pre-generated offers and a countdown timer.
///
/// Reads `ChipOffers` inserted by `generate_chip_offerings` (which runs earlier in the
/// `OnEnter(ChipSelect)` chain). Does not interact with `ChipCatalog` directly.
pub(crate) fn spawn_chip_select(
    mut commands: Commands,
    config: Res<ChipSelectConfig>,
    offers: Res<ChipOffers>,
    offer: Res<ProtocolOffer>,
) {
    commands.insert_resource(ChipSelectTimer {
        remaining: config.timer_secs,
    });
    commands.insert_resource(ChipSelectSelection::default());

    commands
        .spawn((
            ChipSelectScreen,
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
            spawn_protocol_row(parent, &config, &offer);
            spawn_prompt(parent);
        });
}

fn spawn_timer_display(parent: &mut ChildSpawnerCommands<'_>, config: &ChipSelectConfig) {
    parent.spawn((
        ChipTimerText,
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
        Text::new("CHOOSE A CHIP"),
        TextFont {
            font_size: 48.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn spawn_card_row(
    parent: &mut ChildSpawnerCommands<'_>,
    config: &ChipSelectConfig,
    offers: &[ChipOffering],
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
            for (i, offering) in offers.iter().enumerate() {
                let border_color = if i == 0 { selected_color } else { normal_color };
                let def = offering.definition();

                row.spawn((
                    ChipCard { index: i },
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
                        Text::new(def.rarity.to_string()),
                        TextFont {
                            font_size: config.card_description_font_size,
                            ..default()
                        },
                        TextColor(Color::srgba(0.8, 0.7, 0.3, 1.0)),
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

fn spawn_protocol_row(
    parent: &mut ChildSpawnerCommands<'_>,
    config: &ChipSelectConfig,
    offer: &ProtocolOffer,
) {
    let Some(def) = offer.0.as_ref() else {
        return;
    };

    let normal_color = color_from_rgb(config.normal_color_rgb);

    parent
        .spawn((
            ProtocolCard,
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
            BorderColor::all(normal_color),
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
                Text::new("PROTOCOL"),
                TextFont {
                    font_size: config.card_description_font_size,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.3, 0.9, 1.0)),
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
