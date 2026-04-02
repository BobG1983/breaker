//! Error handling and panic guard tests for `dispatch_chip_effects`.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    chips::{definition::ChipDefinition, inventory::ChipInventory, resources::ChipCatalog},
    effect::{
        BoundEffects, EffectKind, EffectNode, StagedEffects, Target,
        effects::damage_boost::ActiveDamageBoosts,
    },
};

// ── Behavior 15: Chip name not found in catalog — silently ignored ──

#[test]
fn unknown_chip_name_does_not_panic() {
    let mut app = test_app();

    // Insert a valid chip targeting Breaker so dispatch fires immediately
    let valid_def = ChipDefinition::test_on(
        "Valid Chip",
        Target::Breaker,
        EffectNode::Do(EffectKind::DamageBoost(1.1)),
        5,
    );
    insert_chip(&mut app, valid_def);

    let breaker = spawn_breaker(&mut app);

    // Send both a valid AND an unknown chip selection in the same frame
    select_chip(&mut app, "Valid Chip");
    select_chip(&mut app, "Unknown Chip");

    // Should not panic; unknown chip is silently ignored
    app.update();

    // The valid chip should have fired — proves the system actually ran
    let damage = app.world().get::<ActiveDamageBoosts>(breaker).unwrap();
    assert_eq!(
        damage.0,
        vec![1.1],
        "Valid chip's DamageBoost(1.1) should have fired, proving the system ran"
    );

    // The unknown chip should NOT have added any extra BoundEffects entries
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        bound.0.is_empty(),
        "Unknown chip should not have added any BoundEffects entries"
    );
}

// ── Behavior 15 edge case: ChipCatalog resource missing entirely ──

#[test]
fn missing_chip_catalog_resource_does_not_panic() {
    use crate::{
        chips::systems::dispatch_chip_effects::dispatch_chip_effects, shared::GameState,
        ui::messages::ChipSelected,
    };

    // --- First prove the system WORKS with catalog ---
    let mut proof_app = test_app();
    let proof_def = ChipDefinition::test_on(
        "Proof",
        Target::Breaker,
        EffectNode::Do(EffectKind::DamageBoost(1.1)),
        5,
    );
    insert_chip(&mut proof_app, proof_def);
    let proof_breaker = spawn_breaker(&mut proof_app);
    select_chip(&mut proof_app, "Proof");
    proof_app.update();

    let proof_damage = proof_app
        .world()
        .get::<ActiveDamageBoosts>(proof_breaker)
        .unwrap();
    assert_eq!(
        proof_damage.0,
        vec![1.1],
        "Proof: system works with catalog present"
    );

    // --- Now test without catalog ---
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .init_state::<GameState>()
        .add_message::<ChipSelected>()
        .init_resource::<ChipInventory>()
        // Deliberately NOT inserting ChipCatalog
        .init_resource::<PendingChipSelections>()
        .add_systems(
            Update,
            (
                send_chip_selections.before(dispatch_chip_effects),
                dispatch_chip_effects,
            ),
        );

    let mut next_state = app.world_mut().resource_mut::<NextState<GameState>>();
    next_state.set(GameState::ChipSelect);
    app.update();

    // Spawn a breaker and send a message — system should handle missing catalog gracefully
    let breaker = {
        use crate::breaker::{components::Breaker, definition::BreakerDefinition};
        let def = BreakerDefinition::default();
        let entity = app
            .world_mut()
            .spawn(
                Breaker::builder()
                    .definition(&def)
                    .headless()
                    .primary()
                    .build(),
            )
            .id();
        app.world_mut()
            .entity_mut(entity)
            .insert((BoundEffects::default(), StagedEffects::default()));
        entity
    };

    select_chip(&mut app, "Any Chip");

    // Should not panic
    app.update();

    // Breaker should have no new BoundEffects entries since catalog was missing
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        bound.0.is_empty(),
        "Without ChipCatalog, no BoundEffects should be added on breaker"
    );
}

// ── Behavior 15 edge case: ChipInventory resource missing entirely ──

#[test]
fn missing_chip_inventory_resource_does_not_panic() {
    use crate::{
        chips::systems::dispatch_chip_effects::dispatch_chip_effects, shared::GameState,
        ui::messages::ChipSelected,
    };

    // --- First prove the system WORKS with inventory ---
    let mut proof_app = test_app();
    let proof_def = ChipDefinition::test_on(
        "Proof",
        Target::Breaker,
        EffectNode::Do(EffectKind::DamageBoost(1.1)),
        5,
    );
    insert_chip(&mut proof_app, proof_def);
    let proof_breaker = spawn_breaker(&mut proof_app);
    select_chip(&mut proof_app, "Proof");
    proof_app.update();

    let proof_damage = proof_app
        .world()
        .get::<ActiveDamageBoosts>(proof_breaker)
        .unwrap();
    assert_eq!(
        proof_damage.0,
        vec![1.1],
        "Proof: system works with inventory present"
    );

    // --- Now test without inventory ---
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .init_state::<GameState>()
        .add_message::<ChipSelected>()
        .init_resource::<ChipCatalog>()
        // Deliberately NOT inserting ChipInventory
        .init_resource::<PendingChipSelections>()
        .add_systems(
            Update,
            (
                send_chip_selections.before(dispatch_chip_effects),
                dispatch_chip_effects,
            ),
        );

    // Insert chip into catalog so lookup succeeds
    {
        let def = ChipDefinition::test_on(
            "Any Chip",
            Target::Breaker,
            EffectNode::Do(EffectKind::DamageBoost(1.5)),
            5,
        );
        app.world_mut().resource_mut::<ChipCatalog>().insert(def);
    }

    let mut next_state = app.world_mut().resource_mut::<NextState<GameState>>();
    next_state.set(GameState::ChipSelect);
    app.update();

    // Spawn a breaker so dispatch has targets
    let breaker = {
        use crate::{
            breaker::{components::Breaker, definition::BreakerDefinition},
            effect::effects::speed_boost::ActiveSpeedBoosts,
        };
        let def = BreakerDefinition::default();
        let entity = app
            .world_mut()
            .spawn(
                Breaker::builder()
                    .definition(&def)
                    .headless()
                    .primary()
                    .build(),
            )
            .id();
        app.world_mut().entity_mut(entity).insert((
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveDamageBoosts::default(),
            ActiveSpeedBoosts::default(),
        ));
        entity
    };

    select_chip(&mut app, "Any Chip");

    // Should not panic even without ChipInventory
    app.update();

    // Effects should still dispatch even without inventory
    let damage = app.world().get::<ActiveDamageBoosts>(breaker).unwrap();
    assert_eq!(
        damage.0,
        vec![1.5],
        "DamageBoost should still fire even without ChipInventory"
    );
}

// ── Behavior 16: No ChipSelected messages pending — system is a no-op ──

#[test]
fn no_messages_pending_no_entities_modified() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Unused",
        Target::Breaker,
        EffectNode::Do(EffectKind::DamageBoost(1.5)),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);

    // --- First prove the system WORKS by sending a message ---
    select_chip(&mut app, "Unused");
    app.update();

    let damage = app.world().get::<ActiveDamageBoosts>(breaker).unwrap();
    assert_eq!(
        damage.0,
        vec![1.5],
        "Proof: system dispatches when a message is sent"
    );

    // --- Now test the no-message case in a fresh round ---
    // Clear the pending selections (no new messages)
    app.world_mut()
        .resource_mut::<PendingChipSelections>()
        .0
        .clear();

    // Another update with NO messages
    app.update();

    // Damage should still be [1.5] from the first round — no new effects added
    let damage_after = app.world().get::<ActiveDamageBoosts>(breaker).unwrap();
    assert_eq!(
        damage_after.0,
        vec![1.5],
        "No messages sent in second update — damage should remain [1.5], not grow"
    );

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        bound.0.is_empty(),
        "No messages sent — BoundEffects should remain empty"
    );
}

// ── Behavior 15 edge case: Both ChipCatalog AND ChipInventory absent ──

#[test]
fn both_catalog_and_inventory_absent_does_not_panic() {
    use crate::{
        chips::systems::dispatch_chip_effects::dispatch_chip_effects, shared::GameState,
        ui::messages::ChipSelected,
    };

    // --- First prove the system WORKS with both resources ---
    let mut proof_app = test_app();
    let proof_def = ChipDefinition::test_on(
        "Proof",
        Target::Breaker,
        EffectNode::Do(EffectKind::DamageBoost(1.1)),
        5,
    );
    insert_chip(&mut proof_app, proof_def);
    let proof_breaker = spawn_breaker(&mut proof_app);
    select_chip(&mut proof_app, "Proof");
    proof_app.update();

    let proof_damage = proof_app
        .world()
        .get::<ActiveDamageBoosts>(proof_breaker)
        .unwrap();
    assert_eq!(
        proof_damage.0,
        vec![1.1],
        "Proof: system works with both resources present"
    );

    // --- Now test without EITHER resource ---
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .init_state::<GameState>()
        .add_message::<ChipSelected>()
        // Deliberately NOT inserting ChipCatalog or ChipInventory
        .init_resource::<PendingChipSelections>()
        .add_systems(
            Update,
            (
                send_chip_selections.before(dispatch_chip_effects),
                dispatch_chip_effects,
            ),
        );

    let mut next_state = app.world_mut().resource_mut::<NextState<GameState>>();
    next_state.set(GameState::ChipSelect);
    app.update();

    select_chip(&mut app, "Any Chip");

    // Should not panic
    app.update();
}
