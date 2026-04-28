use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use crate::{
    mutators::protocols::{
        definition::ProtocolKind,
        greed::{GreedConfig, GreedStacks},
        resources::{ActiveProtocols, ProtocolOffer},
    },
    shared::color_from_rgb,
    state::run::chip_select::{
        ChipOffering, ChipSelectConfig,
        components::{
            ChipCard, ChipSelectScreen, ChipTimerText, ProtocolCard, SkipButton, SkipIndicator,
        },
        resources::{ChipOffers, ChipSelectSelection, ChipSelectTimer},
    },
};

/// Spawns the chip selection UI with cards from the pre-generated offers and a countdown timer.
///
/// Reads `ChipOffers` inserted by `generate_chip_offerings` (which runs earlier in the
/// `OnEnter(ChipSelect)` chain). Does not interact with `ChipCatalog` directly.
///
/// Conditionally spawns the Greed skip row when `ActiveProtocols` contains
/// `ProtocolKind::Greed`. `GreedStacks` and `GreedConfig` are read for the
/// indicator display; both are `Option` because `GreedConfig` is inserted
/// dynamically by `greed::activate()` (and tests may omit either).
pub(crate) fn spawn_chip_select(
    mut commands: Commands,
    config: Res<ChipSelectConfig>,
    offers: Res<ChipOffers>,
    offer: Res<ProtocolOffer>,
    active_protocols: Res<ActiveProtocols>,
    greed_stacks: Option<Res<GreedStacks>>,
    greed_config: Option<Res<GreedConfig>>,
) {
    commands.insert_resource(ChipSelectTimer {
        remaining: config.timer_secs,
    });
    commands.insert_resource(ChipSelectSelection::default());

    let greed_active = active_protocols.contains(ProtocolKind::Greed);

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
            if greed_active {
                let stacks = greed_stacks.as_deref().copied();
                let cfg = greed_config.as_deref().copied();
                let indicator = format_skip_indicator(stacks, cfg);
                spawn_skip_row(parent, &config, &indicator);
            }
            spawn_prompt(parent);
        });
}

/// Visual constants shared by every card-shaped spawn (chip, protocol, skip).
const CARD_BG: Color = Color::srgba(0.05, 0.05, 0.1, 0.9);

/// Build the bordered, button-styled `Node` shared by every card-shaped UI
/// element on the chip-select screen. Callers supply dimensions and inner
/// padding; the rest of the layout (column flex, center alignment, 12px
/// row gap, 2px border) is fixed because every card uses the same shell.
/// If a future card needs a different row gap or border, prefer adding
/// another helper over parameterizing this one — it's used by three callers
/// today and should stay simple.
fn card_node(width_px: f32, height_px: f32, padding_px: f32) -> Node {
    Node {
        width: Val::Px(width_px),
        height: Val::Px(height_px),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        padding: UiRect::all(Val::Px(padding_px)),
        row_gap: Val::Px(12.0),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    }
}

/// Compose the Greed-skip indicator text. Three cases:
/// 1. `GreedConfig` present, no skips yet → teach the rate per skip so the
///    player learns the gamble before they take it.
/// 2. `GreedConfig` present, at least one skip → show the projected boost
///    on the *next* offering.
/// 3. `GreedConfig` absent → fall back to `+0% next` (degenerate state —
///    Greed was activated but never had its config inserted; shouldn't
///    happen in production but the helper stays panic-free).
fn format_skip_indicator(stacks: Option<GreedStacks>, cfg: Option<GreedConfig>) -> String {
    let skips = stacks.map_or(0, |s| s.skips);
    match cfg {
        Some(c) if skips == 0 => {
            format!("Skips: 0 (+{:.0}% per skip)", c.rarity_boost_per_skip)
        }
        Some(c) => {
            let boost = stacks.map_or(0.0, |s| s.rarity_boost(c));
            format!("Skips: {skips} (+{boost:.0}% next)")
        }
        None => format!("Skips: {skips} (+0% next)"),
    }
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
                    card_node(200.0, 280.0, 16.0),
                    BorderColor::all(border_color),
                    BackgroundColor(CARD_BG),
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
            card_node(200.0, 280.0, 16.0),
            BorderColor::all(normal_color),
            BackgroundColor(CARD_BG),
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

fn spawn_skip_row(
    parent: &mut ChildSpawnerCommands<'_>,
    config: &ChipSelectConfig,
    indicator_text: &str,
) {
    let normal_color = color_from_rgb(config.normal_color_rgb);

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                SkipButton,
                Button,
                card_node(200.0, 64.0, 8.0),
                BorderColor::all(normal_color),
                BackgroundColor(CARD_BG),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("SKIP"),
                    TextFont {
                        font_size: config.card_title_font_size,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            row.spawn((
                SkipIndicator,
                Text::new(indicator_text),
                TextFont {
                    font_size: config.card_description_font_size,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.7, 0.3, 1.0)),
            ));
        });
}
