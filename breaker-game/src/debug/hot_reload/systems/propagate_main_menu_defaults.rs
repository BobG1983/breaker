//! System to propagate `MainMenuDefaults` asset changes to `MainMenuConfig`.

use bevy::prelude::*;

use crate::screen::{
    loading::resources::DefaultsCollection,
    main_menu::{MainMenuConfig, MainMenuDefaults},
};

/// Watches for `AssetEvent::Modified` on the main menu defaults asset and
/// re-seeds `MainMenuConfig` from the updated asset data.
pub(crate) fn propagate_main_menu_defaults(
    mut events: MessageReader<AssetEvent<MainMenuDefaults>>,
    collection: Res<DefaultsCollection>,
    assets: Res<Assets<MainMenuDefaults>>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.is_modified(collection.mainmenu.id())
            && let Some(defaults) = assets.get(collection.mainmenu.id())
        {
            commands.insert_resource::<MainMenuConfig>(defaults.clone().into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<MainMenuDefaults>()
            .init_resource::<MainMenuConfig>()
            .add_systems(Update, propagate_main_menu_defaults);
        app
    }

    fn make_collection(mainmenu: Handle<MainMenuDefaults>) -> DefaultsCollection {
        DefaultsCollection {
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            playfield: Handle::default(),
            input: Handle::default(),
            mainmenu,
            timerui: Handle::default(),
            chipselect: Handle::default(),
            cell_types: vec![],
            layouts: vec![],
            archetypes: vec![],
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
            difficulty: Handle::default(),
            evolutions: vec![],
        }
    }

    /// After an Added event only (no Modified), `MainMenuConfig` should not change.
    #[test]
    fn config_unchanged_when_no_modified_event() {
        let mut app = test_app();

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<MainMenuDefaults>>();
            assets.add(MainMenuDefaults::default())
        };
        app.world_mut().insert_resource(make_collection(handle));

        app.update();
        app.update();

        let config = app.world().resource::<MainMenuConfig>();
        let default_size = MainMenuConfig::default().title_font_size;
        assert!(
            (config.title_font_size - default_size).abs() < f32::EPSILON,
            "config should not change when only an Added event is received"
        );
    }

    /// When the main menu defaults asset is mutated, `MainMenuConfig` must be
    /// re-seeded with the new values.
    #[test]
    fn config_updated_when_modified_event_fires() {
        let mut app = test_app();

        let new_title_size = 128.0_f32;
        let defaults = MainMenuDefaults {
            title_font_size: new_title_size,
            ..Default::default()
        };

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<MainMenuDefaults>>();
            assets.add(defaults)
        };
        app.world_mut()
            .insert_resource(make_collection(handle.clone()));

        app.update();
        app.update();

        {
            let mut assets = app.world_mut().resource_mut::<Assets<MainMenuDefaults>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.title_font_size = new_title_size;
        }

        app.update();
        app.update();

        let config = app.world().resource::<MainMenuConfig>();
        assert!(
            (config.title_font_size - new_title_size).abs() < f32::EPSILON,
            "MainMenuConfig.title_font_size should be {new_title_size} after Modified event, got {}",
            config.title_font_size
        );
    }
}
