use bevy::prelude::*;

use super::helpers::*;
use crate::{
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        resources::ProtocolOffer,
    },
    prelude::*,
    state::run::chip_select::{
        components::{ChipCard, ChipSelectScreen, ChipTimerText, ProtocolCard},
        resources::{ChipSelectSelection, ChipSelectTimer, SelectionRow},
    },
};

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
