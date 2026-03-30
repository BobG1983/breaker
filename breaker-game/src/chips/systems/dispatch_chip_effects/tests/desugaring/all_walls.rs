//! `AllWalls` desugaring tests — behavior 3.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    chips::definition::ChipDefinition,
    effect::{BoundEffects, EffectKind, EffectNode, ImpactTarget, Target, Trigger},
};

// ── Behavior 3: AllWalls wraps in When(NodeStart, On(AllWalls, ...)) ──

#[test]
fn all_walls_target_desugars_to_when_node_start_on_all_walls() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Boost",
        Target::AllWalls,
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    // No walls spawned
    select_chip(&mut app, "Wall Boost");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have exactly 1 desugared entry for AllWalls"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Wall Boost");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::AllWalls,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        ..
                    }
                )
            )
        ),
        "Expected When(NodeStart, [On(AllWalls, permanent: true, ...)]), got {node:?}"
    );
}

// ── Behavior 3 edge case: Zero Breakers — no panic ──

#[test]
fn all_walls_with_zero_breakers_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Boost",
        Target::AllWalls,
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    // No breaker, no walls
    select_chip(&mut app, "Wall Boost");

    // Should not panic
    app.update();
}
