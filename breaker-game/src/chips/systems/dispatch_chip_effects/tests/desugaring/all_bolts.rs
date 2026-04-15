//! `ActiveBolts` target dispatch tests — behavior 4.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::{DamageBoostConfig, ShockwaveConfig},
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
    prelude::*,
};

// ── Behavior 4: ActiveBolts stamps tree directly to Breaker's BoundEffects ──

#[test]
fn active_bolts_target_stamps_tree_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Bolt Chain",
        StampTarget::ActiveBolts,
        Tree::When(
            Trigger::PerfectBumped,
            Box::new(Tree::Fire(EffectType::Shockwave(ShockwaveConfig {
                base_range:      OrderedFloat(64.0),
                range_per_level: OrderedFloat(0.0),
                stacks:          1,
                speed:           OrderedFloat(500.0),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    // No bolts spawned
    select_chip(&mut app, "Bolt Chain");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have exactly 1 stamped entry for ActiveBolts"
    );

    let (chip_name, tree) = &bound.0[0];
    assert_eq!(chip_name, "Bolt Chain");
    assert!(
        matches!(tree, Tree::When(Trigger::PerfectBumped, _)),
        "Tree should be stamped directly as When(PerfectBumped, ...), got {tree:?}"
    );
}

// ── Behavior 4 edge case: ActiveBolts with bare Fire — Fire is deferred, not fired ──

#[test]
fn active_bolts_with_fire_child_deferred_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition {
        name:          "Bolt Damage".to_owned(),
        description:   String::new(),
        rarity:        crate::chips::definition::Rarity::Common,
        max_stacks:    5,
        effects:       vec![RootNode::Stamp(
            StampTarget::ActiveBolts,
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(1.3),
            })),
        )],
        ingredients:   None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    // No bolts exist — simulate ChipSelect
    select_chip(&mut app, "Bolt Damage");

    app.update();

    // The Fire tree should be stamped to Breaker for deferred dispatch
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 stamped entry for ActiveBolts Fire"
    );

    let (_, tree) = &bound.0[0];
    assert!(
        matches!(tree, Tree::Fire(EffectType::DamageBoost(_))),
        "Fire tree should be stamped directly to breaker, got {tree:?}"
    );
}
