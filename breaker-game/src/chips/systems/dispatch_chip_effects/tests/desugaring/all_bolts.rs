//! `AllBolts` desugaring tests — behavior 4.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    chips::definition::ChipDefinition,
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, Target, Trigger},
};

// ── Behavior 4: AllBolts wraps in When(NodeStart, On(AllBolts, ...)) ──

#[test]
fn all_bolts_target_desugars_to_when_node_start_on_all_bolts() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Bolt Chain",
        Target::AllBolts,
        EffectNode::When {
            trigger: Trigger::PerfectBumped,
            then: vec![EffectNode::Do(EffectKind::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 500.0,
            })],
        },
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
        "Breaker should have exactly 1 desugared entry for AllBolts"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Bolt Chain");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::AllBolts,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::When {
                        trigger: Trigger::PerfectBumped,
                        ..
                    }
                )
            )
        ),
        "Expected When(NodeStart, [On(AllBolts, permanent: true, ...)]), got {node:?}"
    );
}

// ── Behavior 4 edge case: AllBolts with bare Do — Do is deferred, not fired ──

#[test]
fn all_bolts_with_bare_do_child_deferred_not_fired_immediately() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Bolt Damage".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::AllBolts,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.3))],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    // No bolts exist — simulate ChipSelect
    select_chip(&mut app, "Bolt Damage");

    app.update();

    // The bare Do should NOT fire immediately — it should be wrapped in desugaring
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 desugared entry (bare Do wrapped in When(NodeStart, On(...)))"
    );

    // Verify the Do is inside the desugared tree
    let (_, node) = &bound.0[0];
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::AllBolts,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::Do(EffectKind::DamageBoost(v)) if (*v - 1.3).abs() < f32::EPSILON
                )
            )
        ),
        "Bare Do should be wrapped in When(NodeStart, [On(AllBolts, [Do(DamageBoost(1.3))])]), got {node:?}"
    );
}
