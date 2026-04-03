//! Breaker target dispatch tests — behavior 8.
//!
//! Breaker targets are NOT desugared; they dispatch directly.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    chips::definition::ChipDefinition,
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, Target, Trigger},
};

// ── Behavior 8: Breaker dispatches directly (no desugaring) ──

#[test]
fn breaker_target_dispatches_directly_not_desugared() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Breaker Shield",
        Target::Breaker,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::Shield { duration: 5.0 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Breaker Shield");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1, "Breaker should have 1 BoundEffects entry");

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Breaker Shield");
    // The node should be the direct When(PerfectBump, ...) — NOT wrapped in When(NodeStart, ...)
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then,
            } if then.len() == 1 && matches!(&then[0], EffectNode::Do(EffectKind::Shield { duration: 5.0 }))
        ),
        "Breaker target should NOT be desugared — expected When(PerfectBump, [Do(Shield(1))]), got {node:?}"
    );
}

// ── Behavior 8 edge case: Breaker bare Do fires immediately ──

#[test]
fn breaker_target_bare_do_fires_immediately() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Breaker Size".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::Do(EffectKind::SizeBoost(1.15))],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Breaker Size");

    app.update();

    // SizeBoost should have fired immediately on Breaker
    let sizes = app
        .world()
        .get::<crate::effect::effects::size_boost::ActiveSizeBoosts>(breaker)
        .unwrap();
    assert_eq!(
        sizes.0,
        vec![1.15],
        "SizeBoost(1.15) should fire immediately on Breaker (not desugared)"
    );

    // BoundEffects should be empty (bare Do fires, not pushed)
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        bound.0.is_empty(),
        "BoundEffects should remain empty for bare Do on Breaker target"
    );
}
