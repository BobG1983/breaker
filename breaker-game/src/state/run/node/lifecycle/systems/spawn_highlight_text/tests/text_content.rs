use bevy::prelude::*;
use rantzsoft_stateflow::CleanupOnExit;

use super::helpers::*;
use crate::{
    fx::{FadeOut, PunchScale},
    state::{
        run::{components::HighlightPopup, messages::HighlightTriggered, resources::HighlightKind},
        types::NodeState,
    },
};

// ---------------------------------------------------------------
// Behavior 1: Text and color mapping for each HighlightKind
// ---------------------------------------------------------------

/// Runs a single highlight kind through the system and asserts that:
/// 1. A `Text2d` entity is spawned containing `expected_text`.
/// 2. The entity has a `TextColor` matching `expected_color`.
/// 3. The entity has a `FadeOut` component.
/// 4. The entity has a `CleanupOnExit<NodeState>` component.
/// 5. The entity has a `HighlightPopup` component.
/// 6. The entity has a `PunchScale` component.
fn assert_highlight_spawns_popup(kind: HighlightKind, expected_text: &str, expected_color: Color) {
    let mut app = test_app();
    app.insert_resource(TestHighlightMsg(vec![HighlightTriggered { kind }]));
    app.update();

    let texts: Vec<String> = app
        .world_mut()
        .query::<&Text2d>()
        .iter(app.world())
        .map(|t| t.0.clone())
        .collect();

    assert!(
        texts.iter().any(|t| t.contains(expected_text)),
        "expected Text2d containing {expected_text:?}, found: {texts:?}"
    );

    // Verify TextColor matches expected
    let text_color = app
        .world_mut()
        .query_filtered::<&TextColor, With<HighlightPopup>>()
        .iter(app.world())
        .next()
        .expect("popup should have TextColor");
    assert_eq!(
        text_color.0, expected_color,
        "TextColor should match expected color for {expected_text:?}"
    );

    // Verify all required components exist on the popup entity
    let popup_count = app
        .world_mut()
        .query_filtered::<Entity, (
            With<Text2d>,
            With<FadeOut>,
            With<CleanupOnExit<NodeState>>,
            With<HighlightPopup>,
            With<PunchScale>,
        )>()
        .iter(app.world())
        .count();
    assert_eq!(
        popup_count, 1,
        "popup entity should have Text2d, FadeOut, CleanupOnExit<NodeState>, HighlightPopup, and PunchScale"
    );
}

#[test]
fn clutch_clear_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::ClutchClear,
        "CLUTCH CLEAR!",
        Color::srgb(0.0, 1.0, 1.0),
    );
}

#[test]
fn mass_destruction_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::MassDestruction,
        "MASS DESTRUCTION!",
        Color::srgb(1.0, 0.6, 0.0),
    );
}

#[test]
fn perfect_streak_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::PerfectStreak,
        "PERFECT STREAK!",
        Color::srgb(0.0, 1.0, 1.0),
    );
}

#[test]
fn fast_clear_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::FastClear,
        "SPEED CLEAR!",
        Color::srgb(0.0, 1.0, 1.0),
    );
}

#[test]
fn first_evolution_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::FirstEvolution,
        "FIRST EVOLUTION!",
        Color::srgb(1.0, 1.0, 0.0),
    );
}

#[test]
fn no_damage_node_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::NoDamageNode,
        "FLAWLESS!",
        Color::srgb(0.0, 1.0, 0.4),
    );
}

#[test]
fn close_save_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::CloseSave,
        "CLOSE SAVE!",
        Color::srgb(0.0, 1.0, 0.4),
    );
}

#[test]
fn speed_demon_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::SpeedDemon,
        "SPEED DEMON!",
        Color::srgb(0.0, 1.0, 1.0),
    );
}

#[test]
fn untouchable_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::Untouchable,
        "UNTOUCHABLE!",
        Color::srgb(0.0, 1.0, 1.0),
    );
}

#[test]
fn combo_king_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::ComboKing,
        "COMBO KING!",
        Color::srgb(1.0, 0.6, 0.0),
    );
}

#[test]
fn pinball_wizard_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::PinballWizard,
        "PINBALL WIZARD!",
        Color::srgb(1.0, 0.6, 0.0),
    );
}

#[test]
fn comeback_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::Comeback,
        "COMEBACK!",
        Color::srgb(0.0, 1.0, 0.4),
    );
}

#[test]
fn perfect_node_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::PerfectNode,
        "PERFECT NODE!",
        Color::srgb(0.0, 1.0, 1.0),
    );
}

#[test]
fn nail_biter_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::NailBiter,
        "NAIL BITER!",
        Color::srgb(0.0, 1.0, 0.4),
    );
}

#[test]
fn most_powerful_evolution_spawns_popup() {
    assert_highlight_spawns_popup(
        HighlightKind::MostPowerfulEvolution,
        "DEVASTATING!",
        Color::srgb(1.0, 0.6, 0.0),
    );
}
