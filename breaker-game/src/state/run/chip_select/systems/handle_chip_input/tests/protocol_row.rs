//! Protocol-row tests: navigation D.1–D.5, confirm D.6–D.10, re-entry D.11.

use bevy::{ecs::message::Messages, prelude::*};
use rantzsoft_stateflow::ChangeState;

use super::{super::*, helpers::*};
use crate::{
    mutators::protocols::{
        definition::ProtocolKind,
        resources::{ActiveProtocols, ProtocolOffer},
    },
    shared::test_utils::press_key,
    state::run::chip_select::resources::SelectionRow,
};

// ── D.1: menu_down from chip row moves selection to protocol row ──────────

#[test]
fn menu_down_from_chip_row_moves_selection_to_protocol_row() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Chip,
            chip_index: 0,
        },
    );

    press_key(&mut app, KeyCode::ArrowDown);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(
        selection.row,
        SelectionRow::Protocol,
        "menu_down from Chip row with ProtocolOffer::Some should move focus to Protocol row"
    );
}

#[test]
fn menu_down_from_chip_row_with_key_s_moves_selection_to_protocol_row() {
    // Edge case: second binding in InputConfig::default().menu_down.
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Chip,
            chip_index: 0,
        },
    );

    press_key(&mut app, KeyCode::KeyS);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.row, SelectionRow::Protocol);
}

// ── D.2: menu_up from protocol row returns selection to chip row ──────────

#[test]
fn menu_up_from_protocol_row_returns_selection_to_chip_row() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Protocol,
            chip_index: 0,
        },
    );

    press_key(&mut app, KeyCode::ArrowUp);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.row, SelectionRow::Chip);
}

#[test]
fn menu_up_from_protocol_row_preserves_chip_index() {
    // Edge case: chip_index preserved across row switch.
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Protocol,
            chip_index: 0,
        },
    );

    press_key(&mut app, KeyCode::ArrowUp);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(
        selection.chip_index, 0,
        "chip_index must be preserved across row switch"
    );
}

// ── D.3: menu_down is a no-op when ProtocolOffer is None ──────────────────

#[test]
fn menu_down_is_noop_when_protocol_offer_is_none() {
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        ProtocolOffer(None),
        ChipSelectSelection {
            row:        SelectionRow::Chip,
            chip_index: 0,
        },
    );

    press_key(&mut app, KeyCode::ArrowDown);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(
        selection.row,
        SelectionRow::Chip,
        "menu_down with ProtocolOffer(None) must not change row"
    );

    let msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(
        msgs.iter_current_update_messages().count(),
        0,
        "menu_down with no protocol offer must not emit ChangeState"
    );
}

// ── D.4: menu_up is a no-op when already on the chip row ──────────────────

#[test]
fn menu_up_is_noop_when_already_on_chip_row() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Chip,
            chip_index: 0,
        },
    );

    press_key(&mut app, KeyCode::ArrowUp);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.row, SelectionRow::Chip);
}

// ── D.5: left/right suppressed on protocol row ────────────────────────────

#[test]
fn menu_right_is_suppressed_on_protocol_row() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Protocol,
            chip_index: 1,
        },
    );

    press_key(&mut app, KeyCode::ArrowRight);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(
        selection.chip_index, 1,
        "horizontal navigation must not change chip_index while on protocol row"
    );
}

#[test]
fn menu_left_is_suppressed_on_protocol_row() {
    // Edge case: menu_left equally suppressed.
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Protocol,
            chip_index: 1,
        },
    );

    press_key(&mut app, KeyCode::ArrowLeft);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.chip_index, 1);
}

// ── D.6: confirm on chip row still sends ChipSelected ─────────────────────

#[test]
fn confirm_on_chip_row_sends_chip_selected_not_protocol_selected() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Chip,
            chip_index: 0,
        },
    );

    press_key(&mut app, KeyCode::Enter);

    // Exactly one ChipSelected message with name "Piercing Shot".
    let received_chips = app.world().resource::<ReceivedChips>();
    assert_eq!(
        received_chips.0.len(),
        1,
        "expected exactly one ChipSelected message"
    );
    assert_eq!(received_chips.0[0].name, "Piercing Shot");

    // Exactly one ChangeState<ChipSelectState> message.
    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(
        state_msgs.iter_current_update_messages().count(),
        1,
        "expected exactly one ChangeState<ChipSelectState> message"
    );

    // Zero ProtocolSelected messages.
    let received_protocols = app.world().resource::<ReceivedProtocols>();
    assert!(
        received_protocols.0.is_empty(),
        "expected no ProtocolSelected messages on chip-row confirm"
    );
}

// ── D.7: confirm on protocol row sends ProtocolSelected + ChangeState ─────

#[test]
fn confirm_on_protocol_row_sends_protocol_selected_with_correct_kind() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Protocol,
            chip_index: 0,
        },
    );

    press_key(&mut app, KeyCode::Enter);

    let received_protocols = app.world().resource::<ReceivedProtocols>();
    assert_eq!(
        received_protocols.0.len(),
        1,
        "expected exactly one ProtocolSelected message"
    );
    assert_eq!(received_protocols.0[0].kind, ProtocolKind::Greed);

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(
        state_msgs.iter_current_update_messages().count(),
        1,
        "expected exactly one ChangeState<ChipSelectState> message"
    );

    let received_chips = app.world().resource::<ReceivedChips>();
    assert!(
        received_chips.0.is_empty(),
        "expected no ChipSelected messages on protocol-row confirm"
    );
}

#[test]
fn confirm_on_protocol_row_sends_protocol_selected_for_burnout() {
    // Edge case: a different kind is returned verbatim.
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Burnout, "Burnout")));
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Protocol,
            chip_index: 0,
        },
    );

    press_key(&mut app, KeyCode::Enter);

    let received = app.world().resource::<ReceivedProtocols>();
    assert_eq!(received.0.len(), 1);
    assert_eq!(received.0[0].kind, ProtocolKind::Burnout);
}

// ── D.8: confirm on protocol row applies decay to every offered chip ──────

#[test]
fn confirm_on_protocol_row_decays_every_offered_chip() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Protocol,
            chip_index: 0,
        },
    );

    press_key(&mut app, KeyCode::Enter);

    let inventory = app.world().resource::<ChipInventory>();
    for name in ["Piercing Shot", "Wide Breaker", "Surge"] {
        let decay = inventory.weight_decay(name);
        assert!(
            (decay - 0.8).abs() < 1e-6,
            "expected chip '{name}' to have decay ~0.8 after protocol confirm, got {decay}"
        );
    }

    // One ProtocolSelected message emitted.
    let protocols = app.world().resource::<ReceivedProtocols>();
    assert_eq!(protocols.0.len(), 1);
    assert_eq!(protocols.0[0].kind, ProtocolKind::Greed);

    // One ChangeState, no ChipSelected.
    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(state_msgs.iter_current_update_messages().count(), 1);
    let chips = app.world().resource::<ReceivedChips>();
    assert!(chips.0.is_empty());
}

#[test]
fn confirm_on_protocol_row_with_empty_offers_does_not_panic() {
    // Edge case: empty chip offers + protocol-row confirm still works.
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_with_protocol_offer(
        make_offers(0),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Protocol,
            chip_index: 0,
        },
    );

    press_key(&mut app, KeyCode::Enter);

    let protocols = app.world().resource::<ReceivedProtocols>();
    assert_eq!(
        protocols.0.len(),
        1,
        "expected one ProtocolSelected even with no chip offers"
    );

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(
        state_msgs.iter_current_update_messages().count(),
        1,
        "expected one ChangeState even with no chip offers"
    );
}

// ── D.9: robustness — confirm on protocol row with ProtocolOffer(None) ────

#[test]
fn confirm_on_protocol_row_with_none_offer_emits_no_protocol_selected() {
    // Unreachable in practice (D.3 prevents the row switch). This guards
    // against a panic and asserts no ProtocolSelected leaks out.
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        ProtocolOffer(None),
        ChipSelectSelection {
            row:        SelectionRow::Protocol,
            chip_index: 0,
        },
    );

    press_key(&mut app, KeyCode::Enter);

    let protocols = app.world().resource::<ReceivedProtocols>();
    assert!(
        protocols.0.is_empty(),
        "ProtocolOffer::None must not emit a ProtocolSelected even if row was forced to Protocol"
    );
}

// ── D.10: confirm on chip row unaffected when ProtocolOffer is None ───────

#[test]
fn confirm_on_chip_row_is_unaffected_when_protocol_offer_is_none() {
    let mut app = test_app_with_protocol_offer(
        make_offers(3),
        ProtocolOffer(None),
        ChipSelectSelection {
            row:        SelectionRow::Chip,
            chip_index: 0,
        },
    );

    press_key(&mut app, KeyCode::Enter);

    let received_chips = app.world().resource::<ReceivedChips>();
    assert_eq!(received_chips.0.len(), 1);
    assert_eq!(received_chips.0[0].name, "Piercing Shot");

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(state_msgs.iter_current_update_messages().count(), 1);

    let received_protocols = app.world().resource::<ReceivedProtocols>();
    assert!(received_protocols.0.is_empty());
}

// ── D.11: navigation state resets between chip-select visits ──────────────

#[test]
fn spawn_chip_select_reinserts_selection_with_chip_row_default_on_re_entry() {
    // A lighter form of the end-to-end state re-entry test from the spec:
    // prove `spawn_chip_select` re-inserts `ChipSelectSelection` with the
    // default row. We simulate re-entry by calling the spawn system twice.
    use crate::state::run::chip_select::systems::spawn_chip_select;

    let mut app = TestAppBuilder::new()
        .insert_resource(ChipSelectConfig::default())
        .insert_resource(make_offers(3))
        .with_resource::<ProtocolOffer>()
        .with_system(Update, spawn_chip_select)
        .build();
    app.insert_resource(ActiveProtocols::default());

    // First visit: spawn runs, inserts default selection.
    app.update();
    // Force the selection to the Protocol row to simulate mid-session navigation.
    app.world_mut().insert_resource(ChipSelectSelection {
        row:        SelectionRow::Protocol,
        chip_index: 2,
    });

    // Second visit: spawn_chip_select must overwrite with a fresh default.
    app.update();

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(
        selection.row,
        SelectionRow::Chip,
        "spawn_chip_select must re-insert ChipSelectSelection with the default row on re-entry"
    );
    assert_eq!(selection.chip_index, 0);
}
