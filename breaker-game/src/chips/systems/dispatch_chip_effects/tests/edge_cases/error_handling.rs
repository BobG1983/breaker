//! Error handling and panic guard tests for `dispatch_chip_effects`.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{
        definition::ChipDefinition, inventory::ChipInventory, resources::ChipCatalog,
        systems::dispatch_chip_effects::tests::helpers::*,
    },
    effect_v3::{
        effects::DamageBoostConfig,
        stacking::EffectStack,
        types::{EffectType, StampTarget, Tree},
    },
    prelude::*,
};

fn damage_fire(multiplier: f32) -> Tree {
    Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
        multiplier: OrderedFloat(multiplier),
    }))
}

// ── Behavior 15: Chip name not found in catalog — silently ignored ──

#[test]
fn unknown_chip_name_does_not_panic() {
    let mut app = test_app();

    let valid_def =
        ChipDefinition::test_on("Valid Chip", StampTarget::Breaker, damage_fire(1.1), 5);
    insert_chip(&mut app, valid_def);

    let breaker = spawn_breaker(&mut app);

    select_chip(&mut app, "Valid Chip");
    select_chip(&mut app, "Unknown Chip");

    app.update();

    let stack = app
        .world()
        .get::<EffectStack<DamageBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 1, "Valid chip should have fired");

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        bound.0.is_empty(),
        "Unknown chip should not have added any BoundEffects"
    );
}

// ── Behavior 15 edge case: ChipCatalog resource missing entirely ──

#[test]
fn missing_chip_catalog_resource_does_not_panic() {
    use crate::chips::systems::dispatch_chip_effects::dispatch_chip_effects;

    // --- First prove the system WORKS with catalog ---
    let mut proof_app = test_app();
    let proof_def = ChipDefinition::test_on("Proof", StampTarget::Breaker, damage_fire(1.1), 5);
    insert_chip(&mut proof_app, proof_def);
    let proof_breaker = spawn_breaker(&mut proof_app);
    select_chip(&mut proof_app, "Proof");
    proof_app.update();

    let stack = proof_app
        .world()
        .get::<EffectStack<DamageBoostConfig>>(proof_breaker)
        .unwrap();
    assert_eq!(stack.len(), 1, "Proof: system works with catalog");

    // --- Now test without catalog ---
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .with_message::<ChipSelected>()
        .with_resource::<ChipInventory>()
        .insert_resource(PendingChipSelections::default())
        .with_system(
            Update,
            (
                send_chip_selections.before(dispatch_chip_effects),
                dispatch_chip_effects,
            ),
        )
        .build();

    let breaker = {
        let entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
        app.world_mut()
            .entity_mut(entity)
            .insert((BoundEffects::default(), StagedEffects::default()));
        entity
    };

    select_chip(&mut app, "Any Chip");
    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(bound.0.is_empty());
}

// ── Behavior 15 edge case: ChipInventory resource missing entirely ──

#[test]
fn missing_chip_inventory_resource_does_not_panic() {
    use crate::chips::systems::dispatch_chip_effects::dispatch_chip_effects;

    // --- First prove the system WORKS with inventory ---
    let mut proof_app = test_app();
    let proof_def = ChipDefinition::test_on("Proof", StampTarget::Breaker, damage_fire(1.1), 5);
    insert_chip(&mut proof_app, proof_def);
    let proof_breaker = spawn_breaker(&mut proof_app);
    select_chip(&mut proof_app, "Proof");
    proof_app.update();

    let stack = proof_app
        .world()
        .get::<EffectStack<DamageBoostConfig>>(proof_breaker)
        .unwrap();
    assert_eq!(stack.len(), 1, "Proof: system works with inventory");

    // --- Now test without inventory ---
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .with_message::<ChipSelected>()
        .with_resource::<ChipCatalog>()
        .insert_resource(PendingChipSelections::default())
        .with_system(
            Update,
            (
                send_chip_selections.before(dispatch_chip_effects),
                dispatch_chip_effects,
            ),
        )
        .build();

    {
        let def = ChipDefinition::test_on("Any Chip", StampTarget::Breaker, damage_fire(1.5), 5);
        app.world_mut().resource_mut::<ChipCatalog>().insert(def);
    }

    let breaker = {
        let entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
        app.world_mut()
            .entity_mut(entity)
            .insert((BoundEffects::default(), StagedEffects::default()));
        entity
    };

    select_chip(&mut app, "Any Chip");
    app.update();

    let stack = app
        .world()
        .get::<EffectStack<DamageBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(
        stack.len(),
        1,
        "DamageBoost should fire even without inventory"
    );
}

// ── Behavior 16: No ChipSelected messages pending — system is a no-op ──

#[test]
fn no_messages_pending_no_entities_modified() {
    let mut app = test_app();

    let def = ChipDefinition::test_on("Unused", StampTarget::Breaker, damage_fire(1.5), 5);
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);

    select_chip(&mut app, "Unused");
    app.update();

    let stack = app
        .world()
        .get::<EffectStack<DamageBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 1, "Proof: system dispatches on message");

    app.world_mut()
        .resource_mut::<PendingChipSelections>()
        .0
        .clear();

    app.update();

    let stack_after = app
        .world()
        .get::<EffectStack<DamageBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(
        stack_after.len(),
        1,
        "No new effects added without messages"
    );

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(bound.0.is_empty());
}

// ── Behavior 15 edge case: Both ChipCatalog AND ChipInventory absent ──

#[test]
fn both_catalog_and_inventory_absent_does_not_panic() {
    use crate::chips::systems::dispatch_chip_effects::dispatch_chip_effects;

    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .with_message::<ChipSelected>()
        .insert_resource(PendingChipSelections::default())
        .with_system(
            Update,
            (
                send_chip_selections.before(dispatch_chip_effects),
                dispatch_chip_effects,
            ),
        )
        .build();

    select_chip(&mut app, "Any Chip");
    app.update();
}
