//! Seeds `MainMenuConfig` from loaded `MainMenuDefaults`.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::screen::{
    loading::resources::DefaultsCollection,
    main_menu::{MainMenuConfig, MainMenuDefaults},
};

/// Reads the loaded `MainMenuDefaults` asset and inserts `MainMenuConfig`.
pub(crate) fn seed_main_menu_config(
    collection: Option<Res<DefaultsCollection>>,
    assets: Res<Assets<MainMenuDefaults>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let Some(defaults) = assets.get(&collection.mainmenu) else {
        return Progress { done: 0, total: 1 };
    };

    commands.insert_resource::<MainMenuConfig>(defaults.clone().into());
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<MainMenuDefaults>()
            .add_systems(Update, seed_main_menu_config.map(drop));
        app
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(app.world().get_resource::<MainMenuConfig>().is_none());
    }

    #[test]
    fn seeds_config_when_asset_loaded() {
        let mut app = test_app();

        let defaults = MainMenuDefaults::default();
        let mut assets = app.world_mut().resource_mut::<Assets<MainMenuDefaults>>();
        let handle = assets.add(defaults);

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            input: Handle::default(),
            mainmenu: handle,
            timerui: Handle::default(),
            cell_types: vec![],
            layouts: vec![],
            archetypes: vec![],
            chipselect: Handle::default(),
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
        });

        app.update();

        assert!(app.world().get_resource::<MainMenuConfig>().is_some());
    }
}
