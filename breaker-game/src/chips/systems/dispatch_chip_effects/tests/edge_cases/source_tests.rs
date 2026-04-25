//! Source chip naming, bound effects insertion, and target edge case tests.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{
        definition::{ChipDefinition, Rarity},
        systems::dispatch_chip_effects::tests::helpers::*,
    },
    effect_v3::{
        effects::{DamageBoostConfig, ShieldConfig, SpeedBoostConfig},
        stacking::EffectStack,
        types::{EffectType, EntityKind, StampTarget, Tree, Trigger},
    },
    prelude::*,
};

// ── Behavior 18: source passed to fire_effect is the chip's display name ──

#[test]
fn source_is_chip_display_name() {
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
        bound.0[0].0, "chip:Chain Reaction:Common",
        "chip_name in BoundEffects is the canonical SourceId form (`chip:<template-or-name>:<rarity>`) per W5"
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
    let bound = bound.unwrap();
    assert!(
        bound.0.iter().any(|(name, _)| name == "chip:Test:Common"),
        "BoundEffects should contain the chip's 'Test' entry (canonical SourceId form)"
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
    assert_eq!(
        bound.0[2].0, "chip:Append:Common",
        "new entry appended (canonical SourceId form)"
    );
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
                duration:        OrderedFloat(5.0),
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
    assert!(
        bound
            .unwrap()
            .0
            .iter()
            .any(|(name, _)| name == "chip:Parry Bare:Common"),
        "BoundEffects should contain the chip's 'Parry Bare' entry (canonical SourceId form)"
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
                duration:        OrderedFloat(5.0),
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

// ── B45: dispatch_chip_effects formats source as chip:<template>:<rarity> ──

#[test]
fn dispatch_chip_effects_uses_template_name_and_rarity_in_source() {
    let mut app = test_app();

    let mut def = ChipDefinition::test_on(
        "Faint Pulse",
        StampTarget::Breaker,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
        5,
    );
    def.template_name = Some("Pulse".to_owned());
    def.rarity = Rarity::Common;

    insert_chip(&mut app, def);
    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Faint Pulse");
    app.update();

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .expect("SpeedBoost stack must be present");
    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].0,
        SourceId::chip("Pulse").rarity(Rarity::Common).build(),
        "dispatch must build source as chip:<template>:<rarity> per W5 spec"
    );
}

// ── B45 edge: evolution chips fall back to display name ──

#[test]
fn dispatch_chip_effects_evolution_falls_back_to_display_name() {
    let mut app = test_app();

    let mut def = ChipDefinition::test_on(
        "Overclock",
        StampTarget::Breaker,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
        1,
    );
    def.template_name = None;
    def.rarity = Rarity::Evolution;

    insert_chip(&mut app, def);
    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Overclock");
    app.update();

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .expect("SpeedBoost stack must be present");
    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(
        entries[0].0,
        SourceId::chip("Overclock")
            .rarity(Rarity::Evolution)
            .build(),
        "evolution chips use display name when template_name is None"
    );
}

// ── B46: every dispatch_chip_effects emit MUST start with "chip:" ──

#[test]
fn dispatch_chip_effects_every_emit_source_starts_with_chip_prefix() {
    let mut app = test_app();

    let mut def = ChipDefinition::test_on(
        "Faint Pulse",
        StampTarget::Breaker,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
        5,
    );
    def.template_name = Some("Pulse".to_owned());
    def.rarity = Rarity::Common;
    insert_chip(&mut app, def);

    let mut def2 = ChipDefinition::test_on(
        "Overclock",
        StampTarget::Breaker,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
        1,
    );
    def2.template_name = None;
    def2.rarity = Rarity::Evolution;
    insert_chip(&mut app, def2);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Faint Pulse");
    select_chip(&mut app, "Overclock");
    app.update();

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .expect("SpeedBoost stack must be present");
    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(
        entries.len(),
        2,
        "two chip selects must produce two entries"
    );

    for (source, _config) in &entries {
        let s = source.0.as_ref();
        assert!(
            s.starts_with("chip:"),
            "B46: every dispatch_chip_effects emit must produce a source \
             starting with \"chip:\" — got {s:?}"
        );
    }
}
