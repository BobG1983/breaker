//! Skip-row tests (B4–B14): Greed-gated navigation + confirm.

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

// Helper: build an `ActiveProtocols` with Greed inserted via `def_for`.
fn active_with_greed() -> ActiveProtocols {
    let mut active = ActiveProtocols::default();
    active.insert(def_for(ProtocolKind::Greed, "Greed"));
    active
}

// ── B4: Down from Protocol row navigates to Skip row when Greed is active ──

#[test]
fn down_from_protocol_row_navigates_to_skip_when_greed_active() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Protocol,
            chip_index: 0,
        },
        active_with_greed(),
    );

    press_key(&mut app, KeyCode::ArrowDown);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.row, SelectionRow::Skip);

    // No state change, no chip/protocol/skip emissions this tick.
    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(state_msgs.iter_current_update_messages().count(), 0);

    let chips = app.world().resource::<ReceivedChips>();
    assert!(chips.0.is_empty());
    let protocols = app.world().resource::<ReceivedProtocols>();
    assert!(protocols.0.is_empty());
    let skips = app.world().resource::<ReceivedSkips>();
    assert_eq!(skips.0, 0);
}

#[test]
fn down_from_protocol_row_navigates_to_skip_with_key_s() {
    // Edge case of B4: second binding for menu_down is `KeyS`.
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Protocol,
            chip_index: 0,
        },
        active_with_greed(),
    );

    press_key(&mut app, KeyCode::KeyS);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.row, SelectionRow::Skip);
}

// ── B5: Down from Chip row goes directly to Skip when no protocol offer ──

#[test]
fn down_from_chip_row_goes_to_skip_when_no_protocol_offer_and_greed_active() {
    let mut app = test_app_greed_input(
        make_offers(3),
        ProtocolOffer(None),
        ChipSelectSelection {
            row:        SelectionRow::Chip,
            chip_index: 1,
        },
        active_with_greed(),
    );

    press_key(&mut app, KeyCode::ArrowDown);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.row, SelectionRow::Skip);
    assert_eq!(
        selection.chip_index, 1,
        "chip_index must be preserved across row switch"
    );
}

#[test]
fn down_from_chip_row_with_zero_offers_and_no_protocol_goes_to_skip_when_greed_active() {
    // Edge case of B5: zero chips offered + no protocol offer + Greed active.
    // Down navigates to Skip — the player can still skip even when zero chips
    // were offered. (Confirm-with-empty-offers behavior is covered by B11.)
    let mut app = test_app_greed_input(
        make_offers(0),
        ProtocolOffer(None),
        ChipSelectSelection {
            row:        SelectionRow::Chip,
            chip_index: 0,
        },
        active_with_greed(),
    );

    press_key(&mut app, KeyCode::ArrowDown);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.row, SelectionRow::Skip);
}

// ── B6: Down from Chip with protocol offer goes to Protocol first (not Skip) ──

#[test]
fn down_from_chip_with_protocol_offer_goes_to_protocol_first_when_greed_active() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Chip,
            chip_index: 0,
        },
        active_with_greed(),
    );

    press_key(&mut app, KeyCode::ArrowDown);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(
        selection.row,
        SelectionRow::Protocol,
        "Skip must NOT absorb Down from Chip when a protocol offer exists"
    );
}

// ── B7: Down does NOT enter Skip when Greed is inactive ──

#[test]
fn down_from_protocol_row_does_not_enter_skip_when_greed_inactive() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Protocol,
            chip_index: 0,
        },
        ActiveProtocols::default(), // Greed inactive
    );

    press_key(&mut app, KeyCode::ArrowDown);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(
        selection.row,
        SelectionRow::Protocol,
        "Down from Protocol must do nothing when Greed is inactive"
    );

    let skips = app.world().resource::<ReceivedSkips>();
    assert_eq!(skips.0, 0);

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(state_msgs.iter_current_update_messages().count(), 0);
}

#[test]
fn down_from_chip_row_does_not_enter_skip_when_greed_inactive_and_no_protocol() {
    // Edge case of B7: chip row + no protocol offer + Greed inactive.
    // Down does nothing — there's no Skip row to navigate to.
    let mut app = test_app_greed_input(
        make_offers(3),
        ProtocolOffer(None),
        ChipSelectSelection {
            row:        SelectionRow::Chip,
            chip_index: 0,
        },
        ActiveProtocols::default(),
    );

    press_key(&mut app, KeyCode::ArrowDown);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.row, SelectionRow::Chip);
}

// ── B8: Up from Skip row returns to Protocol row when a protocol offer exists ──

#[test]
fn up_from_skip_row_returns_to_protocol_when_offer_exists() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Skip,
            chip_index: 2,
        },
        active_with_greed(),
    );

    press_key(&mut app, KeyCode::ArrowUp);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.row, SelectionRow::Protocol);
    assert_eq!(
        selection.chip_index, 2,
        "chip_index must be preserved across row switch"
    );
}

#[test]
fn up_from_skip_row_returns_to_protocol_with_key_w() {
    // Edge case of B8: second binding for menu_up is `KeyW`.
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Skip,
            chip_index: 2,
        },
        active_with_greed(),
    );

    press_key(&mut app, KeyCode::KeyW);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.row, SelectionRow::Protocol);
}

// ── B9: Up from Skip row returns to Chip row when no protocol offer ──

#[test]
fn up_from_skip_row_returns_to_chip_when_no_protocol_offer() {
    let mut app = test_app_greed_input(
        make_offers(3),
        ProtocolOffer(None),
        ChipSelectSelection {
            row:        SelectionRow::Skip,
            chip_index: 0,
        },
        active_with_greed(),
    );

    press_key(&mut app, KeyCode::ArrowUp);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.row, SelectionRow::Chip);
    assert_eq!(selection.chip_index, 0, "chip_index preserved");
}

// ── B10: Left/Right are suppressed on the Skip row ──

#[test]
fn right_is_suppressed_on_skip_row() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Skip,
            chip_index: 1,
        },
        active_with_greed(),
    );

    press_key(&mut app, KeyCode::ArrowRight);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(
        selection.chip_index, 1,
        "horizontal nav must not change chip_index while on skip row"
    );
}

#[test]
fn left_is_suppressed_on_skip_row() {
    // Edge case of B10: menu_left equally suppressed.
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Skip,
            chip_index: 1,
        },
        active_with_greed(),
    );

    press_key(&mut app, KeyCode::ArrowLeft);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.chip_index, 1);
}

// ── B11: Confirm on Skip row writes ChipOfferSkipped, decays chips, transitions ──

#[test]
fn confirm_on_skip_row_emits_one_skip_decays_chips_and_transitions() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Skip,
            chip_index: 0,
        },
        active_with_greed(),
    );

    press_key(&mut app, KeyCode::Enter);

    let skips = app.world().resource::<ReceivedSkips>();
    assert_eq!(
        skips.0, 1,
        "expected exactly one ChipOfferSkipped message this tick"
    );

    let chips = app.world().resource::<ReceivedChips>();
    assert!(chips.0.is_empty(), "no ChipSelected on Skip-row confirm");

    let protocols = app.world().resource::<ReceivedProtocols>();
    assert!(
        protocols.0.is_empty(),
        "no ProtocolSelected on Skip-row confirm"
    );

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(
        state_msgs.iter_current_update_messages().count(),
        1,
        "expected exactly one ChangeState<ChipSelectState>"
    );

    let inventory = app.world().resource::<ChipInventory>();
    for name in ["Piercing Shot", "Wide Breaker", "Surge"] {
        let decay = inventory.weight_decay(name);
        assert!(
            (decay - 0.8).abs() < 1e-6,
            "expected chip '{name}' to have decay ~0.8 after Skip confirm, got {decay}"
        );
    }
}

#[test]
fn confirm_on_skip_row_emits_one_skip_when_no_protocol_offered() {
    // Edge case 1 of B11: no protocol offered this visit. All assertions
    // still hold — Skip behavior does not depend on the protocol offer.
    let mut app = test_app_greed_input(
        make_offers(3),
        ProtocolOffer(None),
        ChipSelectSelection {
            row:        SelectionRow::Skip,
            chip_index: 0,
        },
        active_with_greed(),
    );

    press_key(&mut app, KeyCode::Enter);

    let skips = app.world().resource::<ReceivedSkips>();
    assert_eq!(skips.0, 1);

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(state_msgs.iter_current_update_messages().count(), 1);

    let inventory = app.world().resource::<ChipInventory>();
    for name in ["Piercing Shot", "Wide Breaker", "Surge"] {
        let decay = inventory.weight_decay(name);
        assert!(
            (decay - 0.8).abs() < 1e-6,
            "expected chip '{name}' to have decay ~0.8, got {decay}"
        );
    }
}

#[test]
fn confirm_on_skip_row_with_zero_chips_emits_one_skip_and_transitions() {
    // Edge case 2 of B11: zero chips offered + Greed active +
    // ProtocolOffer(Some(_)) so the top-level early return does NOT fire.
    // Skip arm must handle zero offers cleanly without panicking.
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(0),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Skip,
            chip_index: 0,
        },
        active_with_greed(),
    );

    // Pre-existing inventory entry to verify weight_decay is unchanged.
    // Before pressing Enter, weight_decay for an unknown chip is 1.0 (the
    // default). Assert that after confirm it is still 1.0 (record_offered
    // was not called for any of the zero offers).
    let pre_decay_unknown = {
        let inventory = app.world().resource::<ChipInventory>();
        inventory.weight_decay("Piercing Shot")
    };
    assert!((pre_decay_unknown - 1.0).abs() < f32::EPSILON);

    press_key(&mut app, KeyCode::Enter);

    let skips = app.world().resource::<ReceivedSkips>();
    assert_eq!(skips.0, 1, "expected exactly one ChipOfferSkipped message");

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(
        state_msgs.iter_current_update_messages().count(),
        1,
        "expected exactly one ChangeState<ChipSelectState>"
    );

    // record_offered was called zero times — pre-existing entries unchanged.
    let post_decay = {
        let inventory = app.world().resource::<ChipInventory>();
        inventory.weight_decay("Piercing Shot")
    };
    assert!(
        (post_decay - 1.0).abs() < f32::EPSILON,
        "weight_decay for any pre-existing entry must be unchanged with zero chip offers"
    );
}

#[test]
fn confirm_on_skip_row_via_space_emits_one_skip() {
    // Edge case 3 of B11: confirm via `KeyCode::Space` (the second
    // menu_confirm binding) — same outcome as Enter.
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Skip,
            chip_index: 0,
        },
        active_with_greed(),
    );

    press_key(&mut app, KeyCode::Space);

    let skips = app.world().resource::<ReceivedSkips>();
    assert_eq!(skips.0, 1);

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(state_msgs.iter_current_update_messages().count(), 1);
}

// ── B12: Confirm on Skip row uses the configured `seen_decay_factor` ──

#[test]
fn confirm_on_skip_row_uses_configured_seen_decay_factor() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Skip,
            chip_index: 0,
        },
        active_with_greed(),
    );
    // Override the default ChipSelectConfig with a custom seen_decay_factor.
    app.insert_resource(ChipSelectConfig {
        seen_decay_factor: 0.5,
        ..ChipSelectConfig::default()
    });

    press_key(&mut app, KeyCode::Enter);

    let inventory = app.world().resource::<ChipInventory>();
    for name in ["Piercing Shot", "Wide Breaker", "Surge"] {
        let decay = inventory.weight_decay(name);
        assert!(
            (decay - 0.5).abs() < 1e-6,
            "expected chip '{name}' to have decay ~0.5 (configured), got {decay}"
        );
    }
}

// ── B13: Confirm on Skip row with Greed inactive — Skip arm fires unconditionally ──

#[test]
fn confirm_on_skip_row_with_greed_inactive_still_emits_chip_offer_skipped() {
    // Robustness contract: the Skip arm in `handle_chip_input` does NOT
    // re-check `ActiveProtocols`. The gate is purely UI-level (the row is
    // unreachable via navigation). Once `SelectionRow::Skip` is set, confirm
    // fires unconditionally — `greed_on_skip` enforces the gate downstream.
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Skip,
            chip_index: 0,
        },
        ActiveProtocols::default(), // Greed INACTIVE
    );

    press_key(&mut app, KeyCode::Enter);

    let skips = app.world().resource::<ReceivedSkips>();
    assert_eq!(
        skips.0, 1,
        "Skip-row confirm must emit ChipOfferSkipped regardless of ActiveProtocols"
    );

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(state_msgs.iter_current_update_messages().count(), 1);

    let inventory = app.world().resource::<ChipInventory>();
    for name in ["Piercing Shot", "Wide Breaker", "Surge"] {
        let decay = inventory.weight_decay(name);
        assert!(
            (decay - 0.8).abs() < 1e-6,
            "expected chip '{name}' to have decay ~0.8, got {decay}"
        );
    }
}

// ── B14: Regression — Greed inactive, Down from Protocol → confirm emits ProtocolSelected ──

#[test]
fn greed_inactive_down_from_protocol_then_confirm_emits_protocol_selected() {
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = test_app_greed_input(
        make_offers(3),
        offer,
        ChipSelectSelection {
            row:        SelectionRow::Protocol,
            chip_index: 0,
        },
        ActiveProtocols::default(), // Greed inactive
    );

    // Tick 1: press Down. With Greed inactive there is no Skip row to
    // navigate to — selection.row stays at Protocol.
    press_key(&mut app, KeyCode::ArrowDown);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();

    // Tick 2: press Enter. Confirm must fire the existing Protocol-row
    // path — emit one ProtocolSelected with kind == Greed.
    press_key(&mut app, KeyCode::Enter);

    let received_protocols = app.world().resource::<ReceivedProtocols>();
    assert_eq!(received_protocols.0.len(), 1);
    assert_eq!(received_protocols.0[0].kind, ProtocolKind::Greed);

    let skips = app.world().resource::<ReceivedSkips>();
    assert_eq!(
        skips.0, 0,
        "no ChipOfferSkipped should be emitted when Greed is inactive"
    );

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(
        state_msgs.iter_current_update_messages().count(),
        1,
        "expected exactly one ChangeState in the confirm tick"
    );
}
