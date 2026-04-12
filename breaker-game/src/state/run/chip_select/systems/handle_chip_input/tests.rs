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
    shared::test_utils::TestAppBuilder,
    state::run::chip_select::resources::ChipOffering,
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
        .insert_resource(ChipSelectSelection { index: 0 })
        .insert_resource(offers)
        .with_resource::<ReceivedChips>()
        .with_resource::<ChipInventory>()
        .insert_resource(ChipSelectConfig::default())
        .with_message::<ChipSelected>()
        .with_message::<ChangeState<ChipSelectState>>()
        .with_system(Update, (handle_chip_input, collect_chips).chain())
        .build()
}

fn press_key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
}

#[test]
fn right_advances_selection() {
    let mut app = test_app();
    press_key(&mut app, KeyCode::ArrowRight);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.index, 1);
}

#[test]
fn left_wraps_selection() {
    let mut app = test_app();
    press_key(&mut app, KeyCode::ArrowLeft);

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.index, 2); // wraps from 0 to last (2)
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
    assert_eq!(selection.index, 0); // wraps back to 0
}

#[test]
fn no_input_no_change() {
    let mut app = test_app();
    app.update();

    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.index, 0);

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
    assert_eq!(selection.index, 1);

    // Right again -> wraps to 0
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::ArrowRight);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    press_key(&mut app, KeyCode::ArrowRight);
    let selection = app.world().resource::<ChipSelectSelection>();
    assert_eq!(selection.index, 0);
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
