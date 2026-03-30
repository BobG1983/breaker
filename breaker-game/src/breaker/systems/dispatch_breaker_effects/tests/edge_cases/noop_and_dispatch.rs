//! Behaviors 18-21: no-op conditions and multi-child dispatch.

use bevy::prelude::*;

use super::super::{
    super::system::dispatch_breaker_effects,
    helpers::{TEST_BREAKER_NAME, test_app_with_dispatch},
};
use crate::{
    bolt::components::Bolt,
    breaker::{
        SelectedBreaker,
        components::Breaker,
        definition::{BreakerDefinition, BreakerStatOverrides},
        registry::BreakerRegistry,
    },
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, Target, Trigger},
};

// ---- Behavior 18: Missing breaker in registry is a no-op ----

#[test]
fn dispatch_with_missing_breaker_in_registry_is_noop() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(BreakerRegistry::default())
        .insert_resource(SelectedBreaker("NonExistent".to_owned()))
        .add_systems(Update, dispatch_breaker_effects);

    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    let bolt = app.world_mut().spawn((Bolt, BoundEffects::default())).id();
    app.update();

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        0,
        "no effects should be dispatched when breaker not in registry"
    );
    let bolt_bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bolt_bound.0.len(),
        0,
        "no effects should be dispatched when breaker not in registry"
    );
}

// ---- Behavior 19: No breaker entity is a no-op ----

#[test]
fn dispatch_with_no_breaker_entity_is_noop() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    };
    let mut app = test_app_with_dispatch(def);
    // No breaker entity spawned
    let bolt = app.world_mut().spawn((Bolt, BoundEffects::default())).id();
    app.update();

    let bolt_bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bolt_bound.0.len(),
        0,
        "no effects should be dispatched when no breaker entity exists"
    );
}

// ---- Behavior 20: All children of an On node pushed, not just the first ----

#[test]
fn dispatch_pushes_all_children_of_on_node() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![
                EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                },
                EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
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
        2,
        "both children of the On node should be pushed"
    );
    assert!(matches!(
        &bound.0[0].1,
        EffectNode::When {
            trigger: Trigger::BoltLost,
            ..
        }
    ));
    assert!(matches!(
        &bound.0[1].1,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            ..
        }
    ));
}

// ---- Behavior 21: Empty string used as chip name for all pushed effects ----

#[test]
fn dispatch_uses_empty_string_as_chip_name_for_all_targets() {
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
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
        ],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    let bolt = app.world_mut().spawn(Bolt).id();
    app.update();

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        &breaker_bound.0[0].0, "",
        "breaker chip name should be empty string"
    );

    let bolt_bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects");
    assert_eq!(
        &bolt_bound.0[0].0, "",
        "bolt chip name should be empty string"
    );
}
