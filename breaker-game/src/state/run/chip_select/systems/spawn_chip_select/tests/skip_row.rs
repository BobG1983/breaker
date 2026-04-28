use bevy::prelude::*;

use super::helpers::*;
use crate::{
    mutators::protocols::{
        definition::ProtocolKind,
        greed::{GreedConfig, GreedStacks},
        resources::{ActiveProtocols, ProtocolOffer},
    },
    state::run::chip_select::components::{ProtocolCard, SkipButton, SkipIndicator},
};

// ── B1: Skip button NOT spawned when Greed is inactive ──

#[test]
fn skip_button_not_spawned_when_greed_inactive() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed(
        make_offers(3),
        offer,
        ActiveProtocols::default(), // Greed inactive
        Some(GreedStacks { skips: 0 }),
        Some(GreedConfig {
            rarity_boost_per_skip: 5.0,
        }),
    );
    app.update();

    let button_count = app
        .world_mut()
        .query::<&SkipButton>()
        .iter(app.world())
        .count();
    assert_eq!(
        button_count, 0,
        "SkipButton must not spawn when Greed is inactive"
    );

    let indicator_count = app
        .world_mut()
        .query::<&SkipIndicator>()
        .iter(app.world())
        .count();
    assert_eq!(
        indicator_count, 0,
        "SkipIndicator must not spawn when Greed is inactive"
    );
}

#[test]
fn skip_button_not_spawned_when_greed_inactive_and_resources_absent() {
    // Edge case of B1: with `ActiveProtocols::default()` AND `GreedStacks` /
    // `GreedConfig` both absent, the spawn must not panic and counts remain 0.
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed(
        make_offers(3),
        offer,
        ActiveProtocols::default(),
        None,
        None,
    );
    app.update();

    let button_count = app
        .world_mut()
        .query::<&SkipButton>()
        .iter(app.world())
        .count();
    assert_eq!(button_count, 0);

    let indicator_count = app
        .world_mut()
        .query::<&SkipIndicator>()
        .iter(app.world())
        .count();
    assert_eq!(indicator_count, 0);
}

// ── B2: Skip button spawned when Greed is active ──

#[test]
fn skip_button_spawned_when_greed_active() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut active = ActiveProtocols::default();
    active.insert(def_for(ProtocolKind::Greed, "Greed"));
    let mut app = test_app_greed(
        make_offers(3),
        offer,
        active,
        Some(GreedStacks { skips: 0 }),
        Some(GreedConfig {
            rarity_boost_per_skip: 5.0,
        }),
    );
    app.update();

    let button_count = app
        .world_mut()
        .query::<&SkipButton>()
        .iter(app.world())
        .count();
    assert_eq!(
        button_count, 1,
        "SkipButton must spawn exactly once when Greed is active"
    );

    let indicator_count = app
        .world_mut()
        .query::<&SkipIndicator>()
        .iter(app.world())
        .count();
    assert_eq!(
        indicator_count, 1,
        "SkipIndicator must spawn exactly once when Greed is active"
    );
}

#[test]
fn skip_button_spawned_when_greed_active_without_protocol_offer() {
    // Edge case of B2: with `ProtocolOffer(None)` and Greed active, Skip row
    // still spawns (Greed activeness drives the row, not the protocol offer).
    let mut active = ActiveProtocols::default();
    active.insert(def_for(ProtocolKind::Greed, "Greed"));
    let mut app = test_app_greed(
        make_offers(3),
        ProtocolOffer(None),
        active,
        Some(GreedStacks { skips: 0 }),
        Some(GreedConfig {
            rarity_boost_per_skip: 5.0,
        }),
    );
    app.update();

    let button_count = app
        .world_mut()
        .query::<&SkipButton>()
        .iter(app.world())
        .count();
    assert_eq!(button_count, 1);

    let indicator_count = app
        .world_mut()
        .query::<&SkipIndicator>()
        .iter(app.world())
        .count();
    assert_eq!(indicator_count, 1);

    let protocol_count = app
        .world_mut()
        .query::<&ProtocolCard>()
        .iter(app.world())
        .count();
    assert_eq!(protocol_count, 0);
}

// ── B3: Skip indicator text reflects current GreedStacks.skips and computed boost percent ──

#[test]
fn skip_indicator_text_reflects_skips_and_boost_percent() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut active = ActiveProtocols::default();
    active.insert(def_for(ProtocolKind::Greed, "Greed"));
    // GreedConfig::rarity_boost_per_skip is in PERCENT units (post-activate
    // multiplication). Construct directly with 5.0, NOT via def_for which
    // uses the raw RON fraction (0.05).
    let mut app = test_app_greed(
        make_offers(3),
        offer,
        active,
        Some(GreedStacks { skips: 3 }),
        Some(GreedConfig {
            rarity_boost_per_skip: 5.0,
        }),
    );
    app.update();

    // Among Text children of the SkipIndicator, at least one carries text
    // containing both "3" (the skip count) and "15" (3 * 5.0 = 15% boost).
    let mut found = false;
    let mut texts: Vec<String> = Vec::new();
    let mut query = app
        .world_mut()
        .query_filtered::<&Text, With<SkipIndicator>>();
    for text in query.iter(app.world()) {
        let s: &str = text;
        texts.push(s.to_owned());
        if s.contains('3') && s.contains("15") {
            found = true;
        }
    }
    assert!(
        found,
        "expected SkipIndicator text to contain both '3' and '15', got texts: {texts:?}"
    );
}

#[test]
fn skip_indicator_text_zero_skips_teaches_per_skip_rate() {
    // Edge case 1 of B3: skips == 0 — surface the per-skip rate so the player
    // learns the gamble before they take it. With rarity_boost_per_skip == 5.0
    // and zero skips taken, expected text: "Skips: 0 (+5% per skip)".
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut active = ActiveProtocols::default();
    active.insert(def_for(ProtocolKind::Greed, "Greed"));
    let mut app = test_app_greed(
        make_offers(3),
        offer,
        active,
        Some(GreedStacks { skips: 0 }),
        Some(GreedConfig {
            rarity_boost_per_skip: 5.0,
        }),
    );
    app.update();

    let mut found = false;
    let mut texts: Vec<String> = Vec::new();
    let mut query = app
        .world_mut()
        .query_filtered::<&Text, With<SkipIndicator>>();
    for text in query.iter(app.world()) {
        let s: &str = text;
        texts.push(s.to_owned());
        if s.contains('0') && s.contains("+5%") && s.contains("per skip") {
            found = true;
        }
    }
    assert!(
        found,
        "expected SkipIndicator text to teach the +5% per-skip rate, got texts: {texts:?}"
    );
}

#[test]
fn skip_indicator_text_falls_back_to_plus_zero_percent_when_greed_config_absent() {
    // Edge case 2 of B3: GreedStacks { skips: 3 } present, GreedConfig absent.
    // Indicator falls back to displaying "+0%" while still reading the live
    // skip count. Expected: "Skips: 3 (+0% next)".
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut active = ActiveProtocols::default();
    active.insert(def_for(ProtocolKind::Greed, "Greed"));
    let mut app = test_app_greed(
        make_offers(3),
        offer,
        active,
        Some(GreedStacks { skips: 3 }),
        None, // GreedConfig absent
    );
    app.update();

    let mut found = false;
    let mut texts: Vec<String> = Vec::new();
    let mut query = app
        .world_mut()
        .query_filtered::<&Text, With<SkipIndicator>>();
    for text in query.iter(app.world()) {
        let s: &str = text;
        texts.push(s.to_owned());
        if s.contains('3') && s.contains("+0%") {
            found = true;
        }
    }
    assert!(
        found,
        "expected SkipIndicator text to contain both '3' and '+0%' when GreedConfig absent, got texts: {texts:?}"
    );
}

#[test]
fn skip_indicator_text_falls_back_to_zero_when_both_greed_resources_absent() {
    // Defensive case: Greed active, but BOTH GreedStacks AND GreedConfig
    // are absent (e.g., a degenerate test harness or a state-machine bug
    // that activates Greed without inserting its resources). The indicator
    // must still spawn without panic and display "Skips: 0 (+0% next)".
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut active = ActiveProtocols::default();
    active.insert(def_for(ProtocolKind::Greed, "Greed"));
    let mut app = test_app_greed(
        make_offers(3),
        offer,
        active,
        None, // GreedStacks absent
        None, // GreedConfig absent
    );
    app.update();

    let mut found = false;
    let mut texts: Vec<String> = Vec::new();
    let mut query = app
        .world_mut()
        .query_filtered::<&Text, With<SkipIndicator>>();
    for text in query.iter(app.world()) {
        let s: &str = text;
        texts.push(s.to_owned());
        if s.contains('0') && s.contains("+0%") {
            found = true;
        }
    }
    assert!(
        found,
        "expected SkipIndicator text to contain both '0' and '+0%' when both Greed resources absent, got texts: {texts:?}"
    );
}
