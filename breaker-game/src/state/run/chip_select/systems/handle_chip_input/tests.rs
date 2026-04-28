//! Tests for `handle_chip_input` chip selection screen input handling.

use bevy::{ecs::message::Messages, prelude::*};
use rantzsoft_stateflow::ChangeState;

use super::*;
use crate::{
    chips::{ChipDefinition, definition::EvolutionIngredient},
    effect_v3::{
        effects::{DamageBoostConfig, PiercingConfig},
        types::{EffectType, Tree},
    },
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        messages::ProtocolSelected,
        resources::ProtocolOffer,
    },
    state::run::chip_select::resources::{ChipOffering, SelectionRow},
};

#[derive(Resource, Default)]
struct ReceivedChips(Vec<ChipSelected>);

fn collect_chips(mut reader: MessageReader<ChipSelected>, mut received: ResMut<ReceivedChips>) {
    for msg in reader.read() {
        received.0.push(msg.clone());
    }
}

fn make_offers(count: usize) -> ChipOffers {
    let all = vec![
        ChipOffering::Normal(ChipDefinition::test(
            "Piercing Shot",
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
            3,
        )),
        ChipOffering::Normal(ChipDefinition::test_simple("Wide Breaker")),
        ChipOffering::Normal(ChipDefinition::test_simple("Surge")),
    ];
    ChipOffers(all.into_iter().take(count).collect())
}

fn test_app() -> App {
    test_app_with_offers(make_offers(3))
}

fn test_app_with_offers(offers: ChipOffers) -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ButtonInput<KeyCode>>()
        .insert_resource(InputConfig::default())
        .insert_resource(ChipSelectSelection {
            row:        SelectionRow::Chip,
            chip_index: 0,
        })
        .insert_resource(offers)
        .with_resource::<ReceivedChips>()
        .with_resource::<ChipInventory>()
        .insert_resource(ChipSelectConfig::default())
        .with_resource::<ProtocolOffer>()
        .with_message::<ChipSelected>()
        .with_message::<ProtocolSelected>()
        .with_message::<ChangeState<ChipSelectState>>()
        .with_system(Update, (handle_chip_input, collect_chips).chain())
        .build()
}

/// Build a `ProtocolDefinition` for a given `kind` with a human-readable name.
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
            risky_zone_start:  0.7,
            damage_multiplier: 4.0,
            double_penalty:    true,
        },
        ProtocolKind::Burnout => ProtocolTuning::Burnout {
            fill_duration:               4.0,
            drain_duration:              2.0,
            still_threshold:             1.5,
            full_heat_damage_multiplier: 4.0,
            speed_boost_duration:        2.0,
        },
        ProtocolKind::Conductor => ProtocolTuning::Conductor,
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

/// Build a test app with three chip offers, an empty protocol offer by
/// default, and the `ProtocolSelected` message registered.
fn test_app_with_offers_and_protocol_offer(offers: ChipOffers, protocol: ProtocolOffer) -> App {
    let mut app = test_app_with_offers(offers);
    app.insert_resource(protocol);
    app
}

use crate::shared::test_utils::press_key;

#[test]
fn right_advances_selection() {
    let mut app = test_app();
    press_key(&mut app, KeyCode::ArrowRight);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.chip_index, 1);
}

#[test]
fn left_wraps_selection() {
    let mut app = test_app();
    press_key(&mut app, KeyCode::ArrowLeft);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.chip_index, 2); // wraps from 0 to last (2)
}

#[test]
fn confirm_transitions_to_transition_in() {
    let mut app = test_app();
    press_key(&mut app, KeyCode::Enter);

    let msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert!(
        msgs.iter_current_update_messages().count() > 0,
        "expected ChangeState<ChipSelectState> message"
    );
}

#[test]
fn confirm_sends_chip_selected_message() {
    let mut app = test_app();
    press_key(&mut app, KeyCode::Enter);

    let received = app.world().resource::<ReceivedChips>();
    assert_eq!(received.0.len(), 1);
    assert_eq!(received.0[0].name, "Piercing Shot");
}

#[test]
fn confirm_second_card_sends_correct_chip() {
    let mut app = test_app();
    // Navigate right once to select index 1
    press_key(&mut app, KeyCode::ArrowRight);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::ArrowRight);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();

    press_key(&mut app, KeyCode::Enter);

    let received = app.world().resource::<ReceivedChips>();
    assert_eq!(received.0.len(), 1);
    assert_eq!(received.0[0].name, "Wide Breaker");
}

#[test]
fn right_wraps_around() {
    let mut app = test_app();
    // Go right 3 times to wrap around
    for _ in 0..3 {
        press_key(&mut app, KeyCode::ArrowRight);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::ArrowRight);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.chip_index, 0); // wraps back to 0
}

#[test]
fn no_input_no_change() {
    let mut app = test_app();
    app.update();

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.chip_index, 0);

    let msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert_eq!(
        msgs.iter_current_update_messages().count(),
        0,
        "expected no ChangeState message"
    );
}

#[test]
fn empty_offers_confirm_transitions_without_message() {
    let mut app = test_app_with_offers(make_offers(0));
    press_key(&mut app, KeyCode::Enter);

    let msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert!(
        msgs.iter_current_update_messages().count() > 0,
        "expected ChangeState<ChipSelectState> message"
    );

    let received = app.world().resource::<ReceivedChips>();
    assert!(received.0.is_empty(), "expected no ChipSelected messages");
}

#[test]
fn two_card_navigation_wraps_correctly() {
    let mut app = test_app_with_offers(make_offers(2));

    // Right once -> index 1
    press_key(&mut app, KeyCode::ArrowRight);
    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.chip_index, 1);

    // Right again -> wraps to 0
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::ArrowRight);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    press_key(&mut app, KeyCode::ArrowRight);
    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.chip_index, 0);
}

#[test]
fn confirm_records_decay_for_non_selected_chips() {
    // Offers: index 0 = "Piercing Shot", 1 = "Wide Breaker", 2 = "Surge"
    // Selection at index 0 -> confirms "Piercing Shot"
    // Non-selected: "Wide Breaker" and "Surge" should get decay 0.8
    let mut app = test_app();
    press_key(&mut app, KeyCode::Enter);

    let inventory = app.world().resource::<ChipInventory>();

    // Selected chip should NOT have decay applied
    let selected_decay = inventory.weight_decay("Piercing Shot");
    assert!(
        (selected_decay - 1.0).abs() < f32::EPSILON,
        "selected chip 'Piercing Shot' should not have decay, got {selected_decay}"
    );

    // Non-selected chips should have decay = 0.8
    let wb_decay = inventory.weight_decay("Wide Breaker");
    assert!(
        (wb_decay - 0.8).abs() < f32::EPSILON,
        "non-selected 'Wide Breaker' should have decay 0.8, got {wb_decay}"
    );

    let surge_decay = inventory.weight_decay("Surge");
    assert!(
        (surge_decay - 0.8).abs() < f32::EPSILON,
        "non-selected 'Surge' should have decay 0.8, got {surge_decay}"
    );
}

#[test]
fn single_chip_confirm_applies_no_decay() {
    // Only 1 chip offered -- no non-selected chips to decay
    let mut app = test_app_with_offers(make_offers(1));
    press_key(&mut app, KeyCode::Enter);

    let inventory = app.world().resource::<ChipInventory>();

    // The only chip was selected -- no decay should be applied
    let decay = inventory.weight_decay("Piercing Shot");
    assert!(
        (decay - 1.0).abs() < f32::EPSILON,
        "single offered + selected chip should have no decay, got {decay}"
    );
}

// --- Evolution offering tests ---

fn make_evolution_offering() -> ChipOffering {
    ChipOffering::Evolution {
        ingredients: vec![
            EvolutionIngredient {
                chip_name:       "Piercing Shot".to_owned(),
                stacks_required: 2,
            },
            EvolutionIngredient {
                chip_name:       "Damage Up".to_owned(),
                stacks_required: 1,
            },
        ],
        result:      ChipDefinition::test(
            "Barrage",
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 5 })),
            1,
        ),
    }
}

fn test_app_with_evolution_inventory() -> App {
    let offers = ChipOffers(vec![make_evolution_offering()]);
    let mut app = test_app_with_offers(offers);

    // Seed inventory with ingredient stacks
    let ps_def = ChipDefinition::test(
        "Piercing Shot",
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        5,
    )
    .with_template("Piercing Shot");
    let du_def = ChipDefinition::test(
        "Damage Up",
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: ordered_float::OrderedFloat(0.5),
        })),
        5,
    )
    .with_template("Damage Up");
    let mut inventory = app.world_mut().resource_mut::<ChipInventory>();
    let _ = inventory.add_chip("Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Piercing Shot", &ps_def); // 3 stacks
    let _ = inventory.add_chip("Damage Up", &du_def);
    let _ = inventory.add_chip("Damage Up", &du_def); // 2 stacks

    app
}

#[test]
fn confirm_evolution_sends_chip_selected_with_result_name() {
    let mut app = test_app_with_evolution_inventory();
    press_key(&mut app, KeyCode::Enter);

    let received = app.world().resource::<ReceivedChips>();
    assert_eq!(received.0.len(), 1);
    assert_eq!(
        received.0[0].name, "Barrage",
        "evolution confirm should send ChipSelected with the result name"
    );
}

#[test]
fn confirm_evolution_transitions_to_transition_in() {
    let mut app = test_app_with_evolution_inventory();
    press_key(&mut app, KeyCode::Enter);

    let msgs = app
        .world()
        .resource::<Messages<ChangeState<ChipSelectState>>>();
    assert!(
        msgs.iter_current_update_messages().count() > 0,
        "expected ChangeState<ChipSelectState> message after evolution confirm"
    );
}

#[test]
fn confirm_evolution_consumes_ingredient_stacks() {
    // Inventory: "Piercing Shot" at 3, "Damage Up" at 2
    // Evolution requires: "Piercing Shot" x2, "Damage Up" x1
    // After confirm: "Piercing Shot" = 3 - 2 = 1, "Damage Up" = 2 - 1 = 1
    let mut app = test_app_with_evolution_inventory();
    press_key(&mut app, KeyCode::Enter);

    let inventory = app.world().resource::<ChipInventory>();
    assert_eq!(
        inventory.stacks("Piercing Shot"),
        1,
        "Piercing Shot should have 1 stack remaining (3 - 2)"
    );
    assert_eq!(
        inventory.stacks("Damage Up"),
        1,
        "Damage Up should have 1 stack remaining (2 - 1)"
    );
}

#[test]
fn confirm_normal_does_not_consume_ingredient_stacks() {
    // Set up a Normal offering with inventory pre-populated
    let offers = ChipOffers(vec![ChipOffering::Normal(ChipDefinition::test(
        "Piercing Shot",
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        3,
    ))]);
    let mut app = test_app_with_offers(offers);

    // Pre-populate inventory with Piercing Shot at 3 stacks
    let ps_def = ChipDefinition::test(
        "Piercing Shot",
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        5,
    );
    let mut inventory = app.world_mut().resource_mut::<ChipInventory>();
    let _ = inventory.add_chip("Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Piercing Shot", &ps_def);

    press_key(&mut app, KeyCode::Enter);

    let inventory = app.world().resource::<ChipInventory>();
    assert_eq!(
        inventory.stacks("Piercing Shot"),
        3,
        "Normal confirm should NOT consume ingredient stacks"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// Protocol row navigation + confirm tests.
// ─────────────────────────────────────────────────────────────────────────

/// Collect `ProtocolSelected` messages for assertion.
#[derive(Resource, Default)]
struct ReceivedProtocols(Vec<ProtocolSelected>);

fn collect_protocols(
    mut reader: MessageReader<ProtocolSelected>,
    mut received: ResMut<ReceivedProtocols>,
) {
    for msg in reader.read() {
        received.0.push(msg.clone());
    }
}

/// Build a test app with an active `ProtocolOffer(Some(greed))` and the
/// `ReceivedProtocols` collector wired in.
fn test_app_with_protocol_offer(
    offers: ChipOffers,
    protocol: ProtocolOffer,
    selection: ChipSelectSelection,
) -> App {
    let mut app = test_app_with_offers_and_protocol_offer(offers, protocol);
    app.insert_resource(selection);
    app.init_resource::<ReceivedProtocols>();
    app.add_systems(Update, collect_protocols.after(handle_chip_input));
    app
}

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
