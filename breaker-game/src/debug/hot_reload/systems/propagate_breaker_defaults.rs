//! System to propagate `BreakerDefaults` asset changes to `BreakerConfig`,
//! then re-apply the currently selected archetype's stat overrides.

use bevy::prelude::*;

use crate::{
    breaker::{BreakerConfig, BreakerDefaults},
    effect::{init::apply_stat_overrides, registry::ArchetypeRegistry},
    screen::loading::resources::DefaultsCollection,
    shared::SelectedArchetype,
};

/// Watches for `AssetEvent::Modified` on the breaker defaults asset,
/// re-seeds `BreakerConfig` from the updated asset data, then re-applies
/// the selected archetype's stat overrides.
pub(crate) fn propagate_breaker_defaults(
    mut events: MessageReader<AssetEvent<BreakerDefaults>>,
    collection: Res<DefaultsCollection>,
    assets: Res<Assets<BreakerDefaults>>,
    selected: Res<SelectedArchetype>,
    registry: Res<ArchetypeRegistry>,
    mut config: ResMut<BreakerConfig>,
) {
    for event in events.read() {
        if event.is_modified(collection.breaker.id())
            && let Some(defaults) = assets.get(collection.breaker.id())
        {
            *config = BreakerConfig::from(defaults.clone());
            if let Some(def) = registry.get(&selected.0) {
                apply_stat_overrides(&mut config, &def.stat_overrides);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::definition::{ArchetypeDefinition, BreakerStatOverrides};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<BreakerDefaults>()
            .init_resource::<BreakerConfig>()
            .init_resource::<ArchetypeRegistry>()
            .init_resource::<SelectedArchetype>()
            .add_systems(Update, propagate_breaker_defaults);
        app
    }

    fn make_collection(breaker: Handle<BreakerDefaults>) -> DefaultsCollection {
        DefaultsCollection {
            bolt: Handle::default(),
            breaker,
            cell_defaults: Handle::default(),
            playfield: Handle::default(),
            input: Handle::default(),
            main_menu: Handle::default(),
            timer_ui: Handle::default(),
            chip_select: Handle::default(),
            cells: vec![],
            nodes: vec![],
            breakers: vec![],
            chips: vec![],
            chip_templates: vec![],
            difficulty: Handle::default(),
        }
    }

    /// After an Added event only (no Modified), `BreakerConfig` should not change.
    #[test]
    fn config_unchanged_when_no_modified_event() {
        let mut app = test_app();

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefaults>>();
            assets.add(BreakerDefaults::default())
        };
        app.world_mut().insert_resource(make_collection(handle));

        app.update();
        app.update();

        let config = app.world().resource::<BreakerConfig>();
        let default_width = BreakerConfig::default().width;
        assert!(
            (config.width - default_width).abs() < f32::EPSILON,
            "config should not change when only an Added event is received"
        );
    }

    /// When breaker defaults are modified, `BreakerConfig` is re-seeded from the
    /// new asset values.
    #[test]
    fn config_updated_when_modified_event_fires() {
        let mut app = test_app();

        let new_width = 200.0_f32;
        let defaults = BreakerDefaults {
            width: new_width,
            ..Default::default()
        };

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefaults>>();
            assets.add(defaults)
        };

        app.world_mut()
            .insert_resource(make_collection(handle.clone()));
        // No archetype selected with overrides — default registry is empty.
        app.world_mut()
            .insert_resource(SelectedArchetype("None".to_owned()));

        app.update();
        app.update();

        {
            let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefaults>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.width = new_width;
        }

        app.update();
        app.update();

        let config = app.world().resource::<BreakerConfig>();
        assert!(
            (config.width - new_width).abs() < f32::EPSILON,
            "BreakerConfig.width should be {new_width} after Modified, got {}",
            config.width
        );
    }

    /// After re-seeding from defaults, the selected archetype's stat overrides
    /// must be re-applied on top of the base config values.
    #[test]
    fn archetype_overrides_re_applied_after_defaults_modified() {
        const ARCHETYPE_NAME: &str = "TestArch";
        const OVERRIDE_WIDTH: f32 = 250.0;

        let mut app = test_app();

        let def = ArchetypeDefinition {
            name: ARCHETYPE_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides {
                width: Some(OVERRIDE_WIDTH),
                ..default()
            },
            life_pool: None,
            on_bolt_lost: None,
            on_perfect_bump: None,
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![],
        };

        let mut registry = ArchetypeRegistry::default();
        registry.insert(ARCHETYPE_NAME.to_owned(), def);
        app.world_mut().insert_resource(registry);
        app.world_mut()
            .insert_resource(SelectedArchetype(ARCHETYPE_NAME.to_owned()));

        // Defaults have base width of 120.0; override will make it 250.0.
        let defaults = BreakerDefaults {
            width: 120.0,
            ..Default::default()
        };

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefaults>>();
            assets.add(defaults)
        };
        app.world_mut()
            .insert_resource(make_collection(handle.clone()));

        app.update();
        app.update();

        // Mutate defaults to trigger Modified.
        {
            let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefaults>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.width = 120.0; // same base value, override should still apply
        }

        app.update();
        app.update();

        let config = app.world().resource::<BreakerConfig>();
        assert!(
            (config.width - OVERRIDE_WIDTH).abs() < f32::EPSILON,
            "archetype override ({OVERRIDE_WIDTH}) must be applied after re-seeding; got {}",
            config.width
        );
    }

    /// Without an archetype override, the width from the new defaults asset
    /// value is used directly.
    #[test]
    fn without_archetype_override_base_defaults_width_is_used() {
        let mut app = test_app();

        let new_base_width = 180.0_f32;

        // Archetype with no width override.
        let def = ArchetypeDefinition {
            name: "Plain".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            on_bolt_lost: None,
            on_perfect_bump: None,
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![],
        };
        let mut registry = ArchetypeRegistry::default();
        registry.insert("Plain".to_owned(), def);
        app.world_mut().insert_resource(registry);
        app.world_mut()
            .insert_resource(SelectedArchetype("Plain".to_owned()));

        let defaults = BreakerDefaults {
            width: new_base_width,
            ..Default::default()
        };
        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefaults>>();
            assets.add(defaults)
        };
        app.world_mut()
            .insert_resource(make_collection(handle.clone()));

        app.update();
        app.update();

        {
            let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefaults>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.width = new_base_width;
        }

        app.update();
        app.update();

        let config = app.world().resource::<BreakerConfig>();
        assert!(
            (config.width - new_base_width).abs() < f32::EPSILON,
            "without override, base defaults width {new_base_width} should be used; got {}",
            config.width
        );
    }
}
