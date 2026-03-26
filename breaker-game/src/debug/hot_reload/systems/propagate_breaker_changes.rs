//! System to propagate `BreakerDefinition` asset changes to live game state.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    breaker::{
        components::Breaker,
        resources::{BreakerConfig, BreakerDefaults},
    },
    effect::{
        definition::{BreakerDefinition, EffectChains, RootEffect, Target},
        effects::life_lost::LivesCount,
        init::apply_stat_overrides,
        registry::BreakerRegistry,
    },
    screen::loading::resources::DefaultsCollection,
    shared::SelectedBreaker,
};

/// Bundled system parameters for the breaker change propagation system.
#[derive(SystemParam)]
pub(crate) struct BreakerChangeContext<'w, 's> {
    /// Asset collection handles.
    collection: Res<'w, DefaultsCollection>,
    /// Loaded breaker definition assets.
    assets: Res<'w, Assets<BreakerDefinition>>,
    /// Loaded breaker defaults assets.
    defaults_assets: Res<'w, Assets<BreakerDefaults>>,
    /// Currently selected breaker name.
    selected: Res<'w, SelectedBreaker>,
    /// Mutable breaker registry.
    registry: ResMut<'w, BreakerRegistry>,
    /// Mutable breaker configuration.
    config: ResMut<'w, BreakerConfig>,
    /// Breaker entities for re-stamping components.
    breaker_query: Query<'w, 's, Entity, With<Breaker>>,
    /// Breaker `EffectChains` for populating from definition.
    breaker_chains_query: Query<'w, 's, &'static mut EffectChains, With<Breaker>>,
    /// Command buffer for entity modifications.
    commands: Commands<'w, 's>,
}

/// Detects `AssetEvent::Modified` on any `BreakerDefinition`, rebuilds
/// `BreakerRegistry`, and if the selected breaker was modified:
/// 1. Resets `BreakerConfig` from defaults + re-applies stat overrides
/// 2. Resets `LivesCount` if breaker has `life_pool`
/// 3. Rebuilds breaker entity `EffectChains`
pub(crate) fn propagate_breaker_changes(
    mut events: MessageReader<AssetEvent<BreakerDefinition>>,
    mut ctx: BreakerChangeContext,
) {
    let any_modified = events.read().any(|event| {
        ctx.collection
            .breakers
            .iter()
            .any(|h| event.is_modified(h.id()))
    });

    if !any_modified {
        return;
    }

    // Rebuild registry
    ctx.registry.clear();
    for handle in &ctx.collection.breakers {
        if let Some(def) = ctx.assets.get(handle.id()) {
            ctx.registry.insert(def.name.clone(), def.clone());
        }
    }

    // Check if the selected breaker was modified
    let Some(def) = ctx.registry.get(&ctx.selected.0) else {
        return;
    };
    let def = def.clone();

    // Reset BreakerConfig from defaults + re-apply stat overrides
    if let Some(loaded) = ctx.defaults_assets.iter().next().map(|(_, d)| d) {
        *ctx.config = BreakerConfig::from(loaded.clone());
    }
    apply_stat_overrides(&mut ctx.config, &def.stat_overrides);

    // Re-stamp consequence components and lives on breaker entities
    for entity in &ctx.breaker_query {
        if let Some(life_pool) = def.life_pool {
            ctx.commands.entity(entity).insert(LivesCount(life_pool));
        }
    }

    // Resolve On targets to entity EffectChains
    for mut chains in &mut ctx.breaker_chains_query {
        chains.0.clear();
    }
    for root in &def.effects {
        let RootEffect::On { target, then } = root;
        match target {
            Target::Breaker => {
                for mut chains in &mut ctx.breaker_chains_query {
                    for child in then {
                        chains.0.push((None, child.clone()));
                    }
                }
            }
            // At hot-reload time, bolt/cell/wall targets are not resolved here
            Target::Bolt | Target::AllBolts | Target::Cell | Target::Wall | Target::AllCells => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::definition::{
        BreakerStatOverrides, Effect, EffectNode, RootEffect, Target, Trigger,
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<BreakerDefinition>()
            .init_asset::<BreakerDefaults>()
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
}
