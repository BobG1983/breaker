use bevy::prelude::*;

use super::system::*;
use crate::{
    breaker::{SelectedBreaker, components::Breaker, resources::BreakerConfig},
    effect::{
        definition::{
            BreakerDefinition, BreakerStatOverrides, Effect, EffectChains, EffectNode, RootEffect,
            Target, Trigger,
        },
        effects::life_lost::LivesCount,
        registry::BreakerRegistry,
    },
    screen::loading::resources::DefaultsCollection,
};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<BreakerDefinition>()
        .init_asset::<crate::breaker::resources::BreakerDefaults>()
        .init_resource::<BreakerConfig>()
        .init_resource::<BreakerRegistry>()
        .init_resource::<SelectedBreaker>()
        .add_systems(Update, propagate_breaker_changes);
    app
}

fn make_collection(breakers: Vec<Handle<BreakerDefinition>>) -> DefaultsCollection {
    DefaultsCollection {
        cells: vec![],
        nodes: vec![],
        breakers,
        chips: vec![],
        chip_templates: vec![],
        difficulty: Handle::default(),
    }
}

#[test]
fn registry_rebuilt_on_modified() {
    let mut app = test_app();

    let def = BreakerDefinition {
        name: "Test".to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: Some(3),
        effects: vec![],
    };
    let handle = {
        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
        assets.add(def)
    };

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));
    app.world_mut()
        .insert_resource(make_collection(vec![handle.clone()]));

    app.update();
    app.update();

    // Modify the breaker definition
    {
        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
        let asset = assets.get_mut(handle.id()).expect("asset should exist");
        asset.life_pool = Some(5);
    }

    app.update();
    app.update();

    let registry = app.world().resource::<BreakerRegistry>();
    let rebuilt = registry.get("Test").unwrap();
    assert_eq!(rebuilt.life_pool, Some(5));
}

#[test]
fn config_reset_with_overrides_on_breaker_change() {
    let mut app = test_app();

    let def = BreakerDefinition {
        name: "Wide".to_owned(),
        stat_overrides: BreakerStatOverrides {
            width: Some(200.0),
            ..default()
        },
        life_pool: None,
        effects: vec![],
    };
    let handle = {
        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
        assets.add(def)
    };

    app.world_mut()
        .insert_resource(SelectedBreaker("Wide".to_owned()));
    app.world_mut()
        .insert_resource(make_collection(vec![handle.clone()]));

    // Manually set config to something different to detect change
    app.world_mut().resource_mut::<BreakerConfig>().width = 999.0;

    app.update();
    app.update();

    // Modify breaker override to 250
    {
        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
        let asset = assets.get_mut(handle.id()).expect("asset should exist");
        asset.stat_overrides.width = Some(250.0);
    }

    app.update();
    app.update();

    let config = app.world().resource::<BreakerConfig>();
    assert!(
        (config.width - 250.0).abs() < f32::EPSILON,
        "BreakerConfig.width should be 250.0 after breaker override change, got {}",
        config.width
    );
}

#[test]
fn active_chains_rebuilt_on_breaker_change() {
    let mut app = test_app();

    let def = BreakerDefinition {
        name: "Test".to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    };
    let handle = {
        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
        assets.add(def)
    };

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));
    app.world_mut()
        .insert_resource(make_collection(vec![handle.clone()]));

    let breaker_entity = app
        .world_mut()
        .spawn((Breaker, EffectChains::default()))
        .id();

    app.update();
    app.update();

    // Modify: add 3 more effects
    {
        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
        let asset = assets.get_mut(handle.id()).expect("asset should exist");
        asset.effects = vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(Effect::LoseLife)],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::EarlyBump,
                    then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.1 })],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::LateBump,
                    then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.1 })],
                }],
            },
        ];
    }

    app.update();
    app.update();

    let chains = app.world().get::<EffectChains>(breaker_entity).unwrap();
    assert_eq!(
        chains.0.len(),
        4,
        "should have 4 chains on breaker entity (all included), got {}",
        chains.0.len()
    );
}

#[test]
fn lives_count_reset_on_breaker_change() {
    let mut app = test_app();

    let def = BreakerDefinition {
        name: "Test".to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: Some(3),
        effects: vec![],
    };
    let handle = {
        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
        assets.add(def)
    };

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));
    app.world_mut()
        .insert_resource(make_collection(vec![handle.clone()]));

    // Spawn breaker with 1 life remaining (took damage)
    let entity = app
        .world_mut()
        .spawn((Breaker, LivesCount(1), EffectChains::default()))
        .id();

    app.update();
    app.update();

    // Modify breaker to 5 lives
    {
        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
        let asset = assets.get_mut(handle.id()).expect("asset should exist");
        asset.life_pool = Some(5);
    }

    app.update();
    app.update();

    let lives = app.world().get::<LivesCount>(entity).unwrap();
    assert_eq!(
        lives.0, 5,
        "LivesCount should be reset to new life_pool value"
    );
}

#[test]
fn speed_boost_chains_appear_in_effect_chains_on_breaker_change() {
    let mut app = test_app();

    let def = BreakerDefinition {
        name: "Test".to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    };
    let handle = {
        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
        assets.add(def)
    };

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));
    app.world_mut()
        .insert_resource(make_collection(vec![handle.clone()]));

    let breaker_entity = app
        .world_mut()
        .spawn((Breaker, EffectChains::default()))
        .id();

    app.update();
    app.update();

    // Modify multiplier
    {
        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
        let asset = assets.get_mut(handle.id()).expect("asset should exist");
        asset.effects = vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 2.0 })],
            }],
        }];
    }

    app.update();
    app.update();

    let chains = app.world().get::<EffectChains>(breaker_entity).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "should have 1 chain for SpeedBoost on breaker entity, got {}",
        chains.0.len()
    );
    assert!(matches!(
        &chains.0[0],
        (None, EffectNode::When { trigger: Trigger::PerfectBump, then }) if then.len() == 1 && matches!(
            &then[0],
            EffectNode::Do(Effect::SpeedBoost { multiplier, .. }) if (*multiplier - 2.0).abs() < f32::EPSILON
        )
    ));
}
