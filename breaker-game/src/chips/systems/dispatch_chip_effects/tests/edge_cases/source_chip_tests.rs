//! Source chip naming, bound effects insertion, and target desugaring edge case tests.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    chips::definition::ChipDefinition,
    effect::{
        BoundEffects, EffectKind, EffectNode, StagedEffects, Target, Trigger,
        effects::{damage_boost::ActiveDamageBoosts, speed_boost::ActiveSpeedBoosts},
    },
};

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
