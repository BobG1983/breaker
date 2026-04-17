//! Tests for `handle_hazard_input` — left/right navigation with wrap and
//! Enter/Space confirm that emits `HazardSelected` + `ChangeState`.

use bevy::{ecs::message::Messages, prelude::*};
use rantzsoft_stateflow::ChangeState;

use super::*;
use crate::{
    hazard::{
        definition::{HazardDefinition, HazardKind, HazardTuning},
        messages::HazardSelected,
        resources::HazardOffers,
    },
    input::InputConfig,
    prelude::*,
    state::run::hazard_select::resources::HazardSelectSelection,
};

fn tuning_for_kind(kind: HazardKind) -> HazardTuning {
    match kind {
        HazardKind::Drift => HazardTuning::Drift {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        },
        HazardKind::Haste => HazardTuning::Haste {
            base_percent:      0.1,
            per_level_percent: 0.05,
        },
        _ => HazardTuning::Decay {
            base_percent:      0.05,
            per_level_percent: 0.03,
        },
    }
}

fn make_def(kind: HazardKind, name: &str) -> HazardDefinition {
    HazardDefinition {
        name:        name.to_owned(),
        description: String::new(),
        unlock_tier: 0,
        tuning:      tuning_for_kind(kind),
    }
}

fn make_offers(count: usize) -> HazardOffers {
    let all = vec![
        make_def(HazardKind::Decay, "Decay"),
        make_def(HazardKind::Drift, "Drift"),
        make_def(HazardKind::Haste, "Haste"),
    ];
    HazardOffers(all.into_iter().take(count).collect())
}

#[derive(Resource, Default)]
struct ReceivedHazards(Vec<HazardSelected>);

fn collect_hazards(
    mut reader: MessageReader<HazardSelected>,
    mut received: ResMut<ReceivedHazards>,
) {
    for msg in reader.read() {
        received.0.push(msg.clone());
    }
}

fn test_app_with_offers_and_selection(
    offers: HazardOffers,
    selection: HazardSelectSelection,
) -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ButtonInput<KeyCode>>()
        .insert_resource(InputConfig::default())
        .insert_resource(selection)
        .insert_resource(offers)
        .with_resource::<ReceivedHazards>()
        .with_message::<HazardSelected>()
        .with_message::<ChangeState<HazardSelectState>>()
        .with_system(Update, (handle_hazard_input, collect_hazards).chain())
        .build()
}

fn test_app_with_offers(offers: HazardOffers) -> App {
    test_app_with_offers_and_selection(offers, HazardSelectSelection { card_index: 0 })
}

fn press_key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
}

// ── Domain D.1: menu_right advances card_index by 1 ──────────────────────

#[test]
fn menu_right_advances_card_index_by_one_arrow_right() {
    let mut app = test_app_with_offers(make_offers(3));
    press_key(&mut app, KeyCode::ArrowRight);

    let selection = app.world().resource::<HazardSelectSelection>();
    assert_eq!(selection.card_index, 1);
}

#[test]
fn menu_right_advances_card_index_by_one_key_d() {
    let mut app = test_app_with_offers(make_offers(3));
    press_key(&mut app, KeyCode::KeyD);

    let selection = app.world().resource::<HazardSelectSelection>();
    assert_eq!(selection.card_index, 1);
}

// ── Domain D.2: menu_right wraps 2 → 0 ────────────────────────────────────

#[test]
fn menu_right_wraps_from_two_to_zero() {
    let mut app =
        test_app_with_offers_and_selection(make_offers(3), HazardSelectSelection { card_index: 2 });
    press_key(&mut app, KeyCode::ArrowRight);

    let selection = app.world().resource::<HazardSelectSelection>();
    assert_eq!(selection.card_index, 0);
}

// ── Domain D.3: menu_left decrements card_index by 1 ─────────────────────

#[test]
fn menu_left_decrements_card_index_by_one() {
    let mut app =
        test_app_with_offers_and_selection(make_offers(3), HazardSelectSelection { card_index: 1 });
    press_key(&mut app, KeyCode::ArrowLeft);

    let selection = app.world().resource::<HazardSelectSelection>();
    assert_eq!(selection.card_index, 0);
}

// ── Domain D.4: menu_left wraps 0 → 2 ─────────────────────────────────────

#[test]
fn menu_left_wraps_from_zero_to_two_arrow_left() {
    let mut app = test_app_with_offers(make_offers(3));
    press_key(&mut app, KeyCode::ArrowLeft);

    let selection = app.world().resource::<HazardSelectSelection>();
    assert_eq!(selection.card_index, 2);
}

#[test]
fn menu_left_wraps_from_zero_to_two_key_a() {
    let mut app = test_app_with_offers(make_offers(3));
    press_key(&mut app, KeyCode::KeyA);

    let selection = app.world().resource::<HazardSelectSelection>();
    assert_eq!(selection.card_index, 2);
}

// ── Domain D.5: menu_confirm on card 0 emits HazardSelected(Decay) + ChangeState ──

#[test]
fn menu_confirm_on_card_zero_emits_decay_hazard_selected() {
    let mut app = test_app_with_offers(make_offers(3));
    press_key(&mut app, KeyCode::Enter);

    let received = app.world().resource::<ReceivedHazards>();
    assert_eq!(
        received.0.len(),
        1,
        "expected exactly one HazardSelected message"
    );
    assert_eq!(received.0[0].kind, HazardKind::Decay);

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<HazardSelectState>>>();
    assert_eq!(
        state_msgs.iter_current_update_messages().count(),
        1,
        "expected exactly one ChangeState<HazardSelectState> message"
    );
}

#[test]
fn menu_confirm_on_card_zero_via_space_emits_decay_hazard_selected() {
    let mut app = test_app_with_offers(make_offers(3));
    press_key(&mut app, KeyCode::Space);

    let received = app.world().resource::<ReceivedHazards>();
    assert_eq!(received.0.len(), 1);
    assert_eq!(received.0[0].kind, HazardKind::Decay);

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<HazardSelectState>>>();
    assert_eq!(state_msgs.iter_current_update_messages().count(), 1);
}

// ── Domain D.6: menu_confirm on card 1 emits HazardSelected(Drift) ───────

#[test]
fn menu_confirm_on_card_one_emits_drift_hazard_selected() {
    let mut app =
        test_app_with_offers_and_selection(make_offers(3), HazardSelectSelection { card_index: 1 });
    press_key(&mut app, KeyCode::Enter);

    let received = app.world().resource::<ReceivedHazards>();
    assert_eq!(received.0.len(), 1);
    assert_eq!(received.0[0].kind, HazardKind::Drift);
}

// ── Domain D.7: menu_confirm on card 2 emits HazardSelected(Haste) ───────

#[test]
fn menu_confirm_on_card_two_emits_haste_hazard_selected() {
    let mut app =
        test_app_with_offers_and_selection(make_offers(3), HazardSelectSelection { card_index: 2 });
    press_key(&mut app, KeyCode::Enter);

    let received = app.world().resource::<ReceivedHazards>();
    assert_eq!(received.0.len(), 1);
    assert_eq!(received.0[0].kind, HazardKind::Haste);
}

// ── Domain D.8: empty offers + confirm emits ChangeState but no HazardSelected ──

#[test]
fn empty_offers_plus_confirm_emits_change_state_without_hazard_selected() {
    let mut app = test_app_with_offers(make_offers(0));
    press_key(&mut app, KeyCode::Enter);

    let received = app.world().resource::<ReceivedHazards>();
    assert!(
        received.0.is_empty(),
        "empty offers + confirm must not emit HazardSelected"
    );

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<HazardSelectState>>>();
    assert_eq!(
        state_msgs.iter_current_update_messages().count(),
        1,
        "empty offers + confirm must still emit one ChangeState so the state machine advances"
    );
}

// ── Domain D.9: no input produces no messages and no selection change ────

#[test]
fn no_input_produces_no_messages_and_no_selection_change() {
    let mut app = test_app_with_offers(make_offers(3));
    app.update();

    let selection = app.world().resource::<HazardSelectSelection>();
    assert_eq!(selection.card_index, 0);

    let received = app.world().resource::<ReceivedHazards>();
    assert!(received.0.is_empty());

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<HazardSelectState>>>();
    assert_eq!(state_msgs.iter_current_update_messages().count(), 0);
}

// ── Domain D.10: arrow keys do NOT emit HazardSelected ───────────────────

#[test]
fn arrow_keys_do_not_emit_hazard_selected_or_change_state() {
    let mut app = test_app_with_offers(make_offers(3));
    press_key(&mut app, KeyCode::ArrowRight);
    press_key(&mut app, KeyCode::ArrowLeft);

    let received = app.world().resource::<ReceivedHazards>();
    assert!(
        received.0.is_empty(),
        "navigation must not emit HazardSelected"
    );

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<HazardSelectState>>>();
    assert_eq!(
        state_msgs.iter_current_update_messages().count(),
        0,
        "navigation must not emit ChangeState"
    );
}

// ── Domain D.11: 2-card wrap is modulo-based (not hardcoded % 3) ─────────

#[test]
fn two_card_offers_navigation_wrap_is_modulo_two() {
    let mut app =
        test_app_with_offers_and_selection(make_offers(2), HazardSelectSelection { card_index: 1 });
    press_key(&mut app, KeyCode::ArrowRight);
    let selection = app.world().resource::<HazardSelectSelection>();
    assert_eq!(
        selection.card_index, 0,
        "2-card wrap: index 1 + right must wrap to 0 (NOT hardcoded % 3)"
    );

    press_key(&mut app, KeyCode::ArrowRight);
    let selection = app.world().resource::<HazardSelectSelection>();
    assert_eq!(selection.card_index, 1);
}

#[test]
fn single_offer_navigation_is_noop() {
    let mut app = test_app_with_offers(make_offers(1));
    press_key(&mut app, KeyCode::ArrowRight);
    let selection = app.world().resource::<HazardSelectSelection>();
    assert_eq!(
        selection.card_index, 0,
        "1-card wrap: modulo 1 keeps card_index at 0"
    );
}
