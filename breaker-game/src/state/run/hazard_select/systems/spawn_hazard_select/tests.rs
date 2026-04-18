//! Tests for `spawn_hazard_select` — hazard selection screen UI spawn.

use std::collections::HashSet;

use bevy::prelude::*;

use super::*;
use crate::{
    hazard::{
        definition::{HazardDefinition, HazardKind, HazardTuning},
        resources::HazardOffers,
    },
    prelude::*,
    shared::color_from_rgb,
    state::run::hazard_select::{
        HazardSelectConfig,
        components::{HazardCard, HazardSelectScreen, HazardTimerText},
        resources::{HazardSelectSelection, HazardSelectTimer},
    },
};

fn tuning_for_kind(kind: HazardKind) -> HazardTuning {
    match kind {
        HazardKind::Decay => HazardTuning::Decay {
            base_percent:      0.05,
            per_level_percent: 0.03,
        },
        HazardKind::Drift => HazardTuning::Drift {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        },
        HazardKind::Haste => HazardTuning::Haste {
            base_percent:      0.1,
            per_level_percent: 0.05,
        },
        HazardKind::EchoCells => HazardTuning::EchoCells {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        },
        HazardKind::Erosion => HazardTuning::Erosion {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  0.5,
        },
        HazardKind::Cascade => HazardTuning::Cascade {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
        HazardKind::Fracture => HazardTuning::Fracture {
            base_splits:      1,
            per_level_splits: 1,
        },
        HazardKind::Renewal => HazardTuning::Renewal {
            base_period_secs:         10.0,
            per_level_reduction_frac: 0.2,
        },
        HazardKind::Volatility => HazardTuning::Volatility {
            hp_per_interval: 1.0,
            interval_secs:   5.0,
            max_multiplier:  2.0,
        },
        HazardKind::GravitySurge => HazardTuning::GravitySurge {
            base_duration_secs:      2.0,
            per_level_duration_secs: 1.0,
            base_strength:           100.0,
            per_level_strength_frac: 0.2,
        },
        HazardKind::Overcharge => HazardTuning::Overcharge {
            base_frac:      0.05,
            per_level_frac: 0.03,
        },
        HazardKind::Resonance => HazardTuning::Resonance {
            kills_to_trigger:      2,
            base_window:           0.5,
            window_per_level:      0.3,
            wave_speed:            200.0,
            base_slow_duration:    1.5,
            base_slow_strength:    0.5,
            slow_duration_scaling: 0.2,
            slow_strength_scaling: 0.15,
            contact_threshold:     16.0,
            wave_max_lifetime:     10.0,
        },
        HazardKind::Diffusion => HazardTuning::Diffusion {
            base_share_frac:      0.2,
            per_level_share_frac: 0.1,
            depth_every_levels:   5,
        },
        HazardKind::Tether => HazardTuning::Tether {
            base_share_frac:         0.25,
            per_level_share_frac:    0.1,
            base_coverage_frac:      0.4,
            per_level_coverage_frac: 0.1,
        },
        HazardKind::Momentum => HazardTuning::Momentum {
            base_hp_per_hit:      10.0,
            per_level_hp_per_hit: 10.0,
        },
        HazardKind::Sympathy => HazardTuning::Sympathy {
            base_heal_frac:      0.25,
            per_level_heal_frac: 0.05,
            depth_every_levels:  5,
        },
    }
}

fn make_def(kind: HazardKind, name: &str, description: &str) -> HazardDefinition {
    HazardDefinition {
        name:        name.to_owned(),
        description: description.to_owned(),
        unlock_tier: 0,
        tuning:      tuning_for_kind(kind),
    }
}

/// Build `count` (clamped to 16 total available) offers with concrete kinds.
fn make_offers(count: usize) -> HazardOffers {
    let all = [
        (HazardKind::Decay, "Decay", "Cells lose HP over time"),
        (
            HazardKind::Drift,
            "Drift",
            "Periodic lateral force on bolts",
        ),
        (HazardKind::Haste, "Haste", "Cell-timer acceleration"),
        (HazardKind::EchoCells, "EchoCells", "Delayed-respawn cells"),
        (HazardKind::Erosion, "Erosion", "Breaker shrinks on whiff"),
    ];
    let offers: Vec<HazardDefinition> = all
        .iter()
        .take(count)
        .map(|(k, n, d)| make_def(*k, n, d))
        .collect();
    HazardOffers(offers)
}

fn test_app_with_offers(offers: HazardOffers) -> App {
    TestAppBuilder::new()
        .insert_resource(HazardSelectConfig::default())
        .insert_resource(offers)
        .with_system(Update, spawn_hazard_select)
        .build()
}

// ── Domain C.1: exactly 1 HazardSelectScreen ─────────────────────────────

#[test]
fn spawn_creates_exactly_one_hazard_select_screen_entity() {
    let mut app = test_app_with_offers(make_offers(3));
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<HazardSelectScreen>>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 1,
        "expected exactly 1 HazardSelectScreen, got {count}"
    );
}

#[test]
fn spawn_creates_screen_even_when_offers_is_empty() {
    let mut app = test_app_with_offers(make_offers(0));
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<HazardSelectScreen>>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 1,
        "empty offers must still spawn the screen so the state machine can advance"
    );
}

// ── Domain C.2: exactly 3 HazardCard when offers has 3 ────────────────────

#[test]
fn spawn_creates_exactly_three_hazard_cards_when_offers_has_three_entries() {
    let mut app = test_app_with_offers(make_offers(3));
    app.update();

    let count = app
        .world_mut()
        .query::<&HazardCard>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 3,
        "expected exactly 3 HazardCard entities, got {count}"
    );
}

#[test]
fn spawn_creates_two_hazard_cards_when_offers_has_two_entries() {
    let mut app = test_app_with_offers(make_offers(2));
    app.update();

    let count = app
        .world_mut()
        .query::<&HazardCard>()
        .iter(app.world())
        .count();
    assert_eq!(count, 2);
}

#[test]
fn spawn_creates_one_hazard_card_when_offers_has_one_entry() {
    let mut app = test_app_with_offers(make_offers(1));
    app.update();

    let count = app
        .world_mut()
        .query::<&HazardCard>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1);
}

#[test]
fn spawn_creates_zero_hazard_cards_when_offers_is_empty() {
    let mut app = test_app_with_offers(make_offers(0));
    app.update();

    let count = app
        .world_mut()
        .query::<&HazardCard>()
        .iter(app.world())
        .count();
    assert_eq!(count, 0);
}

// ── Domain C.3: HazardCard.index values are {0, 1, 2} ────────────────────

#[test]
fn hazard_card_indices_are_zero_one_two_in_a_set() {
    let mut app = test_app_with_offers(make_offers(3));
    app.update();

    let indices: HashSet<usize> = app
        .world_mut()
        .query::<&HazardCard>()
        .iter(app.world())
        .map(|c| c.index)
        .collect();
    let expected: HashSet<usize> = [0, 1, 2].into_iter().collect();
    assert_eq!(indices, expected, "HazardCard indices must be {{0, 1, 2}}");
}

// ── Domain C.4: exactly 1 HazardTimerText ────────────────────────────────

#[test]
fn spawn_creates_exactly_one_hazard_timer_text_entity() {
    let mut app = test_app_with_offers(make_offers(3));
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<HazardTimerText>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1);
}

// ── Domain C.5: first card border uses selected_color; others use normal ─

#[test]
fn first_card_border_is_selected_color_and_others_are_normal_color() {
    let mut app = test_app_with_offers(make_offers(3));
    app.update();

    let config = HazardSelectConfig::default();
    let selected = BorderColor::all(color_from_rgb(config.selected_color_rgb));
    let normal = BorderColor::all(color_from_rgb(config.normal_color_rgb));

    let mut query = app.world_mut().query::<(&HazardCard, &BorderColor)>();
    let mut seen = 0;
    for (card, border) in query.iter(app.world()) {
        let expected = if card.index == 0 { &selected } else { &normal };
        assert_eq!(
            border, expected,
            "HazardCard index {} must carry expected border, got {border:?}",
            card.index
        );
        seen += 1;
    }
    assert_eq!(
        seen, 3,
        "expected 3 HazardCard entities with BorderColor, got {seen}"
    );
}

#[test]
fn single_offer_first_card_border_is_selected_color() {
    let mut app = test_app_with_offers(make_offers(1));
    app.update();

    let config = HazardSelectConfig::default();
    let selected = BorderColor::all(color_from_rgb(config.selected_color_rgb));

    let mut query = app.world_mut().query::<(&HazardCard, &BorderColor)>();
    let mut found = 0;
    for (card, border) in query.iter(app.world()) {
        assert_eq!(card.index, 0);
        assert_eq!(border, &selected);
        found += 1;
    }
    assert_eq!(found, 1);
}

// ── Domain C.6: HazardSelectTimer inserted with remaining == config.timer_secs ─

#[test]
fn spawn_inserts_hazard_select_timer_resource_with_configured_remaining() {
    let mut app = test_app_with_offers(make_offers(3));
    app.update();

    let timer = app
        .world()
        .get_resource::<HazardSelectTimer>()
        .expect("spawn_hazard_select must insert HazardSelectTimer");
    assert!(
        (timer.remaining - 10.0).abs() < f32::EPSILON,
        "HazardSelectTimer.remaining should be 10.0 (default timer_secs), got {}",
        timer.remaining
    );
}

// ── Domain C.7: HazardSelectSelection inserted with card_index == 0 ──────

#[test]
fn spawn_inserts_hazard_select_selection_resource_with_card_index_zero() {
    let mut app = test_app_with_offers(make_offers(3));
    app.update();

    let selection = app
        .world()
        .get_resource::<HazardSelectSelection>()
        .expect("spawn_hazard_select must insert HazardSelectSelection");
    assert_eq!(selection.card_index, 0);
}

#[test]
fn spawn_twice_reinserts_selection_with_card_index_zero() {
    let mut app = test_app_with_offers(make_offers(3));
    app.update();

    // Force selection to index 2 to simulate prior navigation.
    app.world_mut()
        .insert_resource(HazardSelectSelection { card_index: 2 });

    app.update();

    let selection = app.world().resource::<HazardSelectSelection>();
    assert_eq!(
        selection.card_index, 0,
        "spawn_hazard_select must re-insert HazardSelectSelection with card_index=0 on re-entry"
    );
}

// ── Domain C.8: card Text children carry the hazard name and description ─

#[test]
fn card_text_children_carry_hazard_name_and_description() {
    let offers = HazardOffers(vec![make_def(
        HazardKind::Decay,
        "Decay",
        "Cells lose HP over time",
    )]);
    let mut app = test_app_with_offers(offers);
    app.update();

    let mut found_name = false;
    let mut found_desc = false;
    for text in app.world_mut().query::<&Text>().iter(app.world()) {
        let s: &str = text;
        if s == "Decay" {
            found_name = true;
        }
        if s == "Cells lose HP over time" {
            found_desc = true;
        }
    }
    assert!(found_name, "expected a Text child with content 'Decay'");
    assert!(
        found_desc,
        "expected a Text child with content 'Cells lose HP over time'"
    );
}
