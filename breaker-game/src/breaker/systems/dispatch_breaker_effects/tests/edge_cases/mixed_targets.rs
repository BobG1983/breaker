//! Behaviors 15-17: mixed targets, preserving existing effects, inserting absent components.

use bevy::prelude::*;

use super::super::helpers::{TEST_BREAKER_NAME, make_test_definition, test_app_with_dispatch};
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, StagedEffects, Target, Trigger},
};

// ---- Behavior 15: Mixed targets (Aegis-style) ----

#[test]
fn dispatch_handles_mixed_targets_aegis_style() {
    let mut def = make_test_definition(TEST_BREAKER_NAME, Some(3));
    def.effects = vec![
        RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(EffectKind::LoseLife)],
            }],
        },
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
    ];
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    let bolt = app.world_mut().spawn(Bolt).id();
    app.update();

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "breaker should have exactly 1 effect (BoltLost -> LoseLife)"
    );

    let bolt_bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects inserted");
    assert_eq!(
        bolt_bound.0.len(),
        3,
        "bolt should have exactly 3 effects (PerfectBumped, EarlyBumped, LateBumped)"
    );

    assert!(
        app.world().get::<StagedEffects>(bolt).is_some(),
        "bolt should have StagedEffects inserted"
    );
}

// ---- Behavior 16: Preserves existing BoundEffects on target entities ----

#[test]
fn dispatch_preserves_existing_bound_effects_on_target_entities() {
    let mut def = make_test_definition(TEST_BREAKER_NAME, None);
    def.effects = vec![RootEffect::On {
        target: Target::Bolt,
        then: vec![EffectNode::When {
            trigger: Trigger::PerfectBumped,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
    }];
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoundEffects(vec![(
                "existing_chip".to_owned(),
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
            )]),
            StagedEffects(vec![(
                "staged_chip".to_owned(),
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                },
            )]),
        ))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "bolt should have 2 entries (1 existing + 1 dispatched)"
    );
    assert_eq!(
        &bound.0[0].0, "existing_chip",
        "existing entry should be preserved at index 0"
    );
    assert!(matches!(
        &bound.0[0].1,
        EffectNode::Do(EffectKind::DamageBoost(v)) if (*v - 2.0).abs() < f32::EPSILON
    ));

    let staged = app.world().get::<StagedEffects>(bolt).unwrap();
    assert_eq!(
        staged.0.len(),
        1,
        "existing StagedEffects should be preserved, not replaced"
    );
    assert_eq!(
        &staged.0[0].0, "staged_chip",
        "existing staged entry should be preserved"
    );
}

// ---- Behavior 17: Inserts BoundEffects and StagedEffects on entities that lack them ----

#[test]
fn dispatch_inserts_bound_effects_and_staged_effects_when_absent() {
    let mut def = make_test_definition(TEST_BREAKER_NAME, None);
    def.effects = vec![RootEffect::On {
        target: Target::Bolt,
        then: vec![EffectNode::When {
            trigger: Trigger::PerfectBumped,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
    }];
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    // Spawn bolt with no BoundEffects and no StagedEffects
    let bolt = app.world_mut().spawn(Bolt).id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("BoundEffects should be inserted on bolt");
    assert_eq!(bound.0.len(), 1, "bolt should have 1 dispatched entry");

    let staged = app
        .world()
        .get::<StagedEffects>(bolt)
        .expect("StagedEffects should be inserted on bolt");
    assert_eq!(
        staged.0.len(),
        0,
        "newly inserted StagedEffects should be empty"
    );
}

#[test]
fn dispatch_inserts_staged_effects_when_bound_effects_present_but_staged_absent() {
    let mut def = make_test_definition(TEST_BREAKER_NAME, None);
    def.effects = vec![RootEffect::On {
        target: Target::Bolt,
        then: vec![EffectNode::When {
            trigger: Trigger::PerfectBumped,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
    }];
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    // Spawn bolt WITH BoundEffects (containing a prior entry) but WITHOUT StagedEffects
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoundEffects(vec![(
                "prior_chip".to_owned(),
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
                },
            )]),
        ))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("BoundEffects should still be present on bolt");
    assert_eq!(
        bound.0.len(),
        2,
        "bolt should have 2 entries (1 prior + 1 dispatched)"
    );
    assert_eq!(
        &bound.0[0].0, "prior_chip",
        "prior entry should be preserved at index 0"
    );

    assert!(
        app.world().get::<StagedEffects>(bolt).is_some(),
        "StagedEffects should be inserted even though only BoundEffects was present initially"
    );
}
