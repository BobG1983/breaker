use bevy::prelude::*;

use super::helpers::*;
use crate::state::run::chip_select::{
    components::{ChipCard, ChipSelectScreen, ChipTimerText},
    resources::{ChipOffers, ChipSelectSelection, ChipSelectTimer},
};

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
