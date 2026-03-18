//! Seeds `PlayfieldConfig` from loaded `PlayfieldDefaults`.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    screen::loading::resources::DefaultsCollection,
    shared::{PlayfieldConfig, PlayfieldDefaults},
};

/// Reads the loaded `PlayfieldDefaults` asset and inserts `PlayfieldConfig`.
pub(crate) fn seed_playfield_config(
    collection: Option<Res<DefaultsCollection>>,
    assets: Res<Assets<PlayfieldDefaults>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let Some(defaults) = assets.get(&collection.playfield) else {
        return Progress { done: 0, total: 1 };
    };

    commands.insert_resource::<PlayfieldConfig>(defaults.clone().into());
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<PlayfieldDefaults>()
            .add_systems(Update, seed_playfield_config.map(drop));
        app
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(app.world().get_resource::<PlayfieldConfig>().is_none());
    }

    #[test]
    fn seeds_config_when_asset_loaded() {
        let mut app = test_app();

        let defaults = PlayfieldDefaults::default();
        let mut assets = app.world_mut().resource_mut::<Assets<PlayfieldDefaults>>();
        let handle = assets.add(defaults);

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: handle,
            bolt: Handle::default(),
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
        });

        app.update();

        assert!(app.world().get_resource::<PlayfieldConfig>().is_some());
    }
}
