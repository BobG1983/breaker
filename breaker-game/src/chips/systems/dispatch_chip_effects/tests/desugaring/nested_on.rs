//! Nested On target tests — behavior 9.
//!
//! Nested On nodes inside a Breaker root are NOT desugared; they resolve
//! directly against existing entities.

use bevy::prelude::*;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect::{BoundEffects, EffectKind, EffectNode, ImpactTarget, RootEffect, Target, Trigger},
};

// ── Behavior 9: Nested On(Bolt) inside Breaker root is not desugared ──

#[test]
fn nested_on_bolt_inside_breaker_root_not_desugared() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Nested Bolt".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::On {
                target: Target::Bolt,
                permanent: false,
                then: vec![EffectNode::When {
                    trigger: Trigger::Bumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
                }],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let _breaker = spawn_breaker(&mut app);
    let bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Nested Bolt");

    app.update();

    // The nested On(Bolt) should resolve against the existing Bolt entity
    let bolt_bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bolt_bound.0.len(),
        1,
        "Bolt should have 1 BoundEffects entry from nested On resolution"
    );
    assert_eq!(bolt_bound.0[0].0, "Nested Bolt");
    assert!(
        matches!(
            &bolt_bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::Bumped,
                ..
            }
        ),
        "Bolt's entry should be the inner When(Bumped, ...), not a desugared When(NodeStart, ...)"
    );
}

// ── Behavior 9 edge case: Nested On(AllCells) inside Breaker root with Cells present ──

#[test]
fn nested_on_all_cells_inside_breaker_root_resolves_directly() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Nested Cells".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::On {
                target: Target::AllCells,
                permanent: false,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::Shield {
                        duration: 5.0,
                        reflection_cost: 0.0,
                    })],
                }],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let _breaker = spawn_breaker(&mut app);
    let cell = spawn_cell(&mut app);
    select_chip(&mut app, "Nested Cells");

    app.update();

    // Nested On(AllCells) resolves directly (not desugared) because it's inside a Breaker root
    let cell_bound = app.world().get::<BoundEffects>(cell).unwrap();
    assert_eq!(
        cell_bound.0.len(),
        1,
        "Cell should have 1 BoundEffects entry from nested On(AllCells) direct resolution"
    );
}
