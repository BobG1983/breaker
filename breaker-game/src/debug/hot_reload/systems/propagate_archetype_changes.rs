//! System to propagate `ArchetypeDefinition` asset changes to live game state.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    effect::{
        active::ActiveEffects, definition::ArchetypeDefinition, effects::life_lost::LivesCount,
        init::apply_stat_overrides, registry::ArchetypeRegistry,
    },
    breaker::{
        components::Breaker,
        resources::{BreakerConfig, BreakerDefaults},
    },
    chips::definition::TriggerChain,
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
    /// Mutable active chains.
    active: ResMut<'w, ActiveEffects>,
    /// Breaker entities for re-stamping components.
    breaker_query: Query<'w, 's, Entity, With<Breaker>>,
    /// Command buffer for entity modifications.
    commands: Commands<'w, 's>,
}

/// Detects `AssetEvent::Modified` on any `ArchetypeDefinition`, rebuilds
/// `ArchetypeRegistry`, and if the selected archetype was modified:
/// 1. Resets `BreakerConfig` from defaults + re-applies stat overrides
/// 2. Resets `LivesCount` if archetype has `life_pool`
/// 3. Rebuilds `ActiveEffects`
pub(crate) fn propagate_archetype_changes(
    mut events: MessageReader<AssetEvent<ArchetypeDefinition>>,
    mut ctx: ArchetypeChangeContext,
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

    // Check if the selected archetype was modified
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

    // Build ActiveEffects from root fields + chains
    let mut chains = Vec::new();
    if let Some(chain) = &def.on_bolt_lost {
        chains.push((None, TriggerChain::OnBoltLost(vec![chain.clone()])));
    }
    if let Some(chain) = &def.on_perfect_bump {
        chains.push((None, TriggerChain::OnPerfectBump(vec![chain.clone()])));
    }
    if let Some(chain) = &def.on_early_bump {
        chains.push((None, TriggerChain::OnEarlyBump(vec![chain.clone()])));
    }
    if let Some(chain) = &def.on_late_bump {
        chains.push((None, TriggerChain::OnLateBump(vec![chain.clone()])));
    }
    chains.extend(def.chains.iter().cloned().map(|c| (None, c)));
    *ctx.active = ActiveEffects(chains);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{effect::definition::BreakerStatOverrides, chips::definition::Target};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<ArchetypeDefinition>()
            .init_asset::<BreakerDefaults>()
            .init_resource::<BreakerConfig>()
            .init_resource::<ArchetypeRegistry>()
            .init_resource::<SelectedArchetype>()
            .init_resource::<ActiveEffects>()
            .add_systems(Update, propagate_archetype_changes);
        app
    }

    fn make_collection(breakers: Vec<Handle<ArchetypeDefinition>>) -> DefaultsCollection {
        DefaultsCollection {
            bolt: Handle::default(),
            breaker: Handle::default(),
            cell_defaults: Handle::default(),
            playfield: Handle::default(),
            input: Handle::default(),
            main_menu: Handle::default(),
            timer_ui: Handle::default(),
            chip_select: Handle::default(),
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

        let def = ArchetypeDefinition {
            name: "Test".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: Some(3),
            on_bolt_lost: None,
            on_perfect_bump: None,
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![],
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
        let rebuilt = registry.get("Test").unwrap();
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
            on_bolt_lost: None,
            on_perfect_bump: None,
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![],
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
    fn active_chains_rebuilt_on_archetype_change() {
        let mut app = test_app();

        let def = ArchetypeDefinition {
            name: "Test".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            on_bolt_lost: None,
            on_perfect_bump: Some(TriggerChain::SpeedBoost {
                target: Target::Bolt,
                multiplier: 1.5,
            }),
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![],
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

        // Modify: add bolt_lost and early/late bump
        {
            let mut assets = app
                .world_mut()
                .resource_mut::<Assets<ArchetypeDefinition>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.on_bolt_lost = Some(TriggerChain::LoseLife);
            asset.on_early_bump = Some(TriggerChain::SpeedBoost {
                target: Target::Bolt,
                multiplier: 1.1,
            });
            asset.on_late_bump = Some(TriggerChain::SpeedBoost {
                target: Target::Bolt,
                multiplier: 1.1,
            });
        }

        app.update();
        app.update();

        let active = app.world().resource::<ActiveEffects>();
        // on_bolt_lost=LoseLife → OnBoltLost(LoseLife)
        // on_perfect_bump=SpeedBoost → OnPerfectBump(SpeedBoost{...})
        // on_early_bump=SpeedBoost → OnEarlyBump(SpeedBoost{...})
        // on_late_bump=SpeedBoost → OnLateBump(SpeedBoost{...})
        assert_eq!(
            active.0.len(),
            4,
            "should have 4 active chains (all included), got {}",
            active.0.len()
        );
    }

    #[test]
    fn lives_count_reset_on_archetype_change() {
        let mut app = test_app();

        let def = ArchetypeDefinition {
            name: "Test".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: Some(3),
            on_bolt_lost: None,
            on_perfect_bump: None,
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![],
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
    fn speed_boost_chains_appear_in_active_chains_on_archetype_change() {
        let mut app = test_app();

        let def = ArchetypeDefinition {
            name: "Test".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            on_bolt_lost: None,
            on_perfect_bump: Some(TriggerChain::SpeedBoost {
                target: Target::Bolt,
                multiplier: 1.5,
            }),
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![],
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

        app.world_mut().spawn(Breaker);

        app.update();
        app.update();

        // Modify multiplier
        {
            let mut assets = app
                .world_mut()
                .resource_mut::<Assets<ArchetypeDefinition>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.on_perfect_bump = Some(TriggerChain::SpeedBoost {
                target: Target::Bolt,
                multiplier: 2.0,
            });
        }

        app.update();
        app.update();

        let active = app.world().resource::<ActiveEffects>();
        assert_eq!(
            active.0.len(),
            1,
            "should have 1 active chain for SpeedBoost, got {}",
            active.0.len()
        );
        assert!(matches!(
            &active.0[0],
            (None, TriggerChain::OnPerfectBump(effects)) if effects.len() == 1 && matches!(
                effects[0],
                TriggerChain::SpeedBoost { multiplier, .. } if (multiplier - 2.0).abs() < f32::EPSILON
            )
        ));
    }
}
