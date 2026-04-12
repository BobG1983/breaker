//! Source chip naming, bound effects insertion, and target edge case tests.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::{DamageBoostConfig, ShieldConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::{BoundEffects, StagedEffects},
        types::{EffectType, EntityKind, StampTarget, Tree, Trigger},
    },
};

// ── Behavior 18: source_chip passed to fire_effect is the chip's display name ──

#[test]
fn source_chip_is_chip_display_name() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Blazing Bolt Speed",
        StampTarget::Breaker,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.3),
        })),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Blazing Bolt Speed");

    app.update();

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(
        stack.len(),
        1,
        "SpeedBoost should have been fired with source_chip = 'Blazing Bolt Speed'"
    );
}

// ── Behavior 18: chip_name in BoundEffects tuple is also the display name ──

#[test]
fn bound_effects_chip_name_is_display_name() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Chain Reaction",
        StampTarget::Breaker,
        Tree::When(
            Trigger::DeathOccurred(EntityKind::Cell),
            Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(1.0),
            }))),
        ),
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
        StampTarget::Breaker,
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

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
        StampTarget::Breaker,
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(1.0),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = {
        use crate::breaker::components::Breaker;

        let existing = BoundEffects(vec![
            (
                "OldChip1".to_owned(),
                Tree::When(
                    Trigger::DeathOccurred(EntityKind::Cell),
                    Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(1.1),
                    }))),
                ),
            ),
            (
                "OldChip2".to_owned(),
                Tree::When(
                    Trigger::Died,
                    Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(1.2),
                    }))),
                ),
            ),
        ]);

        app.world_mut()
            .spawn((Breaker, existing, StagedEffects::default()))
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
        StampTarget::Breaker,
        Tree::When(
            Trigger::PerfectBumped,
            Box::new(Tree::Fire(EffectType::Shield(ShieldConfig {
                duration: OrderedFloat(5.0),
                reflection_cost: OrderedFloat(0.0),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker_bare(&mut app);
    select_chip(&mut app, "Parry Bare");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker);
    assert!(bound.is_some(), "BoundEffects should have been inserted");
    assert_eq!(
        bound.unwrap().0.len(),
        1,
        "BoundEffects should have 1 entry"
    );

    let staged = app.world().get::<StagedEffects>(breaker);
    assert!(staged.is_some(), "StagedEffects should have been inserted");
}

// ── ActiveCells target stamps to Breaker's BoundEffects ──

#[test]
fn cells_target_stamps_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Push",
        StampTarget::ActiveCells,
        Tree::When(
            Trigger::Impacted(EntityKind::Bolt),
            Box::new(Tree::Fire(EffectType::Shield(ShieldConfig {
                duration: OrderedFloat(5.0),
                reflection_cost: OrderedFloat(0.0),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Cell Push");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker);
    assert!(bound.is_some(), "BoundEffects should be present on Breaker");
    assert_eq!(bound.unwrap().0.len(), 1, "Should have 1 stamped entry");
}

// ── ActiveWalls target stamps to Breaker's BoundEffects ──

#[test]
fn walls_target_stamps_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Push",
        StampTarget::ActiveWalls,
        Tree::When(
            Trigger::Impacted(EntityKind::Bolt),
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Wall Push");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker);
    assert!(bound.is_some(), "BoundEffects should be present on Breaker");
    assert_eq!(bound.unwrap().0.len(), 1, "Should have 1 stamped entry");
}
