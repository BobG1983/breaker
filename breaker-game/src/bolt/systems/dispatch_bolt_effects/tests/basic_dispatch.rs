//! Behaviors 1-4, 14-16: basic bolt effect dispatch.

use bevy::prelude::*;

use super::helpers::{TEST_BOLT_NAME, test_app_with_dispatch};
use crate::{
    bolt::{
        components::{Bolt, BoltDefinitionRef},
        definition::BoltDefinition,
    },
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, Target, Trigger},
};

/// Helper: creates a minimal `BoltDefinition` with the given effects.
fn make_bolt_def(name: &str, effects: Vec<RootEffect>) -> BoltDefinition {
    BoltDefinition {
        name: name.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects,
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    }
}

// ---- Behavior 1: Bolt-targeted When child pushed to bolt BoundEffects on spawn ----

#[test]
fn dispatch_pushes_bolt_targeted_when_to_bolt_bound_effects() {
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects inserted");
    assert_eq!(bound.0.len(), 1, "expected 1 effect in BoundEffects");
    assert_eq!(
        &bound.0[0].0, "",
        "chip name should be empty string for bolt-definition-sourced effects"
    );
    assert!(matches!(
        &bound.0[0].1,
        EffectNode::When { trigger: Trigger::PerfectBumped, then } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 1.5).abs() < f32::EPSILON)
    ));
}

#[test]
fn dispatch_preserves_pre_existing_bound_effects_on_bolt() {
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
            BoundEffects(vec![(
                "prior_chip".to_owned(),
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
            )]),
        ))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "expected 2 entries (1 pre-existing + 1 dispatched)"
    );
    assert_eq!(
        &bound.0[0].0, "prior_chip",
        "pre-existing entry should be preserved at index 0"
    );
}

// ---- Behavior 2: Empty effects definition produces no BoundEffects entries ----

#[test]
fn dispatch_empty_effects_does_not_add_entries() {
    let def = make_bolt_def("PlainBolt", vec![]);
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef("PlainBolt".to_owned()),
            BoundEffects::default(),
        ))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        0,
        "empty effects definition should leave BoundEffects empty"
    );
}

#[test]
fn dispatch_empty_effects_preserves_pre_existing_entries() {
    let def = make_bolt_def("PlainBolt", vec![]);
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef("PlainBolt".to_owned()),
            BoundEffects(vec![(
                "existing_chip".to_owned(),
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
            )]),
        ))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "pre-existing entry should remain when effects are empty"
    );
    assert_eq!(&bound.0[0].0, "existing_chip");
}

// ---- Behavior 3: Bare Do children fired immediately, not stored in BoundEffects ----

#[test]
fn dispatch_fires_bare_do_immediately_not_stored_in_bound_effects() {
    let def = make_bolt_def(
        "EffectBolt",
        vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
                EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
            ],
        }],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef("EffectBolt".to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects inserted");
    assert_eq!(
        bound.0.len(),
        1,
        "only the When child should be stored; the bare Do should be fired immediately"
    );
    assert!(
        matches!(
            &bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::PerfectBumped,
                ..
            }
        ),
        "the stored entry should be the When node, not the Do"
    );
}

#[test]
fn dispatch_all_bare_do_children_results_in_no_bound_entries() {
    let def = make_bolt_def(
        "AllDoBolt",
        vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![
                EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
                EffectNode::Do(EffectKind::DamageBoost(3.0)),
            ],
        }],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef("AllDoBolt".to_owned())))
        .id();
    app.update();

    // BoundEffects may or may not be inserted, but if present it should have 0 dispatched entries
    if let Some(bound) = app.world().get::<BoundEffects>(bolt) {
        // All entries should be chip-sourced or definition-sourced non-Do; since only Do children,
        // no definition-sourced entries should be pushed
        let def_entry_count = bound.0.iter().filter(|(name, _)| name.is_empty()).count();
        assert_eq!(
            def_entry_count, 0,
            "all Do children should be fired immediately, not stored"
        );
    }
}

// ---- Behavior 4: Multiple root effects on same bolt all dispatched ----

#[test]
fn dispatch_pushes_multiple_root_effects_on_same_bolt() {
    let def = make_bolt_def(
        "MultiBolt",
        vec![
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::EarlyBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::LateBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                }],
            },
        ],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef("MultiBolt".to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects");
    assert_eq!(bound.0.len(), 3, "expected 3 effects in BoundEffects");
}

#[test]
fn dispatch_zero_root_effects_pushes_nothing() {
    let def = make_bolt_def("EmptyBolt", vec![]);
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef("EmptyBolt".to_owned())))
        .id();
    app.update();

    // No BoundEffects should be added, or if present, 0 entries
    if let Some(bound) = app.world().get::<BoundEffects>(bolt) {
        assert_eq!(
            bound.0.len(),
            0,
            "zero root effects should result in zero entries"
        );
    }
}

// ---- Behavior 14: Empty string used as chip name for all dispatched effects ----

#[test]
fn dispatch_uses_empty_string_as_chip_name() {
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects");
    for (i, (chip_name, _)) in bound.0.iter().enumerate() {
        assert_eq!(
            chip_name, "",
            "entry {i} chip name should be empty string for bolt-definition-sourced effects"
        );
    }
}

// ---- Behavior 15: All children of a single On node are dispatched ----

#[test]
fn dispatch_pushes_all_children_of_single_on_node() {
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![
                EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
                EffectNode::When {
                    trigger: Trigger::EarlyBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                },
            ],
        }],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        2,
        "both When children of the single On node should be pushed"
    );
    assert!(matches!(
        &bound.0[0].1,
        EffectNode::When {
            trigger: Trigger::PerfectBumped,
            ..
        }
    ));
    assert!(matches!(
        &bound.0[1].1,
        EffectNode::When {
            trigger: Trigger::EarlyBumped,
            ..
        }
    ));
}

#[test]
fn dispatch_on_node_with_zero_children_pushes_nothing() {
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![],
        }],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();
    app.update();

    if let Some(bound) = app.world().get::<BoundEffects>(bolt) {
        assert_eq!(
            bound.0.len(),
            0,
            "On node with 0 children should push nothing"
        );
    }
}

// ---- Behavior 16: System only triggers on Added<BoltDefinitionRef>, not on every frame ----

#[test]
fn dispatch_only_triggers_on_added_bolt_definition_ref() {
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();

    // First update: Added<BoltDefinitionRef> triggers dispatch
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects after first update");
    let count_after_first = bound.0.len();

    // Second update: no new Added<BoltDefinitionRef> -- should not add more entries
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        count_after_first,
        "no additional entries should be added on subsequent frames"
    );
}

#[test]
fn dispatch_second_bolt_on_later_frame_triggers_new_dispatch() {
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    );
    let mut app = test_app_with_dispatch(def);
    let _bolt1 = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();

    // First update: dispatch for bolt1 (Added triggers)
    app.update();

    // Spawn bolt2 on a later frame
    let bolt2 = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();

    app.update();

    // bolt2 should get effects from its own dispatch
    let bound2 = app
        .world()
        .get::<BoundEffects>(bolt2)
        .expect("bolt2 should have BoundEffects after its spawn frame");
    assert!(
        !bound2.0.is_empty(),
        "bolt2 should have at least 1 entry from its dispatch"
    );

    // After two more frames with no new Added<BoltDefinitionRef>,
    // no further entries should be added
    let bolt2_count = bound2.0.len();
    app.update();

    let bound2_after = app.world().get::<BoundEffects>(bolt2).unwrap();
    assert_eq!(
        bound2_after.0.len(),
        bolt2_count,
        "no additional entries should be added on frames without new Added<BoltDefinitionRef>"
    );
}
