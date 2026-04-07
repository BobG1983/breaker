//! `AllCells` desugaring tests — behaviors 1-2.

use bevy::prelude::*;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect::{
        BoundEffects, EffectKind, EffectNode, ImpactTarget, RootEffect, StagedEffects, Target,
        Trigger,
    },
};

// ── Behavior 1: AllCells wraps in When(NodeStart, On(AllCells, ...)) ──

#[test]
fn all_cells_target_desugars_to_when_node_start_on_all_cells() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Cell Fortify".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::AllCells,
            then: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::Shield {
                    duration: 5.0,
                    reflection_cost: 0.0,
                })],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    // Only Breaker exists — no Cells (simulating ChipSelect state)
    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Cell Fortify");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have exactly 1 BoundEffects entry from desugared AllCells"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Cell Fortify", "chip_name must be preserved");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::AllCells,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        then: do_children,
                    } if do_children.len() == 1 && matches!(
                        &do_children[0],
                        EffectNode::Do(EffectKind::Shield { duration: 5.0, reflection_cost: 0.0 })
                    )
                )
            )
        ),
        "Expected When(NodeStart, [On(AllCells, permanent: true, [When(Impacted(Bolt), [Do(Shield(1))])])]), got {node:?}"
    );
}

// ── Behavior 1 edge case: AllCells with multiple children ──

#[test]
fn all_cells_with_multiple_children_all_wrapped_in_single_on_node() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Multi Cell".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::AllCells,
            then: vec![
                EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.1))],
                },
                EffectNode::When {
                    trigger: Trigger::Died,
                    then: vec![EffectNode::Do(EffectKind::Explode {
                        range: 40.0,
                        damage: 1.0,
                    })],
                },
            ],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Multi Cell");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 desugared entry (not 2)"
    );

    let (_, node) = &bound.0[0];
    // Verify the inner On node's `then` has 2 children
    if let EffectNode::When {
        trigger: Trigger::NodeStart,
        then: outer,
    } = node
    {
        assert_eq!(outer.len(), 1, "Should have 1 On child");
        if let EffectNode::On {
            target: Target::AllCells,
            permanent: true,
            then: inner,
        } = &outer[0]
        {
            assert_eq!(
                inner.len(),
                2,
                "Inner On node should have 2 children (both When nodes)"
            );
        } else {
            panic!(
                "Expected On(AllCells, permanent: true, ...), got {:?}",
                outer[0]
            );
        }
    } else {
        panic!("Expected When(NodeStart, ...), got {node:?}");
    }
}

// ── Behavior 2: AllCells desugaring does not dispatch to Cells ──

#[test]
fn all_cells_desugaring_does_not_create_bound_effects_on_cells() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Fortify",
        Target::AllCells,
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield {
                duration: 5.0,
                reflection_cost: 0.0,
            })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    // No cells spawned — simulating ChipSelect state
    select_chip(&mut app, "Cell Fortify");

    app.update();

    // Only Breaker should have BoundEffects modified
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have exactly 1 desugared BoundEffects entry"
    );
}

// ── Behavior 2 edge case: Existing BoundEffects preserved ──

#[test]
fn desugaring_preserves_existing_bound_effects_entries() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Fortify",
        Target::AllCells,
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield {
                duration: 5.0,
                reflection_cost: 0.0,
            })],
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
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::Shield {
                        duration: 5.0,
                        reflection_cost: 0.0,
                    })],
                },
            ),
            (
                "OldChip2".to_owned(),
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                },
            ),
        ]);

        app.world_mut()
            .spawn((
                Breaker,
                existing,
                StagedEffects::default(),
                ActiveBumpForces::default(),
                ActiveSizeBoosts::default(),
            ))
            .id()
    };

    select_chip(&mut app, "Cell Fortify");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        3,
        "Should have 2 old + 1 new desugared = 3 entries"
    );
    assert_eq!(bound.0[0].0, "OldChip1", "first old entry preserved");
    assert_eq!(bound.0[1].0, "OldChip2", "second old entry preserved");
    assert_eq!(bound.0[2].0, "Cell Fortify", "new desugared entry appended");
}
