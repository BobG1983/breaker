//! Seeds `BoltConfig` from loaded `BoltDefaults`.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    bolt::{BoltConfig, BoltDefaults},
    screen::loading::resources::DefaultsCollection,
};

/// Reads the loaded `BoltDefaults` asset and inserts `BoltConfig`.
pub(crate) fn seed_bolt_config(
    collection: Option<Res<DefaultsCollection>>,
    assets: Res<Assets<BoltDefaults>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let Some(defaults) = assets.get(&collection.bolt) else {
        return Progress { done: 0, total: 1 };
    };

    commands.insert_resource::<BoltConfig>(defaults.clone().into());
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<BoltDefaults>()
            .add_systems(Update, seed_bolt_config.map(drop));
        app
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(app.world().get_resource::<BoltConfig>().is_none());
    }

    #[test]
    fn seeds_config_when_asset_loaded() {
        let mut app = test_app();

        let defaults = BoltDefaults::default();
        let mut assets = app.world_mut().resource_mut::<Assets<BoltDefaults>>();
        let handle = assets.add(defaults);

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: Handle::default(),
            bolt: handle,
            breaker: Handle::default(),
            cells: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            cell_types: vec![],
            layouts: vec![],
            archetypes: vec![],
            chipselect: Handle::default(),
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
            difficulty: Handle::default(),
        });

        app.update();

        assert!(app.world().get_resource::<BoltConfig>().is_some());
    }
}
