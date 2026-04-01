//! System to spawn the chip selection screen UI.

use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use crate::screen::chip_select::{
    ChipOffering, ChipSelectConfig, color_from_rgb,
    components::{ChipCard, ChipSelectScreen, ChipTimerText},
    resources::{ChipOffers, ChipSelectSelection, ChipSelectTimer},
};

/// Spawns the chip selection UI with cards from the pre-generated offers and a countdown timer.
///
/// Reads `ChipOffers` inserted by `generate_chip_offerings` (which runs earlier in the
/// `OnEnter(ChipSelect)` chain). Does not interact with `ChipCatalog` directly.
pub(crate) fn spawn_chip_select(
    mut commands: Commands,
    config: Res<ChipSelectConfig>,
    offers: Res<ChipOffers>,
) {
    let offer_list = offers.0.clone();

    commands.insert_resource(ChipSelectTimer {
        remaining: config.timer_secs,
    });
    commands.insert_resource(ChipSelectSelection { index: 0 });

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
            spawn_card_row(parent, &config, &offer_list);
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
mod tests {
    use super::*;
    use crate::chips::ChipDefinition;

    fn make_offers(count: usize) -> ChipOffers {
        let all = vec![
            ChipOffering::Normal(ChipDefinition::test_simple("Piercing Shot")),
            ChipOffering::Normal(ChipDefinition::test_simple("Wide Breaker")),
            ChipOffering::Normal(ChipDefinition::test_simple("Surge")),
            ChipOffering::Normal(ChipDefinition::test_simple("Ricochet")),
            ChipOffering::Normal(ChipDefinition::test_simple("Quick Dash")),
        ];
        ChipOffers(all.into_iter().take(count).collect())
    }

    fn test_app_with_offers(offers: ChipOffers) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(offers)
            .add_systems(Update, spawn_chip_select);
        app
    }

    #[test]
    fn spawn_creates_screen_entity() {
        let mut app = test_app_with_offers(make_offers(3));
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<ChipSelectScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn spawn_creates_three_cards_from_offers() {
        let mut app = test_app_with_offers(make_offers(3));
        app.update();

        let count = app
            .world_mut()
            .query::<&ChipCard>()
            .iter(app.world())
            .count();
        assert_eq!(count, 3);
    }

    #[test]
    fn spawn_creates_cards_matching_offers_size() {
        let mut app = test_app_with_offers(make_offers(2));
        app.update();

        let count = app
            .world_mut()
            .query::<&ChipCard>()
            .iter(app.world())
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn empty_offers_creates_no_cards() {
        let mut app = test_app_with_offers(make_offers(0));
        app.update();

        let count = app
            .world_mut()
            .query::<&ChipCard>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn spawn_inserts_timer_resource() {
        let mut app = test_app_with_offers(make_offers(3));
        app.update();

        let timer = app.world().resource::<ChipSelectTimer>();
        assert!((timer.remaining - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn spawn_inserts_selection_resource() {
        let mut app = test_app_with_offers(make_offers(3));
        app.update();

        let selection = app.world().resource::<ChipSelectSelection>();
        assert_eq!(selection.index, 0);
    }

    #[test]
    fn spawn_reads_existing_offers_resource() {
        let mut app = test_app_with_offers(make_offers(3));
        app.update();

        let offers = app.world().resource::<ChipOffers>();
        assert_eq!(offers.0.len(), 3);
        assert_eq!(offers.0[0].name(), "Piercing Shot");
        assert_eq!(offers.0[1].name(), "Wide Breaker");
        assert_eq!(offers.0[2].name(), "Surge");
    }

    #[test]
    fn spawn_creates_timer_text() {
        let mut app = test_app_with_offers(make_offers(3));
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<ChipTimerText>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn cards_display_real_chip_names() {
        let mut app = test_app_with_offers(make_offers(3));
        app.update();

        let mut found_names: Vec<String> = Vec::new();
        for text in app.world_mut().query::<&Text>().iter(app.world()) {
            let s: &str = text;
            if s == "Piercing Shot" || s == "Wide Breaker" || s == "Surge" {
                found_names.push(s.to_owned());
            }
        }
        assert_eq!(found_names.len(), 3);
    }

    #[test]
    fn empty_offers_still_creates_screen() {
        let mut app = test_app_with_offers(make_offers(0));
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<ChipSelectScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn offers_with_five_spawns_all_five_cards() {
        let mut app = test_app_with_offers(make_offers(5));
        app.update();

        let count = app
            .world_mut()
            .query::<&ChipCard>()
            .iter(app.world())
            .count();
        assert_eq!(count, 5, "should spawn a card for each offer");
    }
}
