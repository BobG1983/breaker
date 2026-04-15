//! `ActiveWalls` target dispatch tests — behavior 3.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::SpeedBoostConfig,
        types::{EffectType, EntityKind, StampTarget, Tree, Trigger},
    },
    prelude::*,
};

// ── Behavior 3: ActiveWalls stamps tree to Breaker's BoundEffects ──

#[test]
fn active_walls_target_stamps_tree_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Boost",
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
    select_chip(&mut app, "Wall Boost");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 stamped entry for ActiveWalls"
    );

    let (chip_name, tree) = &bound.0[0];
    assert_eq!(chip_name, "Wall Boost");
    assert!(
        matches!(tree, Tree::When(Trigger::Impacted(EntityKind::Bolt), _)),
        "Tree should be stamped directly, got {tree:?}"
    );
}

// ── Behavior 3 edge case: Zero Breaker entities ──

#[test]
fn active_walls_target_with_zero_breakers_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Empty",
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

    select_chip(&mut app, "Wall Empty");
    app.update();
}
