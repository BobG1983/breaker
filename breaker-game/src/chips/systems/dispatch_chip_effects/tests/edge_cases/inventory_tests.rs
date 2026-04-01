//! Chip inventory and stacking behavior tests for `dispatch_chip_effects`.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    chips::{definition::ChipDefinition, inventory::ChipInventory, resources::ChipCatalog},
    effect::{
        BoundEffects, EffectKind, EffectNode, Target, Trigger,
        effects::{damage_boost::ActiveDamageBoosts, speed_boost::ActiveSpeedBoosts},
    },
};

// ── Behavior 17: System adds chip to ChipInventory on dispatch ──

#[test]
fn chip_added_to_inventory_on_dispatch() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Minor Damage Boost",
        Target::Breaker,
        EffectNode::Do(EffectKind::DamageBoost(1.1)),
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
        Target::Breaker,
        EffectNode::Do(EffectKind::DamageBoost(1.5)),
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
    let damage = app.world().get::<ActiveDamageBoosts>(breaker).unwrap();
    assert!(
        damage.0.is_empty(),
        "Effects should NOT be dispatched when chip is at max stacks, got {:?}",
        damage.0
    );
}

// ── Behavior 19: Multiple ChipSelected messages in same frame — all processed ──

#[test]
fn multiple_chip_selections_in_same_frame_all_processed() {
    let mut app = test_app();

    let def_a = ChipDefinition::test_on(
        "Chip A",
        Target::Breaker,
        EffectNode::Do(EffectKind::DamageBoost(1.1)),
        5,
    );
    let def_b = ChipDefinition::test_on(
        "Chip B",
        Target::Breaker,
        EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 }),
        5,
    );
    insert_chip(&mut app, def_a);
    insert_chip(&mut app, def_b);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Chip A");
    select_chip(&mut app, "Chip B");

    app.update();

    let damage = app.world().get::<ActiveDamageBoosts>(breaker).unwrap();
    assert_eq!(
        damage.0,
        vec![1.1],
        "Chip A's DamageBoost(1.1) should have been applied"
    );

    let speed = app.world().get::<ActiveSpeedBoosts>(breaker).unwrap();
    assert_eq!(
        speed.0,
        vec![1.2],
        "Chip B's SpeedBoost(1.2) should have been applied"
    );

    let inventory = app.world().resource::<ChipInventory>();
    assert_eq!(
        inventory.stacks("Chip A"),
        1,
        "Chip A should have 1 stack in inventory"
    );
    assert_eq!(
        inventory.stacks("Chip B"),
        1,
        "Chip B should have 1 stack in inventory"
    );
}

// ── Behavior 19 edge case: Same chip selected twice in one frame ──

#[test]
fn same_chip_selected_twice_in_one_frame_both_processed() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Double Pick",
        Target::Breaker,
        EffectNode::Do(EffectKind::DamageBoost(1.1)),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Double Pick");
    select_chip(&mut app, "Double Pick");

    app.update();

    let damage = app.world().get::<ActiveDamageBoosts>(breaker).unwrap();
    assert_eq!(
        damage.0,
        vec![1.1, 1.1],
        "Both selections should fire DamageBoost, resulting in [1.1, 1.1]"
    );

    let inventory = app.world().resource::<ChipInventory>();
    assert_eq!(
        inventory.stacks("Double Pick"),
        2,
        "Inventory should show 2 stacks of 'Double Pick'"
    );
}

// ── Behavior 17 edge case: Inventory starts at 2 stacks, dispatch increments to 3 ──

#[test]
fn inventory_increments_from_existing_stacks() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Stackable Chip",
        Target::Breaker,
        EffectNode::Do(EffectKind::DamageBoost(1.1)),
        5, // max_stacks = 5
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

    // Verify precondition: 2 stacks
    let pre_stacks = app
        .world()
        .resource::<ChipInventory>()
        .stacks("Stackable Chip");
    assert_eq!(
        pre_stacks, 2,
        "Precondition: should have 2 stacks before dispatch"
    );

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
        Target::Breaker,
        EffectNode::When {
            trigger: Trigger::CellDestroyed,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
        },
        1, // max_stacks = 1
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

    // Chip is at max stacks — When node should NOT be pushed to BoundEffects
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        bound.0.is_empty(),
        "When node should NOT be pushed to BoundEffects when chip is at max stacks, got {} entries",
        bound.0.len()
    );
}
