use bevy::prelude::*;

use super::*;
use crate::{
    breaker::{
        SelectedBreaker,
        components::{Breaker, BreakerInitialized},
        definition::{BreakerDefinition, BreakerStatOverrides},
        registry::BreakerRegistry,
        resources::{BreakerConfig, BreakerDefaults},
    },
    effect::{effects::life_lost::LivesCount, *},
};

const TEST_BREAKER_NAME: &str = "TestBreaker";

fn make_test_breaker() -> BreakerDefinition {
    BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        bolt: "Bolt".to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: Some(3),
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
    }
}

fn test_app_with_breaker(def: BreakerDefinition) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let mut registry = BreakerRegistry::default();
    registry.insert(def.name.clone(), def);
    app.insert_resource(registry)
        .insert_resource(SelectedBreaker(TEST_BREAKER_NAME.to_owned()))
        .add_systems(Update, init_breaker);
    app
}

#[test]
fn init_breaker_stamps_lives_count() {
    let mut app = test_app_with_breaker(make_test_breaker());
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let lives = app.world().get::<LivesCount>(entity).unwrap();
    assert_eq!(lives.0, Some(3));
}

#[test]
fn init_breaker_stamps_breaker_initialized_marker() {
    let mut app = test_app_with_breaker(make_test_breaker());
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    assert!(
        app.world().get::<BreakerInitialized>(entity).is_some(),
        "init_breaker should insert BreakerInitialized marker component"
    );
}

#[test]
fn init_breaker_does_not_push_effects() {
    let mut app = test_app_with_breaker(make_test_breaker());
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let chains = app.world().get::<BoundEffects>(entity).unwrap();
    // init_breaker should NOT push effects to BoundEffects -- that is
    // dispatch_breaker_effects' responsibility
    assert_eq!(
        chains.0.len(),
        0,
        "init_breaker should not push any effects to BoundEffects"
    );
}

#[test]
fn init_breaker_does_not_push_effects_mixed_targets() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        bolt: "Bolt".to_owned(),
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
            RootEffect::On {
                target: Target::AllCells,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::test_shockwave(32.0))],
                }],
            },
        ],
    };
    let mut app = test_app_with_breaker(def);
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let chains = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        chains.0.len(),
        0,
        "init_breaker should not push any effects (including Breaker-targeted) to BoundEffects"
    );
}

#[test]
fn init_breaker_does_not_push_effects_chrono_style() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        bolt: "Bolt".to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::TimePenalty { seconds: 5.0 })],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::SpawnBolts {
                        count: 1,
                        lifespan: None,
                        inherit: false,
                    })],
                }],
            },
        ],
    };
    let mut app = test_app_with_breaker(def);
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let chains = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        chains.0.len(),
        0,
        "init_breaker should not push any effects to BoundEffects"
    );
}

#[test]
fn init_breaker_does_not_push_nested_chains() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        bolt: "Bolt".to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(EffectKind::test_shockwave(64.0))],
                }],
            }],
        }],
    };
    let mut app = test_app_with_breaker(def);
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let chains = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        chains.0.len(),
        0,
        "init_breaker should not push any effects to BoundEffects"
    );
}

#[test]
fn init_breaker_skips_already_initialized() {
    let mut app = test_app_with_breaker(make_test_breaker());
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BreakerInitialized,
            LivesCount(Some(99)),
            BoundEffects::default(),
        ))
        .id();
    app.update();

    let lives = app.world().get::<LivesCount>(entity).unwrap();
    assert_eq!(
        lives.0,
        Some(99),
        "should not overwrite existing LivesCount"
    );
}

#[test]
fn apply_stat_overrides_partial() {
    let mut config = BreakerConfig::default();
    let original_max_speed = config.max_speed;
    let original_accel = config.acceleration;

    let overrides = BreakerStatOverrides {
        width: Some(200.0),
        height: Some(30.0),
        ..default()
    };

    apply_stat_overrides(&mut config, &overrides);

    assert!((config.width - 200.0).abs() < f32::EPSILON);
    assert!((config.height - 30.0).abs() < f32::EPSILON);
    assert!(
        (config.max_speed - original_max_speed).abs() < f32::EPSILON,
        "unset fields should remain unchanged"
    );
    assert!(
        (config.acceleration - original_accel).abs() < f32::EPSILON,
        "unset fields should remain unchanged"
    );
}

#[test]
fn apply_stat_overrides_all_fields() {
    let mut config = BreakerConfig::default();
    let overrides = BreakerStatOverrides {
        width: Some(100.0),
        height: Some(20.0),
        max_speed: Some(500.0),
        acceleration: Some(1000.0),
        deceleration: Some(2000.0),
    };

    apply_stat_overrides(&mut config, &overrides);

    assert!((config.width - 100.0).abs() < f32::EPSILON);
    assert!((config.height - 20.0).abs() < f32::EPSILON);
    assert!((config.max_speed - 500.0).abs() < f32::EPSILON);
    assert!((config.acceleration - 1000.0).abs() < f32::EPSILON);
    assert!((config.deceleration - 2000.0).abs() < f32::EPSILON);
}

#[test]
fn apply_stat_overrides_empty_is_noop() {
    let original = BreakerConfig::default();
    let mut config = BreakerConfig::default();
    let overrides = BreakerStatOverrides::default();

    apply_stat_overrides(&mut config, &overrides);

    assert!((config.width - original.width).abs() < f32::EPSILON);
    assert!((config.height - original.height).abs() < f32::EPSILON);
    assert!((config.max_speed - original.max_speed).abs() < f32::EPSILON);
}

#[test]
fn apply_overrides_modifies_config() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<BreakerDefaults>()
        .init_resource::<BreakerConfig>();

    let def = BreakerDefinition {
        name: "Wide".to_owned(),
        bolt: "Bolt".to_owned(),
        stat_overrides: BreakerStatOverrides {
            width: Some(200.0),
            ..default()
        },
        life_pool: None,
        effects: vec![],
    };

    let mut registry = BreakerRegistry::default();
    registry.insert("Wide".to_owned(), def);
    app.insert_resource(registry)
        .insert_resource(SelectedBreaker("Wide".to_owned()))
        .add_systems(Update, apply_breaker_config_overrides);
    app.update();

    let config = app.world().resource::<BreakerConfig>();
    assert!((config.width - 200.0).abs() < f32::EPSILON);
    let default_config = BreakerConfig::default();
    assert!((config.max_speed - default_config.max_speed).abs() < f32::EPSILON);
}

#[test]
fn life_pool_none_stamps_infinite_lives() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        bolt: "Bolt".to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![],
    };

    let mut app = test_app_with_breaker(def);
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let lives = app
        .world()
        .get::<LivesCount>(entity)
        .expect("breaker with life_pool: None should still have LivesCount component");
    assert_eq!(
        lives.0, None,
        "breaker with life_pool: None should have LivesCount(None) for infinite lives"
    );
}

#[test]
fn init_breaker_no_duplicate_init_on_reentry() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        bolt: "Bolt".to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    };
    let mut app = test_app_with_breaker(def);
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();

    // First invocation — init_breaker no longer pushes effects
    app.update();
    let chains = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        chains.0.len(),
        0,
        "init_breaker should not push any effects to BoundEffects"
    );

    // Second invocation — BreakerInitialized marker should prevent re-entry
    app.update();
    let chains = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        chains.0.len(),
        0,
        "second init should still leave BoundEffects empty"
    );
}
