//! System to propagate `ArchetypeDefinition` asset changes to live game state.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    behaviors::{
        active::ActiveBehaviors,
        consequences::{bolt_speed_boost::apply_bolt_speed_boosts, life_lost::LivesCount},
        definition::ArchetypeDefinition,
        init::apply_stat_overrides,
        registry::ArchetypeRegistry,
    },
    breaker::{
        components::Breaker,
        resources::{BreakerConfig, BreakerDefaults},
    },
    screen::loading::resources::DefaultsCollection,
    shared::SelectedArchetype,
};

/// Bundled system parameters for the archetype change propagation system.
#[derive(SystemParam)]
pub(crate) struct ArchetypeChangeContext<'w, 's> {
    /// Asset collection handles.
    collection: Res<'w, DefaultsCollection>,
    /// Loaded archetype definition assets.
    assets: Res<'w, Assets<ArchetypeDefinition>>,
    /// Loaded breaker defaults assets.
    defaults_assets: Res<'w, Assets<BreakerDefaults>>,
    /// Currently selected archetype name.
    selected: Res<'w, SelectedArchetype>,
    /// Mutable archetype registry.
    registry: ResMut<'w, ArchetypeRegistry>,
    /// Mutable breaker configuration.
    config: ResMut<'w, BreakerConfig>,
    /// Mutable active behaviors set.
    active: ResMut<'w, ActiveBehaviors>,
    /// Breaker entities for re-stamping components.
    breaker_query: Query<'w, 's, Entity, With<Breaker>>,
    /// Command buffer for entity modifications.
    commands: Commands<'w, 's>,
}

/// Detects `AssetEvent::Modified` on any `ArchetypeDefinition`, rebuilds
/// `ArchetypeRegistry`, and if the selected archetype was modified:
/// 1. Resets `BreakerConfig` from defaults + re-applies stat overrides
/// 2. Re-stamps consequence components (bolt speed multipliers)
/// 3. Resets `LivesCount` if archetype has `life_pool`
/// 4. Rebuilds `ActiveBehaviors`
pub(crate) fn propagate_archetype_changes(
    mut events: MessageReader<AssetEvent<ArchetypeDefinition>>,
    mut ctx: ArchetypeChangeContext,
) {
    let any_modified = events.read().any(|event| {
        ctx.collection
            .archetypes
            .iter()
            .any(|h| event.is_modified(h.id()))
    });

    if !any_modified {
        return;
    }

    // Rebuild registry
    ctx.registry.archetypes.clear();
    for handle in &ctx.collection.archetypes {
        if let Some(def) = ctx.assets.get(handle.id()) {
            ctx.registry
                .archetypes
                .insert(def.name.clone(), def.clone());
        }
    }

    // Check if the selected archetype was modified
    let Some(def) = ctx.registry.archetypes.get(&ctx.selected.0) else {
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
        apply_bolt_speed_boosts(&mut ctx.commands, entity, &def.behaviors);

        if let Some(life_pool) = def.life_pool {
            ctx.commands.entity(entity).insert(LivesCount(life_pool));
        }
    }

    *ctx.active = ActiveBehaviors::from_bindings(&def.behaviors);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        behaviors::definition::{BehaviorBinding, BreakerStatOverrides, Consequence, Trigger},
        breaker::components::BumpPerfectMultiplier,
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<ArchetypeDefinition>()
            .init_asset::<BreakerDefaults>()
            .init_resource::<BreakerConfig>()
            .init_resource::<ArchetypeRegistry>()
            .init_resource::<SelectedArchetype>()
            .init_resource::<ActiveBehaviors>()
            .add_systems(Update, propagate_archetype_changes);
        app
    }

    fn make_collection(archetypes: Vec<Handle<ArchetypeDefinition>>) -> DefaultsCollection {
        DefaultsCollection {
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            playfield: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            chipselect: Handle::default(),
            cell_types: vec![],
            layouts: vec![],
            archetypes,
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
            difficulty: Handle::default(),
        }
    }

    #[test]
    fn registry_rebuilt_on_modified() {
        let mut app = test_app();

        let def = ArchetypeDefinition {
            name: "Test".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: Some(3),
            behaviors: vec![],
        };
        let handle = {
            let mut assets = app
                .world_mut()
                .resource_mut::<Assets<ArchetypeDefinition>>();
            assets.add(def)
        };

        app.world_mut()
            .insert_resource(SelectedArchetype("Test".to_owned()));
        app.world_mut()
            .insert_resource(make_collection(vec![handle.clone()]));

        app.update();
        app.update();

        // Modify the archetype
        {
            let mut assets = app
                .world_mut()
                .resource_mut::<Assets<ArchetypeDefinition>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.life_pool = Some(5);
        }

        app.update();
        app.update();

        let registry = app.world().resource::<ArchetypeRegistry>();
        let rebuilt = registry.archetypes.get("Test").unwrap();
        assert_eq!(rebuilt.life_pool, Some(5));
    }

    #[test]
    fn config_reset_with_overrides_on_archetype_change() {
        let mut app = test_app();

        let def = ArchetypeDefinition {
            name: "Wide".to_owned(),
            stat_overrides: BreakerStatOverrides {
                width: Some(200.0),
                ..default()
            },
            life_pool: None,
            behaviors: vec![],
        };
        let handle = {
            let mut assets = app
                .world_mut()
                .resource_mut::<Assets<ArchetypeDefinition>>();
            assets.add(def)
        };

        app.world_mut()
            .insert_resource(SelectedArchetype("Wide".to_owned()));
        app.world_mut()
            .insert_resource(make_collection(vec![handle.clone()]));

        // Manually set config to something different to detect change
        app.world_mut().resource_mut::<BreakerConfig>().width = 999.0;

        app.update();
        app.update();

        // Modify archetype override to 250
        {
            let mut assets = app
                .world_mut()
                .resource_mut::<Assets<ArchetypeDefinition>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.stat_overrides.width = Some(250.0);
        }

        app.update();
        app.update();

        let config = app.world().resource::<BreakerConfig>();
        assert!(
            (config.width - 250.0).abs() < f32::EPSILON,
            "BreakerConfig.width should be 250.0 after archetype override change, got {}",
            config.width
        );
    }

    #[test]
    fn active_behaviors_rebuilt_on_archetype_change() {
        let mut app = test_app();

        let def = ArchetypeDefinition {
            name: "Test".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            behaviors: vec![BehaviorBinding {
                triggers: vec![Trigger::PerfectBump],
                consequence: Consequence::BoltSpeedBoost(1.5),
            }],
        };
        let handle = {
            let mut assets = app
                .world_mut()
                .resource_mut::<Assets<ArchetypeDefinition>>();
            assets.add(def)
        };

        app.world_mut()
            .insert_resource(SelectedArchetype("Test".to_owned()));
        app.world_mut()
            .insert_resource(make_collection(vec![handle.clone()]));

        app.update();
        app.update();

        // Modify behaviors
        {
            let mut assets = app
                .world_mut()
                .resource_mut::<Assets<ArchetypeDefinition>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.behaviors = vec![
                BehaviorBinding {
                    triggers: vec![Trigger::BoltLost],
                    consequence: Consequence::LoseLife,
                },
                BehaviorBinding {
                    triggers: vec![Trigger::EarlyBump, Trigger::LateBump],
                    consequence: Consequence::BoltSpeedBoost(1.1),
                },
            ];
        }

        app.update();
        app.update();

        let active = app.world().resource::<ActiveBehaviors>();
        assert_eq!(
            active.0.len(),
            3,
            "should have 3 flattened bindings (1 + 2 multi-trigger)"
        );
        assert!(active.has_trigger(Trigger::BoltLost));
        assert!(active.has_trigger(Trigger::EarlyBump));
        assert!(active.has_trigger(Trigger::LateBump));
    }

    #[test]
    fn lives_count_reset_on_archetype_change() {
        let mut app = test_app();

        let def = ArchetypeDefinition {
            name: "Test".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: Some(3),
            behaviors: vec![],
        };
        let handle = {
            let mut assets = app
                .world_mut()
                .resource_mut::<Assets<ArchetypeDefinition>>();
            assets.add(def)
        };

        app.world_mut()
            .insert_resource(SelectedArchetype("Test".to_owned()));
        app.world_mut()
            .insert_resource(make_collection(vec![handle.clone()]));

        // Spawn breaker with 1 life remaining (took damage)
        let entity = app.world_mut().spawn((Breaker, LivesCount(1))).id();

        app.update();
        app.update();

        // Modify archetype to 5 lives
        {
            let mut assets = app
                .world_mut()
                .resource_mut::<Assets<ArchetypeDefinition>>();
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
    fn multipliers_re_stamped_on_archetype_change() {
        let mut app = test_app();

        let def = ArchetypeDefinition {
            name: "Test".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            behaviors: vec![BehaviorBinding {
                triggers: vec![Trigger::PerfectBump],
                consequence: Consequence::BoltSpeedBoost(1.5),
            }],
        };
        let handle = {
            let mut assets = app
                .world_mut()
                .resource_mut::<Assets<ArchetypeDefinition>>();
            assets.add(def)
        };

        app.world_mut()
            .insert_resource(SelectedArchetype("Test".to_owned()));
        app.world_mut()
            .insert_resource(make_collection(vec![handle.clone()]));

        let entity = app
            .world_mut()
            .spawn((Breaker, BumpPerfectMultiplier(1.0)))
            .id();

        app.update();
        app.update();

        // Modify multiplier
        {
            let mut assets = app
                .world_mut()
                .resource_mut::<Assets<ArchetypeDefinition>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.behaviors = vec![BehaviorBinding {
                triggers: vec![Trigger::PerfectBump],
                consequence: Consequence::BoltSpeedBoost(2.0),
            }];
        }

        app.update();
        app.update();

        let mult = app.world().get::<BumpPerfectMultiplier>(entity).unwrap();
        assert!(
            (mult.0 - 2.0).abs() < f32::EPSILON,
            "BumpPerfectMultiplier should be re-stamped to 2.0, got {}",
            mult.0
        );
    }
}
