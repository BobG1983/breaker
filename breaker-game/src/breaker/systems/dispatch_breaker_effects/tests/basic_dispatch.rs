use bevy::prelude::*;

use super::helpers::{TEST_BREAKER_NAME, test_app_with_dispatch};
use crate::{
    breaker::{
        components::Breaker,
        definition::{BreakerDefinition, BreakerStatOverrides},
    },
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, Target, Trigger},
};

// ---- Behavior 6: Breaker-targeted When children pushed to breaker BoundEffects ----

#[test]
fn dispatch_pushes_breaker_targeted_when_to_breaker_bound_effects() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(EffectKind::LoseLife)],
            }],
        }],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1, "expected 1 effect in BoundEffects");
    assert_eq!(
        &bound.0[0].0, "",
        "chip name should be empty string for breaker-defined effects"
    );
    assert!(matches!(
        &bound.0[0].1,
        EffectNode::When { trigger: Trigger::BoltLost, then } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::LoseLife))
    ));
}

#[test]
fn dispatch_empty_effects_leaves_bound_effects_empty() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        0,
        "empty effects definition should leave BoundEffects empty"
    );
}

// ---- Behavior 7: Bare Do children fired immediately, not stored in BoundEffects ----

#[test]
fn dispatch_fires_bare_do_immediately_not_stored_in_bound_effects() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
                EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                },
            ],
        }],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "only the When child should be stored; the bare Do should be fired immediately"
    );
    assert!(
        matches!(
            &bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::BoltLost,
                ..
            }
        ),
        "the stored entry should be the When node, not the Do"
    );
}

#[test]
fn dispatch_mixed_do_and_when_stores_only_when() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
                EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                },
            ],
        }],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "only the When child should be stored; Do should be fired immediately"
    );
    assert!(matches!(
        &bound.0[0].1,
        EffectNode::When {
            trigger: Trigger::BoltLost,
            ..
        }
    ));
}

// ---- Behavior 8: Multiple Breaker-targeted effects ----

#[test]
fn dispatch_pushes_multiple_breaker_targeted_effects() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::EarlyBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::LateBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                }],
            },
        ],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 4, "expected 4 effects in BoundEffects");
}
