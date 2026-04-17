//! System to spawn the chip selection screen UI.

use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use crate::{
    protocol::resources::ProtocolOffer,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chips::ChipDefinition,
        prelude::*,
        protocol::{
            definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
            resources::ProtocolOffer,
        },
        state::run::chip_select::{components::ProtocolCard, resources::SelectionRow},
    };

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

    /// Local `def_for` helper — mirrors `protocol/resources.rs::tests::def_for`.
    fn def_for(kind: ProtocolKind, name: &str) -> ProtocolDefinition {
        let tuning = match kind {
            ProtocolKind::Deadline => ProtocolTuning::Deadline { effects: vec![] },
            ProtocolKind::Ricochet => ProtocolTuning::Ricochet { effects: vec![] },
            ProtocolKind::Anchor => ProtocolTuning::Anchor { effects: vec![] },
            ProtocolKind::Kickstart => ProtocolTuning::Kickstart { effects: vec![] },
            ProtocolKind::DebtCollector => ProtocolTuning::DebtCollector {
                stack_per_bump: 0.1,
            },
            ProtocolKind::IronCurtain => ProtocolTuning::IronCurtain {
                damage_fraction: 0.25,
                falloff_start:   0.5,
            },
            ProtocolKind::EchoStrike => ProtocolTuning::EchoStrike {
                max_echoes:      3,
                newest_fraction: 0.5,
                middle_fraction: 0.25,
                oldest_fraction: 0.125,
            },
            ProtocolKind::Siphon => ProtocolTuning::Siphon {
                streak_window: 2.0,
                time_per_kill: 0.25,
            },
            ProtocolKind::Greed => ProtocolTuning::Greed {
                rarity_boost_per_skip: 0.05,
            },
            ProtocolKind::RecklessDash => ProtocolTuning::RecklessDash {
                risky_zone_start:  0.3,
                damage_multiplier: 4.0,
                double_penalty:    true,
            },
            ProtocolKind::Burnout => ProtocolTuning::Burnout {
                fill_duration:               3.0,
                drain_duration:              5.0,
                still_threshold:             0.25,
                full_heat_damage_multiplier: 2.0,
                speed_boost_duration:        1.0,
            },
            ProtocolKind::Conductor => ProtocolTuning::Conductor {
                primary_swap_window: 0.2,
            },
            ProtocolKind::Afterimage => ProtocolTuning::Afterimage {
                phantom_duration:      1.5,
                phantom_bolt_duration: 0.75,
            },
            ProtocolKind::Fission => ProtocolTuning::Fission {
                kills_per_split: 10,
            },
            ProtocolKind::TierRegression => ProtocolTuning::TierRegression { tiers_back: 1 },
        };
        ProtocolDefinition {
            name: name.to_string(),
            description: String::new(),
            unlock_tier: 0,
            tuning,
        }
    }

    fn test_app_with_offers(offers: ChipOffers) -> App {
        TestAppBuilder::new()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(offers)
            .with_resource::<ProtocolOffer>()
            .with_system(Update, spawn_chip_select)
            .build()
    }

    /// Like `test_app_with_offers` but also inserts a `ProtocolOffer` value.
    fn test_app_with_offers_and_protocol_offer(
        offers: ChipOffers,
        protocol_offer: ProtocolOffer,
    ) -> App {
        let mut app = test_app_with_offers(offers);
        app.insert_resource(protocol_offer);
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
        assert_eq!(selection.chip_index, 0);
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

    // ─────────────────────────────────────────────────────────────────
    // ProtocolCard rendering tests.
    // ─────────────────────────────────────────────────────────────────

    // ── C.1: ProtocolOffer::Some spawns exactly one ProtocolCard entity ──

    #[test]
    fn protocol_offer_some_spawns_exactly_one_protocol_card() {
        let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
        let mut app = test_app_with_offers_and_protocol_offer(make_offers(3), offer);
        app.update();

        let count = app
            .world_mut()
            .query::<&ProtocolCard>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 1,
            "ProtocolOffer::Some must spawn exactly one ProtocolCard entity"
        );
    }

    #[test]
    fn protocol_offer_some_with_empty_chip_offers_still_spawns_protocol_card() {
        // Edge case of C.1: no chip offers but a protocol offer — ProtocolCard
        // still spawns, chip-card count is 0.
        let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
        let mut app = test_app_with_offers_and_protocol_offer(make_offers(0), offer);
        app.update();

        let protocol_count = app
            .world_mut()
            .query::<&ProtocolCard>()
            .iter(app.world())
            .count();
        assert_eq!(protocol_count, 1, "ProtocolCard should still spawn");

        let chip_count = app
            .world_mut()
            .query::<&ChipCard>()
            .iter(app.world())
            .count();
        assert_eq!(chip_count, 0, "chip card count should be 0");
    }

    // ── C.2: ProtocolOffer(None) spawns zero ProtocolCard entities ──

    #[test]
    fn protocol_offer_none_spawns_zero_protocol_cards() {
        let mut app = test_app_with_offers_and_protocol_offer(make_offers(3), ProtocolOffer(None));
        app.update();

        let count = app
            .world_mut()
            .query::<&ProtocolCard>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 0,
            "ProtocolOffer(None) must not spawn any ProtocolCard"
        );
    }

    #[test]
    fn protocol_offer_none_with_empty_chip_offers_still_spawns_screen() {
        // Edge case of C.2: no cards, no protocol card, but screen still exists.
        let mut app = test_app_with_offers_and_protocol_offer(make_offers(0), ProtocolOffer(None));
        app.update();

        let protocol_count = app
            .world_mut()
            .query::<&ProtocolCard>()
            .iter(app.world())
            .count();
        assert_eq!(protocol_count, 0);

        let chip_count = app
            .world_mut()
            .query::<&ChipCard>()
            .iter(app.world())
            .count();
        assert_eq!(chip_count, 0);

        let screen_count = app
            .world_mut()
            .query_filtered::<Entity, With<ChipSelectScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(screen_count, 1, "ChipSelectScreen must still exist");
    }

    // ── C.3: ProtocolCard carries name + description text children ──

    #[test]
    fn protocol_card_renders_name_and_description_in_text_children() {
        let offer = ProtocolOffer(Some(ProtocolDefinition {
            name:        "Greed".to_string(),
            description: "Skip chips to boost rarity".to_string(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Greed {
                rarity_boost_per_skip: 0.05,
            },
        }));
        let mut app = test_app_with_offers_and_protocol_offer(make_offers(3), offer);
        app.update();

        let mut saw_name = false;
        let mut saw_description = false;
        for text in app.world_mut().query::<&Text>().iter(app.world()) {
            let s: &str = text;
            if s == "Greed" {
                saw_name = true;
            }
            if s == "Skip chips to boost rarity" {
                saw_description = true;
            }
        }
        assert!(saw_name, "expected a Text child with content \"Greed\"");
        assert!(
            saw_description,
            "expected a Text child with content \"Skip chips to boost rarity\""
        );
    }

    #[test]
    fn protocol_card_with_empty_description_still_renders_name() {
        // Edge case of C.3: empty description string — name must still appear.
        let offer = ProtocolOffer(Some(ProtocolDefinition {
            name:        "Greed".to_string(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Greed {
                rarity_boost_per_skip: 0.05,
            },
        }));
        let mut app = test_app_with_offers_and_protocol_offer(make_offers(3), offer);
        app.update();

        let mut saw_name = false;
        for text in app.world_mut().query::<&Text>().iter(app.world()) {
            let s: &str = text;
            if s == "Greed" {
                saw_name = true;
            }
        }
        assert!(
            saw_name,
            "name must still render even when description is empty"
        );
    }

    // ── C.4: Existing chip-select invariants remain intact ──

    #[test]
    fn existing_chip_select_invariants_remain_intact_with_protocol_offer() {
        let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
        let mut app = test_app_with_offers_and_protocol_offer(make_offers(3), offer);
        app.update();

        let screen_count = app
            .world_mut()
            .query_filtered::<Entity, With<ChipSelectScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(screen_count, 1);

        let timer_text_count = app
            .world_mut()
            .query_filtered::<Entity, With<ChipTimerText>>()
            .iter(app.world())
            .count();
        assert_eq!(timer_text_count, 1);

        let chip_count = app
            .world_mut()
            .query::<&ChipCard>()
            .iter(app.world())
            .count();
        assert_eq!(chip_count, 3);

        let timer = app.world().resource::<ChipSelectTimer>();
        assert!(
            (timer.remaining - 10.0).abs() < f32::EPSILON,
            "ChipSelectTimer.remaining should be the default 10.0s"
        );

        let selection = app.world().resource::<ChipSelectSelection>();
        assert_eq!(selection.chip_index, 0);
        assert_eq!(selection.row, SelectionRow::Chip);
    }

    #[test]
    fn existing_invariants_hold_with_empty_offers_and_none_protocol() {
        // Edge case of C.4: no chip offers, no protocol — screen still exists.
        let mut app = test_app_with_offers_and_protocol_offer(make_offers(0), ProtocolOffer(None));
        app.update();

        let screen_count = app
            .world_mut()
            .query_filtered::<Entity, With<ChipSelectScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(screen_count, 1);

        let chip_count = app
            .world_mut()
            .query::<&ChipCard>()
            .iter(app.world())
            .count();
        assert_eq!(chip_count, 0);

        let protocol_count = app
            .world_mut()
            .query::<&ProtocolCard>()
            .iter(app.world())
            .count();
        assert_eq!(protocol_count, 0);
    }

    // ── C.5: ProtocolCard is a unit marker component with Component derive ──

    #[test]
    fn protocol_card_component_derive_and_visibility_smoke_test() {
        // Spawn an entity with ProtocolCard directly — query must find it.
        let mut app = TestAppBuilder::new().build();
        let entity = app.world_mut().spawn(ProtocolCard).id();

        let found = app
            .world_mut()
            .query::<&ProtocolCard>()
            .iter(app.world())
            .count();
        assert!(
            found >= 1,
            "Query<&ProtocolCard> should yield the spawned ProtocolCard entity"
        );

        // Secondary check: the entity has the marker by direct lookup.
        assert!(
            app.world().get::<ProtocolCard>(entity).is_some(),
            "entity should carry the ProtocolCard marker"
        );
    }
}
