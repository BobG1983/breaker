//! System to propagate `BreakerDefaults` asset changes to `BreakerConfig`,
//! then re-apply the currently selected archetype's stat overrides.

use bevy::prelude::*;

use crate::{
    behaviors::{init::apply_stat_overrides, registry::ArchetypeRegistry},
    breaker::{BreakerConfig, BreakerDefaults},
    screen::loading::resources::DefaultsCollection,
    shared::SelectedArchetype,
};

/// Watches for `AssetEvent::Modified` on the breaker defaults asset,
/// re-seeds `BreakerConfig` from the updated asset data, then re-applies
/// the selected archetype's stat overrides.
pub fn propagate_breaker_defaults(
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
            if let Some(def) = registry.archetypes.get(&selected.0) {
                apply_stat_overrides(&mut config, &def.stat_overrides);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::behaviors::definition::{
        ArchetypeDefinition, BehaviorBinding, BreakerStatOverrides,
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<BreakerDefaults>();
        app.init_resource::<BreakerConfig>();
        app.init_resource::<ArchetypeRegistry>();
        app.init_resource::<SelectedArchetype>();
        app.add_systems(Update, propagate_breaker_defaults);
        app
    }

    fn make_collection(breaker: Handle<BreakerDefaults>) -> DefaultsCollection {
        use crate::{
            bolt::BoltDefaults,
            cells::CellDefaults,
            input::InputDefaults,
            screen::{chip_select::ChipSelectDefaults, main_menu::MainMenuDefaults},
            shared::PlayfieldDefaults,
            ui::TimerUiDefaults,
        };
        DefaultsCollection {
            bolt: Handle::default(),
            breaker,
            cells: Handle::default(),
            playfield: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            chipselect: Handle::default(),
            cell_types: vec![],
            layouts: vec![],
            archetypes: vec![],
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
        }
    }

    /// After an Added event only (no Modified), BreakerConfig should not change.
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

    /// When breaker defaults are modified, BreakerConfig is re-seeded from the
    /// new asset values.
    #[test]
    fn config_updated_when_modified_event_fires() {
        let mut app = test_app();

        let new_width = 200.0_f32;
        let mut defaults = BreakerDefaults::default();
        defaults.width = new_width;

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
        let mut app = test_app();

        const ARCHETYPE_NAME: &str = "TestArch";
        const OVERRIDE_WIDTH: f32 = 250.0;

        let def = ArchetypeDefinition {
            name: ARCHETYPE_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides {
                width: Some(OVERRIDE_WIDTH),
                ..default()
            },
            life_pool: None,
            behaviors: vec![],
        };

        let mut registry = ArchetypeRegistry::default();
        registry.archetypes.insert(ARCHETYPE_NAME.to_owned(), def);
        app.world_mut().insert_resource(registry);
        app.world_mut()
            .insert_resource(SelectedArchetype(ARCHETYPE_NAME.to_owned()));

        // Defaults have base width of 120.0; override will make it 250.0.
        let mut defaults = BreakerDefaults::default();
        defaults.width = 120.0;

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
            behaviors: vec![],
        };
        let mut registry = ArchetypeRegistry::default();
        registry.archetypes.insert("Plain".to_owned(), def);
        app.world_mut().insert_resource(registry);
        app.world_mut()
            .insert_resource(SelectedArchetype("Plain".to_owned()));

        let mut defaults = BreakerDefaults::default();
        defaults.width = new_base_width;
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
