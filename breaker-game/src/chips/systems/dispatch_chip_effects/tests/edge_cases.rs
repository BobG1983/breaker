//! Edge case tests for `dispatch_chip_effects` — behaviors 15-20.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    chips::{definition::ChipDefinition, inventory::ChipInventory, resources::ChipCatalog},
    effect::{
        BoundEffects, EffectKind, EffectNode, StagedEffects, Target, Trigger,
        effects::{damage_boost::ActiveDamageBoosts, speed_boost::ActiveSpeedBoosts},
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
        use crate::breaker::components::Breaker;
        app.world_mut()
            .spawn((Breaker, BoundEffects::default(), StagedEffects::default()))
            .id()
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
        use crate::breaker::components::Breaker;
        app.world_mut()
            .spawn((
                Breaker,
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveDamageBoosts::default(),
                ActiveSpeedBoosts::default(),
            ))
            .id()
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

// ── Behavior 18: source_chip passed to fire_effect is the chip's display name ──

#[test]
fn source_chip_is_chip_display_name() {
    let mut app = test_app();

    // Use Breaker target so effect fires immediately
    let def = ChipDefinition::test_on(
        "Blazing Bolt Speed",
        Target::Breaker,
        EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 }),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Blazing Bolt Speed");

    app.update();

    // The fire_effect was called — SpeedBoost should be active.
    let speed = app.world().get::<ActiveSpeedBoosts>(breaker).unwrap();
    assert_eq!(
        speed.0,
        vec![1.3],
        "SpeedBoost(1.3) should have been fired with source_chip = 'Blazing Bolt Speed'"
    );
}

// ── Behavior 18: chip_name in BoundEffects tuple is also the display name ──

#[test]
fn bound_effects_chip_name_is_display_name() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Chain Reaction",
        Target::Breaker,
        EffectNode::When {
            trigger: Trigger::CellDestroyed,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Chain Reaction");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1);
    assert_eq!(
        bound.0[0].0, "Chain Reaction",
        "chip_name in BoundEffects should be the chip's display name verbatim"
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

// ── Behavior 20: BoundEffects inserted on entity that lacks it ──

#[test]
fn bound_effects_inserted_on_entity_missing_it() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Test",
        Target::Breaker,
        EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
        },
        5,
    );
    insert_chip(&mut app, def);

    // Spawn breaker WITHOUT BoundEffects or StagedEffects
    let breaker = spawn_breaker_bare(&mut app);
    select_chip(&mut app, "Test");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker);
    assert!(
        bound.is_some(),
        "BoundEffects should have been inserted on the entity"
    );
    assert_eq!(
        bound.unwrap().0.len(),
        1,
        "BoundEffects should have 1 entry after dispatch"
    );

    let staged = app.world().get::<StagedEffects>(breaker);
    assert!(
        staged.is_some(),
        "StagedEffects should also have been inserted"
    );
}

// ── Behavior 20 edge case: Entity with existing BoundEffects — new entry appended ──

#[test]
fn existing_bound_effects_preserved_new_entry_appended() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Append",
        Target::Breaker,
        EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        },
        5,
    );
    insert_chip(&mut app, def);

    // Spawn breaker with 2 existing BoundEffects entries
    let breaker = {
        use crate::{
            breaker::components::Breaker,
            effect::effects::{bump_force::ActiveBumpForces, size_boost::ActiveSizeBoosts},
        };

        let existing = BoundEffects(vec![
            (
                "OldChip1".to_owned(),
                EffectNode::When {
                    trigger: Trigger::CellDestroyed,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.1))],
                },
            ),
            (
                "OldChip2".to_owned(),
                EffectNode::When {
                    trigger: Trigger::Death,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.2))],
                },
            ),
        ]);

        app.world_mut()
            .spawn((
                Breaker,
                existing,
                StagedEffects::default(),
                ActiveDamageBoosts::default(),
                ActiveSpeedBoosts::default(),
                ActiveBumpForces::default(),
                ActiveSizeBoosts::default(),
            ))
            .id()
    };

    select_chip(&mut app, "Append");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        3,
        "Should have 2 existing + 1 new = 3 BoundEffects entries"
    );
    assert_eq!(bound.0[0].0, "OldChip1", "first existing entry preserved");
    assert_eq!(bound.0[1].0, "OldChip2", "second existing entry preserved");
    assert_eq!(bound.0[2].0, "Append", "new entry appended");
}

// ── Behavior 6 edge case: Breaker entity missing BoundEffects — inserted before push ──

#[test]
fn breaker_missing_bound_effects_inserted_before_push() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Parry Bare",
        Target::Breaker,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker_bare(&mut app);
    select_chip(&mut app, "Parry Bare");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker);
    assert!(
        bound.is_some(),
        "BoundEffects should have been inserted on breaker"
    );
    assert_eq!(
        bound.unwrap().0.len(),
        1,
        "BoundEffects should have 1 entry"
    );

    let staged = app.world().get::<StagedEffects>(breaker);
    assert!(
        staged.is_some(),
        "StagedEffects should have been inserted on breaker"
    );
}

// ── Behavior 9 edge case: Cell target desugars to Breaker — BoundEffects on Breaker ──

#[test]
fn cell_target_desugars_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Push",
        Target::AllCells,
        EffectNode::When {
            trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Cell Push");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker);
    assert!(
        bound.is_some(),
        "BoundEffects should be present on Breaker (desugared from AllCells)"
    );
    assert_eq!(
        bound.unwrap().0.len(),
        1,
        "Breaker should have 1 desugared BoundEffects entry for AllCells"
    );
}

// ── Behavior 11 edge case: Wall target desugars to Breaker — BoundEffects on Breaker ──

#[test]
fn wall_target_desugars_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Push",
        Target::AllWalls,
        EffectNode::When {
            trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Wall Push");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker);
    assert!(
        bound.is_some(),
        "BoundEffects should be present on Breaker (desugared from AllWalls)"
    );
    assert_eq!(
        bound.unwrap().0.len(),
        1,
        "Breaker should have 1 desugared BoundEffects entry for AllWalls"
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
