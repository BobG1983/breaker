//! Chip inventory and stacking behavior tests for `dispatch_chip_effects`.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{
        definition::ChipDefinition, inventory::ChipInventory, resources::ChipCatalog,
        systems::dispatch_chip_effects::tests::helpers::*,
    },
    effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::BoundEffects,
        types::{EffectType, EntityKind, StampTarget, Tree, Trigger},
    },
};

// ── Behavior 17: System adds chip to ChipInventory on dispatch ──

#[test]
fn chip_added_to_inventory_on_dispatch() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Minor Damage Boost",
        StampTarget::Breaker,
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.1),
        })),
        5,
    );
    insert_chip(&mut app, def);

    let _breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Minor Damage Boost");

    app.update();

    let inventory = app.world().resource::<ChipInventory>();
    assert_eq!(
        inventory.stacks("Minor Damage Boost"),
        1,
        "ChipInventory should have 1 stack of 'Minor Damage Boost' after dispatch"
    );
}

// ── Behavior 17 edge case: Chip at max stacks — effects NOT dispatched ──

#[test]
fn chip_at_max_stacks_does_not_dispatch_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Capped Chip",
        StampTarget::Breaker,
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
        1, // max_stacks = 1
    );
    insert_chip(&mut app, def);

    // Pre-fill inventory to max
    {
        let catalog = app.world().resource::<ChipCatalog>();
        let chip_def = catalog.get("Capped Chip").unwrap().clone();
        let mut inventory = app.world_mut().resource_mut::<ChipInventory>();
        let _ = inventory.add_chip("Capped Chip", &chip_def);
    }

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Capped Chip");

    app.update();

    // Chip is at max stacks — effects should NOT be dispatched
    let stack = app.world().get::<EffectStack<DamageBoostConfig>>(breaker);
    assert!(
        stack.is_none() || stack.unwrap().is_empty(),
        "Effects should NOT be dispatched when chip is at max stacks"
    );
}

// ── Behavior 19: Multiple ChipSelected messages in same frame — all processed ──

#[test]
fn multiple_chip_selections_in_same_frame_all_processed() {
    let mut app = test_app();

    let def_a = ChipDefinition::test_on(
        "Chip A",
        StampTarget::Breaker,
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.1),
        })),
        5,
    );
    let def_b = ChipDefinition::test_on(
        "Chip B",
        StampTarget::Breaker,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.2),
        })),
        5,
    );
    insert_chip(&mut app, def_a);
    insert_chip(&mut app, def_b);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Chip A");
    select_chip(&mut app, "Chip B");

    app.update();

    let damage = app
        .world()
        .get::<EffectStack<DamageBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(
        damage.len(),
        1,
        "Chip A's DamageBoost should have been applied"
    );

    let speed = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(
        speed.len(),
        1,
        "Chip B's SpeedBoost should have been applied"
    );

    let inventory = app.world().resource::<ChipInventory>();
    assert_eq!(inventory.stacks("Chip A"), 1);
    assert_eq!(inventory.stacks("Chip B"), 1);
}

// ── Behavior 19 edge case: Same chip selected twice in one frame ──

#[test]
fn same_chip_selected_twice_in_one_frame_both_processed() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Double Pick",
        StampTarget::Breaker,
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.1),
        })),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Double Pick");
    select_chip(&mut app, "Double Pick");

    app.update();

    let damage = app
        .world()
        .get::<EffectStack<DamageBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(damage.len(), 2, "Both selections should fire DamageBoost");

    let inventory = app.world().resource::<ChipInventory>();
    assert_eq!(inventory.stacks("Double Pick"), 2);
}

// ── Behavior 17 edge case: Inventory starts at 2 stacks, dispatch increments to 3 ──

#[test]
fn inventory_increments_from_existing_stacks() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Stackable Chip",
        StampTarget::Breaker,
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.1),
        })),
        5,
    );
    insert_chip(&mut app, def);

    // Pre-fill inventory to 2 stacks
    {
        let catalog = app.world().resource::<ChipCatalog>();
        let chip_def = catalog.get("Stackable Chip").unwrap().clone();
        let mut inventory = app.world_mut().resource_mut::<ChipInventory>();
        let _ = inventory.add_chip("Stackable Chip", &chip_def);
        let _ = inventory.add_chip("Stackable Chip", &chip_def);
    }

    let _breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Stackable Chip");

    app.update();

    let inventory = app.world().resource::<ChipInventory>();
    assert_eq!(
        inventory.stacks("Stackable Chip"),
        3,
        "ChipInventory should have 3 stacks after dispatch (was 2, added 1)"
    );
}

// ── Behavior 17 edge case: Max stacks with When child — effects NOT dispatched ──

#[test]
fn chip_at_max_stacks_with_when_child_does_not_push_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Maxed When",
        StampTarget::Breaker,
        Tree::When(
            Trigger::DeathOccurred(EntityKind::Cell),
            Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        ),
        1,
    );
    insert_chip(&mut app, def);

    // Pre-fill inventory to max
    {
        let catalog = app.world().resource::<ChipCatalog>();
        let chip_def = catalog.get("Maxed When").unwrap().clone();
        let mut inventory = app.world_mut().resource_mut::<ChipInventory>();
        let _ = inventory.add_chip("Maxed When", &chip_def);
    }

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Maxed When");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        bound.0.is_empty(),
        "When node should NOT be pushed to BoundEffects when chip is at max stacks"
    );
}
